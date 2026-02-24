//! Persistent descriptor management for optimal performance
//! 
//! Implements Set0 as persistent storage buffer descriptors that are:
//! - Created once per buffer lifetime
//! - Never updated in hot path
//! - Parameters passed via push constants (≤128B)

use std::collections::HashMap;
use std::sync::Mutex;
use crate::sys::*;
use crate::core::*;
use crate::ffi::*;
use super::error::IcdError;

/// Maximum push constant size (typical hardware limit)
pub const MAX_PUSH_CONSTANT_SIZE: u32 = 128;

/// Descriptor set 0 is reserved for persistent storage buffers
pub const PERSISTENT_DESCRIPTOR_SET: u32 = 0;

/// Persistent descriptor cache entry
struct PersistentDescriptor {
    descriptor_set: VkDescriptorSet,
    buffers: Vec<VkBuffer>,
    generation: u64,
}

/// Global persistent descriptor manager
pub struct PersistentDescriptorManager {
    /// Device -> Pool mapping
    pools: HashMap<u64, VkDescriptorPool>,
    
    /// Layout for Set0 (storage buffers only)
    set0_layout: HashMap<u64, VkDescriptorSetLayout>,
    
    /// Buffer -> Descriptor mapping
    descriptors: HashMap<u64, PersistentDescriptor>,
    /// Device -> descriptor cache keys (for deterministic cleanup)
    descriptors_by_device: HashMap<u64, Vec<u64>>,
    
    /// Generation counter for cache invalidation
    generation: u64,
}

lazy_static::lazy_static! {
    static ref DESCRIPTOR_MANAGER: Mutex<PersistentDescriptorManager> = Mutex::new(PersistentDescriptorManager {
        pools: HashMap::new(),
        set0_layout: HashMap::new(),
        descriptors: HashMap::new(),
        descriptors_by_device: HashMap::new(),
        generation: 0,
    });
}

/// Create Set0 layout for storage buffers
///
/// # Safety
///
/// This function is unsafe because:
/// - The device must be a valid VkDevice handle
/// - Calls vkCreateDescriptorSetLayout through ICD function pointer
/// - The returned layout must be destroyed with vkDestroyDescriptorSetLayout
/// - Invalid device handle will cause undefined behavior
/// - The ICD must be properly initialized with valid function pointers
pub unsafe fn create_persistent_layout(
    device: VkDevice,
    max_bindings: u32,
) -> Result<VkDescriptorSetLayout, IcdError> {
    let mut manager = DESCRIPTOR_MANAGER.lock()?;
    let device_key = device.as_raw();
    
    // Return existing layout if already created
    if let Some(&layout) = manager.set0_layout.get(&device_key) {
        return Ok(layout);
    }
    
    // Create bindings for storage buffers
    let mut bindings = Vec::with_capacity(max_bindings as usize);
    for i in 0..max_bindings {
        bindings.push(VkDescriptorSetLayoutBinding {
            binding: i,
            descriptorType: VkDescriptorType::StorageBuffer,
            descriptorCount: 1,
            stageFlags: VkShaderStageFlags::COMPUTE,
            pImmutableSamplers: std::ptr::null(),
        });
    }
    
    let create_info = VkDescriptorSetLayoutCreateInfo {
        sType: VkStructureType::DescriptorSetLayoutCreateInfo,
        pNext: std::ptr::null(),
        flags: 0,
        bindingCount: bindings.len() as u32,
        pBindings: bindings.as_ptr(),
    };
    
    // Forward to ICD
    if let Some(icd) = super::icd_loader::get_icd() {
        if let Some(create_fn) = icd.create_descriptor_set_layout {
            let mut layout = VkDescriptorSetLayout::NULL;
            let result = create_fn(device, &create_info, std::ptr::null(), &mut layout);
            
            if result == VkResult::Success {
                manager.set0_layout.insert(device_key, layout);
                return Ok(layout);
            }
            return Err(IcdError::VulkanError(result));
        }
    }
    
    Err(IcdError::MissingFunction("vkCreateDescriptorSetLayout"))
}

/// Create or get persistent descriptor pool
///
/// # Safety
///
/// This function is unsafe because:
/// - The device must be a valid VkDevice handle
/// - Calls vkCreateDescriptorPool through ICD function pointer
/// - The returned pool must be destroyed with vkDestroyDescriptorPool
/// - Pool limits (max_sets, max_descriptors) must not exceed device limits
/// - Invalid device handle will cause undefined behavior
/// - Thread safety relies on the Mutex protecting the global manager
pub unsafe fn get_persistent_pool(
    device: VkDevice,
    max_sets: u32,
    max_descriptors: u32,
) -> Result<VkDescriptorPool, IcdError> {
    let mut manager = DESCRIPTOR_MANAGER.lock()?;
    let device_key = device.as_raw();
    
    // Return existing pool if already created
    if let Some(&pool) = manager.pools.get(&device_key) {
        return Ok(pool);
    }
    
    // Create pool for storage buffer descriptors only
    let pool_size = VkDescriptorPoolSize {
        type_: VkDescriptorType::StorageBuffer,
        descriptorCount: max_descriptors,
    };
    
    let create_info = VkDescriptorPoolCreateInfo {
        sType: VkStructureType::DescriptorPoolCreateInfo,
        pNext: std::ptr::null(),
        flags: VkDescriptorPoolCreateFlags::FREE_DESCRIPTOR_SET,
        maxSets: max_sets,
        poolSizeCount: 1,
        pPoolSizes: &pool_size,
    };
    
    // Forward to ICD
    if let Some(icd) = super::icd_loader::get_icd() {
        if let Some(create_fn) = icd.create_descriptor_pool {
            let mut pool = VkDescriptorPool::NULL;
            let result = create_fn(device, &create_info, std::ptr::null(), &mut pool);
            
            if result == VkResult::Success {
                manager.pools.insert(device_key, pool);
                return Ok(pool);
            }
            return Err(IcdError::VulkanError(result));
        }
    }
    
    Err(IcdError::MissingFunction("vkCreateDescriptorPool"))
}

/// Get or create persistent descriptor set for buffers
///
/// # Safety
///
/// This function is unsafe because:
/// - The device must be a valid VkDevice handle
/// - All buffers in the array must be valid VkBuffer handles
/// - Calls multiple Vulkan functions through ICD pointers
/// - The descriptor set references the provided buffers
/// - Buffers must remain valid for the lifetime of the descriptor set
/// - Buffer usage must be compatible with STORAGE_BUFFER descriptor type
pub unsafe fn get_persistent_descriptor_set(
    device: VkDevice,
    buffers: &[VkBuffer],
) -> Result<VkDescriptorSet, IcdError> {
    let mut manager = DESCRIPTOR_MANAGER.lock()?;
    let device_key = device.as_raw();
    
    // Create cache key from buffer handles
    let binding_signature = buffers.iter()
        .map(|b| b.as_raw())
        .fold(0u64, |acc, h| acc.wrapping_mul(0x9e3779b185ebca87) ^ h.rotate_left(13));
    let cache_key = device_key.wrapping_mul(0x9e3779b97f4a7c15) ^ binding_signature;
    
    // Check if we already have this descriptor set
    if let Some(descriptor) = manager.descriptors.get(&cache_key) {
        if descriptor.buffers == buffers {
            return Ok(descriptor.descriptor_set);
        }
    }
    
    // Get or create layout and pool
    let layout = create_persistent_layout(device, buffers.len() as u32)?;
    let pool = get_persistent_pool(device, 1000, 10000)?;
    
    // Allocate descriptor set
    let alloc_info = VkDescriptorSetAllocateInfo {
        sType: VkStructureType::DescriptorSetAllocateInfo,
        pNext: std::ptr::null(),
        descriptorPool: pool,
        descriptorSetCount: 1,
        pSetLayouts: &layout,
    };
    
    let mut descriptor_set = VkDescriptorSet::NULL;
    
    if let Some(icd) = super::icd_loader::get_icd() {
        if let Some(alloc_fn) = icd.allocate_descriptor_sets {
            let result = alloc_fn(device, &alloc_info, &mut descriptor_set);
            if result != VkResult::Success {
                return Err(IcdError::VulkanError(result));
            }
        } else {
            return Err(IcdError::MissingFunction("vkAllocateDescriptorSets"));
        }
    } else {
        return Err(IcdError::NoIcdLoaded);
    }
    
    // Write descriptor set with buffer bindings
    let mut buffer_infos = Vec::with_capacity(buffers.len());
    let mut writes = Vec::with_capacity(buffers.len());
    
    for (_i, &buffer) in buffers.iter().enumerate() {
        buffer_infos.push(VkDescriptorBufferInfo {
            buffer,
            offset: 0,
            range: VK_WHOLE_SIZE,
        });
    }
    
    for (i, buffer_info) in buffer_infos.iter().enumerate() {
        writes.push(VkWriteDescriptorSet {
            sType: VkStructureType::WriteDescriptorSet,
            pNext: std::ptr::null(),
            dstSet: descriptor_set,
            dstBinding: i as u32,
            dstArrayElement: 0,
            descriptorCount: 1,
            descriptorType: VkDescriptorType::StorageBuffer,
            pImageInfo: std::ptr::null(),
            pBufferInfo: buffer_info,
            pTexelBufferView: std::ptr::null(),
        });
    }
    
    if let Some(icd) = super::icd_loader::get_icd() {
        if let Some(update_fn) = icd.update_descriptor_sets {
            update_fn(device, writes.len() as u32, writes.as_ptr(), 0, std::ptr::null());
        }
    }
    
    // Cache the descriptor
    manager.generation += 1;
    let generation = manager.generation;
    let descriptors_for_device = manager
        .descriptors_by_device
        .entry(device_key)
        .or_default();
    descriptors_for_device.retain(|key| *key != cache_key);
    descriptors_for_device.push(cache_key);
    manager.descriptors.insert(cache_key, PersistentDescriptor {
        descriptor_set,
        buffers: buffers.to_vec(),
        generation,
    });
    
    Ok(descriptor_set)
}

/// Create push constant range for parameters
pub fn create_push_constant_range(size: u32) -> VkPushConstantRange {
    assert!(size <= MAX_PUSH_CONSTANT_SIZE, "Push constant size {} exceeds limit {}", size, MAX_PUSH_CONSTANT_SIZE);
    
    VkPushConstantRange {
        stageFlags: VkShaderStageFlags::COMPUTE,
        offset: 0,
        size,
    }
}

/// Create optimized pipeline layout with Set0 + push constants
///
/// # Safety
///
/// This function is unsafe because:
/// - The device must be a valid VkDevice handle
/// - Calls vkCreatePipelineLayout through ICD function pointer
/// - The returned layout must be destroyed with vkDestroyPipelineLayout
/// - push_constant_size must not exceed MAX_PUSH_CONSTANT_SIZE (128 bytes)
/// - set0_binding_count must not exceed device limits
/// - Invalid parameters may cause device lost or undefined behavior
pub unsafe fn create_compute_pipeline_layout(
    device: VkDevice,
    set0_binding_count: u32,
    push_constant_size: u32,
) -> Result<VkPipelineLayout, IcdError> {
    let set0_layout = create_persistent_layout(device, set0_binding_count)?;
    
    let mut create_info = VkPipelineLayoutCreateInfo {
        sType: VkStructureType::PipelineLayoutCreateInfo,
        pNext: std::ptr::null(),
        flags: 0,
        setLayoutCount: 1,
        pSetLayouts: &set0_layout,
        pushConstantRangeCount: 0,
        pPushConstantRanges: std::ptr::null(),
    };
    
    let push_range = if push_constant_size > 0 {
        Some(create_push_constant_range(push_constant_size))
    } else {
        None
    };
    
    if let Some(ref range) = push_range {
        create_info.pushConstantRangeCount = 1;
        create_info.pPushConstantRanges = range;
    }
    
    let mut layout = VkPipelineLayout::NULL;
    
    if let Some(icd) = super::icd_loader::get_icd() {
        if let Some(create_fn) = icd.create_pipeline_layout {
            let result = create_fn(device, &create_info, std::ptr::null(), &mut layout);
            if result == VkResult::Success {
                return Ok(layout);
            }
            return Err(IcdError::VulkanError(result));
        }
    }
    
    Err(IcdError::MissingFunction("vkCreatePipelineLayout"))
}

/// Cleanup persistent descriptors for a device
///
/// # Safety
///
/// This function is unsafe because:
/// - The device must be a valid VkDevice handle
/// - Calls vkDestroyDescriptorPool and vkDestroyDescriptorSetLayout
/// - All descriptor sets allocated from the pool become invalid
/// - Must be called before device destruction
/// - Concurrent use of descriptors during cleanup causes undefined behavior
/// - The global manager mutex provides thread safety for the cleanup
pub unsafe fn cleanup_persistent_descriptors(device: VkDevice) -> Result<(), IcdError> {
    let mut manager = DESCRIPTOR_MANAGER.lock()?;
    let device_key = device.as_raw();
    
    // Clean up pool
    if let Some(pool) = manager.pools.remove(&device_key) {
        if let Some(icd) = super::icd_loader::get_icd() {
            if let Some(destroy_fn) = icd.destroy_descriptor_pool {
                destroy_fn(device, pool, std::ptr::null());
            }
        }
    }
    
    // Clean up layout
    if let Some(layout) = manager.set0_layout.remove(&device_key) {
        if let Some(icd) = super::icd_loader::get_icd() {
            if let Some(destroy_fn) = icd.destroy_descriptor_set_layout {
                destroy_fn(device, layout, std::ptr::null());
            }
        }
    }
    
    // Remove cached descriptors for this device
    if let Some(keys) = manager.descriptors_by_device.remove(&device_key) {
        for key in keys {
            manager.descriptors.remove(&key);
        }
    }
    
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_push_constant_range() {
        let range = create_push_constant_range(64);
        assert_eq!(range.stageFlags, VkShaderStageFlags::COMPUTE);
        assert_eq!(range.offset, 0);
        assert_eq!(range.size, 64);
    }
    
    #[test]
    #[should_panic]
    fn test_push_constant_size_limit() {
        create_push_constant_range(256); // Exceeds 128 byte limit
    }
}
