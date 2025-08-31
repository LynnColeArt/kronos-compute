//! REAL Kronos memory implementation - NO ICD forwarding!

use crate::sys::*;
use crate::core::*;
use crate::ffi::*;
use std::ptr;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Mutex;
use std::collections::HashMap;

// Memory handle counter
static MEMORY_COUNTER: AtomicU64 = AtomicU64::new(1);

// Registry of active memory allocations
lazy_static::lazy_static! {
    static ref MEMORY_ALLOCS: Mutex<HashMap<u64, MemoryData>> = Mutex::new(HashMap::new());
}

struct MemoryData {
    device: VkDevice,
    size: VkDeviceSize,
    memory_type_index: u32,
    data: Vec<u8>,
    mapped: bool,
}

/// Allocate device memory - REAL implementation
#[no_mangle]
pub unsafe extern "C" fn vkAllocateMemory(
    device: VkDevice,
    pAllocateInfo: *const VkMemoryAllocateInfo,
    _pAllocator: *const VkAllocationCallbacks,
    pMemory: *mut VkDeviceMemory,
) -> VkResult {
    log::info!("=== KRONOS vkAllocateMemory called (Pure Rust) ===");
    
    if device.is_null() || pAllocateInfo.is_null() || pMemory.is_null() {
        return VkResult::ErrorInitializationFailed;
    }
    
    let alloc_info = &*pAllocateInfo;
    
    // Validate allocation size
    if alloc_info.allocationSize == 0 {
        return VkResult::ErrorInitializationFailed;
    }
    
    // Create memory handle
    let handle = MEMORY_COUNTER.fetch_add(1, Ordering::SeqCst);
    
    // Allocate actual memory
    let data = vec![0u8; alloc_info.allocationSize as usize];
    
    // Store memory data
    let memory_data = MemoryData {
        device,
        size: alloc_info.allocationSize,
        memory_type_index: alloc_info.memoryTypeIndex,
        data,
        mapped: false,
    };
    
    MEMORY_ALLOCS.lock().unwrap().insert(handle, memory_data);
    
    *pMemory = VkDeviceMemory::from_raw(handle);
    
    log::info!("Allocated {} bytes of memory as handle {:?}", alloc_info.allocationSize, handle);
    
    VkResult::Success
}

/// Free device memory
#[no_mangle]
pub unsafe extern "C" fn vkFreeMemory(
    device: VkDevice,
    memory: VkDeviceMemory,
    _pAllocator: *const VkAllocationCallbacks,
) {
    if device.is_null() || memory.is_null() {
        return;
    }
    
    let handle = memory.as_raw();
    MEMORY_ALLOCS.lock().unwrap().remove(&handle);
    
    log::info!("Freed memory {:?}", handle);
}

/// Map device memory
#[no_mangle]
pub unsafe extern "C" fn vkMapMemory(
    device: VkDevice,
    memory: VkDeviceMemory,
    offset: VkDeviceSize,
    size: VkDeviceSize,
    _flags: VkMemoryMapFlags,
    ppData: *mut *mut std::ffi::c_void,
) -> VkResult {
    if device.is_null() || memory.is_null() || ppData.is_null() {
        return VkResult::ErrorInitializationFailed;
    }
    
    let handle = memory.as_raw();
    if let Some(memory_data) = MEMORY_ALLOCS.lock().unwrap().get_mut(&handle) {
        // Validate offset and size
        let map_size = if size == VK_WHOLE_SIZE {
            memory_data.size - offset
        } else {
            size
        };
        
        if offset + map_size > memory_data.size {
            return VkResult::ErrorMemoryMapFailed;
        }
        
        // Return pointer to our data
        let ptr = memory_data.data.as_mut_ptr().add(offset as usize);
        *ppData = ptr as *mut std::ffi::c_void;
        
        memory_data.mapped = true;
        
        log::info!("Mapped memory {:?} at offset {} size {}", handle, offset, map_size);
        
        VkResult::Success
    } else {
        VkResult::ErrorMemoryMapFailed
    }
}

/// Unmap device memory
#[no_mangle]
pub unsafe extern "C" fn vkUnmapMemory(
    device: VkDevice,
    memory: VkDeviceMemory,
) {
    if device.is_null() || memory.is_null() {
        return;
    }
    
    let handle = memory.as_raw();
    if let Some(memory_data) = MEMORY_ALLOCS.lock().unwrap().get_mut(&handle) {
        memory_data.mapped = false;
        log::info!("Unmapped memory {:?}", handle);
    }
}