//! 3-pool memory allocator for zero allocation in steady state
//! 
//! Pools:
//! 1. DEVICE_LOCAL - GPU-only memory
//! 2. HOST_VISIBLE|COHERENT - Pinned staging, persistently mapped
//! 3. HOST_VISIBLE|CACHED - Readback memory

use std::collections::HashMap;
use std::sync::Mutex;
use crate::sys::*;
use crate::core::*;
use crate::ffi::*;
use super::error::IcdError;

/// Slab size for suballocation (256 KiB default)
const SLAB_SIZE: VkDeviceSize = 256 * 1024;

/// Minimum allocation size (64 KiB)
#[allow(dead_code)]
const MIN_ALLOCATION_SIZE: VkDeviceSize = 64 * 1024;

/// Memory pool types
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum PoolType {
    /// GPU-only memory
    DeviceLocal,
    /// Pinned staging memory (persistently mapped)
    HostVisibleCoherent,
    /// Readback memory
    HostVisibleCached,
}

impl PoolType {
    /// Get required memory property flags
    pub fn required_flags(&self) -> VkMemoryPropertyFlags {
        match self {
            PoolType::DeviceLocal => VkMemoryPropertyFlags::DEVICE_LOCAL,
            PoolType::HostVisibleCoherent => {
                VkMemoryPropertyFlags::HOST_VISIBLE | VkMemoryPropertyFlags::HOST_COHERENT
            }
            PoolType::HostVisibleCached => {
                VkMemoryPropertyFlags::HOST_VISIBLE | VkMemoryPropertyFlags::HOST_CACHED
            }
        }
    }
    
    /// Check if pool should be persistently mapped
    pub fn should_map(&self) -> bool {
        matches!(self, PoolType::HostVisibleCoherent | PoolType::HostVisibleCached)
    }
}

/// A single allocation within a slab
#[derive(Debug)]
struct SubAllocation {
    offset: VkDeviceSize,
    size: VkDeviceSize,
    in_use: bool,
}

/// A slab of memory that can be subdivided
struct MemorySlab {
    memory: VkDeviceMemory,
    size: VkDeviceSize,
    mapped_ptr: Option<*mut std::ffi::c_void>,
    allocations: Vec<SubAllocation>,
    free_space: VkDeviceSize,
}

// Safe to send between threads - the pointer is just an address
unsafe impl Send for MemorySlab {}
unsafe impl Sync for MemorySlab {}

impl MemorySlab {
    /// Try to allocate from this slab
    fn allocate(&mut self, size: VkDeviceSize, alignment: VkDeviceSize) -> Option<VkDeviceSize> {
        if self.free_space < size {
            return None;
        }
        
        // Find a free spot (first-fit algorithm)
        let mut current_offset = 0;
        
        for alloc in &self.allocations {
            if !alloc.in_use {
                continue;
            }
            
            // Check if we can fit before this allocation
            let aligned_offset = (current_offset + alignment - 1) & !(alignment - 1);
            if aligned_offset + size <= alloc.offset {
                // Found a spot
                self.allocations.push(SubAllocation {
                    offset: aligned_offset,
                    size,
                    in_use: true,
                });
                self.free_space -= size;
                return Some(aligned_offset);
            }
            
            current_offset = alloc.offset + alloc.size;
        }
        
        // Check if we can fit at the end
        let aligned_offset = (current_offset + alignment - 1) & !(alignment - 1);
        if aligned_offset + size <= self.size {
            self.allocations.push(SubAllocation {
                offset: aligned_offset,
                size,
                in_use: true,
            });
            self.free_space -= size;
            Some(aligned_offset)
        } else {
            None
        }
    }
    
    /// Free an allocation
    fn free(&mut self, offset: VkDeviceSize) -> bool {
        if let Some(alloc) = self.allocations.iter_mut().find(|a| a.offset == offset) {
            if alloc.in_use {
                alloc.in_use = false;
                self.free_space += alloc.size;
                return true;
            }
        }
        false
    }
}

/// Memory pool for a specific type
struct MemoryPool {
    device: VkDevice,
    pool_type: PoolType,
    memory_type_index: u32,
    slabs: Vec<MemorySlab>,
    total_allocated: VkDeviceSize,
}

impl MemoryPool {
    fn new(device: VkDevice, pool_type: PoolType, memory_type_index: u32) -> Self {
        Self {
            device,
            pool_type,
            memory_type_index,
            slabs: Vec::new(),
            total_allocated: 0,
        }
    }
    
    /// Allocate memory from the pool
    ///
    /// # Safety
    ///
    /// This function is unsafe because:
    /// - Calls vkAllocateMemory through ICD function pointer
    /// - May call vkMapMemory for host-visible memory types
    /// - The device must be a valid VkDevice handle
    /// - Returned memory must be freed with vkFreeMemory
    /// - Mapped pointers are only valid while memory is allocated
    /// - Size and alignment must be within device limits
    unsafe fn allocate(
        &mut self,
        size: VkDeviceSize,
        alignment: VkDeviceSize,
    ) -> Result<(VkDeviceMemory, VkDeviceSize, Option<*mut std::ffi::c_void>), IcdError> {
        // Try existing slabs first
        for slab in &mut self.slabs {
            if let Some(offset) = slab.allocate(size, alignment) {
                let mapped_ptr = slab.mapped_ptr.map(|ptr| {
                    (ptr as *mut u8).add(offset as usize) as *mut std::ffi::c_void
                });
                return Ok((slab.memory, offset, mapped_ptr));
            }
        }
        
        // Need a new slab
        let slab_size = SLAB_SIZE.max(size);
        
        let alloc_info = VkMemoryAllocateInfo {
            sType: VkStructureType::MemoryAllocateInfo,
            pNext: std::ptr::null(),
            allocationSize: slab_size,
            memoryTypeIndex: self.memory_type_index,
        };
        
        let mut memory = VkDeviceMemory::NULL;
        
        if let Some(icd) = super::icd_loader::get_icd() {
            if let Some(alloc_fn) = icd.allocate_memory {
                let result = alloc_fn(self.device, &alloc_info, std::ptr::null(), &mut memory);
                if result != VkResult::Success {
                    return Err(IcdError::VulkanError(result));
                }
            } else {
                return Err(IcdError::MissingFunction("vkAllocateMemory"));
            }
        } else {
            return Err(IcdError::NoIcdLoaded);
        }
        
        // Map if needed
        let mapped_ptr = if self.pool_type.should_map() {
            let mut ptr = std::ptr::null_mut();
            if let Some(icd) = super::icd_loader::get_icd() {
                if let Some(map_fn) = icd.map_memory {
                    let result = map_fn(self.device, memory, 0, VK_WHOLE_SIZE, 0, &mut ptr);
                    if result == VkResult::Success {
                        Some(ptr)
                    } else {
                        None
                    }
                } else {
                    None
                }
            } else {
                None
            }
        } else {
            None
        };
        
        // Create new slab
        let mut slab = MemorySlab {
            memory,
            size: slab_size,
            mapped_ptr,
            allocations: Vec::new(),
            free_space: slab_size,
        };
        
        // Allocate from new slab
        let offset = slab.allocate(size, alignment)
            .expect("New slab should have space");
        
        let result_ptr = mapped_ptr.map(|ptr| {
            (ptr as *mut u8).add(offset as usize) as *mut std::ffi::c_void
        });
        
        self.slabs.push(slab);
        self.total_allocated += slab_size;
        
        Ok((memory, offset, result_ptr))
    }
    
    /// Free an allocation
    ///
    /// # Safety
    ///
    /// This function is unsafe because:
    /// - The memory and offset must correspond to a valid allocation
    /// - The allocation must not be in use by the GPU
    /// - After freeing, any mapped pointers become invalid
    /// - Double-free will corrupt the allocator state
    unsafe fn free(&mut self, memory: VkDeviceMemory, offset: VkDeviceSize) -> bool {
        for slab in &mut self.slabs {
            if slab.memory == memory {
                return slab.free(offset);
            }
        }
        false
    }
}

/// Allocation handle
#[derive(Debug, Clone, Copy)]
pub struct AllocationHandle {
    memory: VkDeviceMemory,
    offset: VkDeviceSize,
    size: VkDeviceSize,
    pool_type: PoolType,
    mapped_ptr: Option<*mut std::ffi::c_void>,
}

// Safe to send between threads - the pointer is just an address
unsafe impl Send for AllocationHandle {}
unsafe impl Sync for AllocationHandle {}

impl AllocationHandle {
    /// Get the device memory handle
    pub fn memory(&self) -> VkDeviceMemory {
        self.memory
    }
    
    /// Get the offset within the memory
    pub fn offset(&self) -> VkDeviceSize {
        self.offset
    }
    
    /// Get the allocation size
    pub fn size(&self) -> VkDeviceSize {
        self.size
    }
    
    /// Get mapped pointer if available
    pub fn mapped_ptr(&self) -> Option<*mut std::ffi::c_void> {
        self.mapped_ptr
    }
}

/// Global pool allocator
pub struct PoolAllocator {
    pools: HashMap<(u64, PoolType), MemoryPool>,
    allocations: HashMap<u64, AllocationHandle>,
    next_id: u64,
}

lazy_static::lazy_static! {
    static ref POOL_ALLOCATOR: Mutex<PoolAllocator> = Mutex::new(PoolAllocator {
        pools: HashMap::new(),
        allocations: HashMap::new(),
        next_id: 1,
    });
}

/// Initialize pools for a device
///
/// # Safety
///
/// This function is unsafe because:
/// - Both device and physical_device must be valid Vulkan handles
/// - Calls vkGetPhysicalDeviceMemoryProperties through ICD
/// - The device must have been created from the physical device
/// - Pools must be cleaned up before device destruction
/// - Thread safety is provided by the global POOL_ALLOCATOR mutex
pub unsafe fn initialize_pools(
    device: VkDevice,
    physical_device: VkPhysicalDevice,
) -> Result<(), IcdError> {
    let mut allocator = POOL_ALLOCATOR.lock()?;
    
    // Get memory properties
    let mut mem_props = VkPhysicalDeviceMemoryProperties::default();
    if let Some(icd) = super::icd_loader::get_icd() {
        if let Some(get_props_fn) = icd.get_physical_device_memory_properties {
            get_props_fn(physical_device, &mut mem_props);
        }
    }
    
    // Find memory types for each pool
    for pool_type in &[PoolType::DeviceLocal, PoolType::HostVisibleCoherent, PoolType::HostVisibleCached] {
        let required_flags = pool_type.required_flags();
        
        for i in 0..mem_props.memoryTypeCount {
            let mem_type = &mem_props.memoryTypes[i as usize];
            if mem_type.propertyFlags.contains(required_flags) {
                let key = (device.as_raw(), *pool_type);
                allocator.pools.insert(key, MemoryPool::new(device, *pool_type, i));
                break;
            }
        }
    }
    
    Ok(())
}

/// Allocate memory from appropriate pool
///
/// # Safety
///
/// This function is unsafe because:
/// - The device must be a valid VkDevice handle
/// - Pools must be initialized for the device first
/// - The requirements must be valid (from vkGetBufferMemoryRequirements etc.)
/// - The returned allocation ID must be freed with free_allocation
/// - Memory allocated is not bound to any resource yet
pub unsafe fn allocate_from_pool(
    device: VkDevice,
    requirements: &VkMemoryRequirements,
    pool_type: PoolType,
) -> Result<u64, IcdError> {
    let mut allocator = POOL_ALLOCATOR.lock()?;
    
    let key = (device.as_raw(), pool_type);
    let pool = allocator.pools.get_mut(&key)
        .ok_or(IcdError::InvalidOperation("Pool not initialized"))?;
    
    let (memory, offset, mapped_ptr) = pool.allocate(requirements.size, requirements.alignment)?;
    
    let handle = AllocationHandle {
        memory,
        offset,
        size: requirements.size,
        pool_type,
        mapped_ptr,
    };
    
    let id = allocator.next_id;
    allocator.next_id += 1;
    allocator.allocations.insert(id, handle);
    
    Ok(id)
}

/// Get allocation handle
pub fn get_allocation(id: u64) -> Result<AllocationHandle, IcdError> {
    let allocator = POOL_ALLOCATOR.lock()?;
    allocator.allocations.get(&id)
        .copied()
        .ok_or(IcdError::InvalidOperation("Invalid allocation ID"))
}

/// Free allocation
///
/// # Safety
///
/// This function is unsafe because:
/// - The device must be a valid VkDevice handle
/// - The allocation ID must be valid and not already freed
/// - Any resources bound to this memory must be destroyed first
/// - Any mapped pointers from this allocation become invalid
/// - GPU must not be using the memory
pub unsafe fn free_allocation(device: VkDevice, id: u64) -> Result<(), IcdError> {
    let mut allocator = POOL_ALLOCATOR.lock()?;
    
    let handle = allocator.allocations.remove(&id)
        .ok_or(IcdError::InvalidOperation("Invalid allocation ID"))?;
    
    let key = (device.as_raw(), handle.pool_type);
    if let Some(pool) = allocator.pools.get_mut(&key) {
        pool.free(handle.memory, handle.offset);
    }
    
    Ok(())
}

/// Get pool statistics
#[derive(Debug, Default)]
pub struct PoolStats {
    pub total_allocated: VkDeviceSize,
    pub total_slabs: usize,
    pub allocations_in_flight: usize,
}

pub fn get_pool_stats(device: VkDevice, pool_type: PoolType) -> Result<PoolStats, IcdError> {
    let allocator = POOL_ALLOCATOR.lock()?;
    
    let key = (device.as_raw(), pool_type);
    if let Some(pool) = allocator.pools.get(&key) {
        Ok(PoolStats {
            total_allocated: pool.total_allocated,
            total_slabs: pool.slabs.len(),
            allocations_in_flight: allocator.allocations.values()
                .filter(|a| a.pool_type == pool_type)
                .count(),
        })
    } else {
        Ok(PoolStats::default())
    }
}

/// Helper to allocate buffer memory
///
/// # Safety
///
/// This function is unsafe because:
/// - Both device and buffer must be valid Vulkan handles
/// - Calls vkGetBufferMemoryRequirements and vkBindBufferMemory
/// - The buffer must not already have memory bound
/// - The pool type must be compatible with buffer usage
/// - On failure, the allocation is automatically freed
/// - The returned allocation ID owns the memory binding
pub unsafe fn allocate_buffer_memory(
    device: VkDevice,
    buffer: VkBuffer,
    pool_type: PoolType,
) -> Result<u64, IcdError> {
    let mut requirements = VkMemoryRequirements::default();
    
    if let Some(icd) = super::icd_loader::get_icd() {
        if let Some(get_reqs_fn) = icd.get_buffer_memory_requirements {
            get_reqs_fn(device, buffer, &mut requirements);
        }
    }
    
    let allocation_id = allocate_from_pool(device, &requirements, pool_type)?;
    let handle = get_allocation(allocation_id)?;
    
    // Bind buffer to memory
    if let Some(icd) = super::icd_loader::get_icd() {
        if let Some(bind_fn) = icd.bind_buffer_memory {
            let result = bind_fn(device, buffer, handle.memory, handle.offset);
            if result != VkResult::Success {
                free_allocation(device, allocation_id)?;
                return Err(IcdError::VulkanError(result));
            }
        }
    }
    
    Ok(allocation_id)
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_pool_type_flags() {
        assert_eq!(
            PoolType::DeviceLocal.required_flags(),
            VkMemoryPropertyFlags::DEVICE_LOCAL
        );
        assert!(PoolType::HostVisibleCoherent.should_map());
        assert!(!PoolType::DeviceLocal.should_map());
    }
    
    #[test]
    fn test_slab_allocation() {
        let memory = VkDeviceMemory::from_raw(0x1234);
        let mut slab = MemorySlab {
            memory,
            size: 1024,
            mapped_ptr: None,
            allocations: Vec::new(),
            free_space: 1024,
        };
        
        // Test allocation
        let offset1 = slab.allocate(256, 16).unwrap();
        assert_eq!(offset1, 0);
        assert_eq!(slab.free_space, 768);
        
        let offset2 = slab.allocate(256, 16).unwrap();
        assert_eq!(offset2, 256);
        assert_eq!(slab.free_space, 512);
        
        // Test free
        assert!(slab.free(offset1));
        assert_eq!(slab.free_space, 768);
    }
}