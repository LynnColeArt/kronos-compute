//! Memory allocation and management

use crate::sys::*;
use crate::core::*;
use crate::ffi::*;

/// Allocate device memory
#[no_mangle]
pub unsafe extern "C" fn vkAllocateMemory(
    device: VkDevice,
    pAllocateInfo: *const VkMemoryAllocateInfo,
    pAllocator: *const VkAllocationCallbacks,
    pMemory: *mut VkDeviceMemory,
) -> VkResult {
    if device.is_null() || pAllocateInfo.is_null() || pMemory.is_null() {
        return VkResult::ErrorInitializationFailed;
    }
    
    // Forward to real driver
    if let Some(icd) = super::forward::get_icd_if_enabled() {
        if let Some(allocate_memory) = icd.allocate_memory {
            return allocate_memory(device, pAllocateInfo, pAllocator, pMemory);
        }
    }
    
    // No ICD available
    VkResult::ErrorInitializationFailed
}

/// Free device memory
#[no_mangle]
pub unsafe extern "C" fn vkFreeMemory(
    device: VkDevice,
    memory: VkDeviceMemory,
    pAllocator: *const VkAllocationCallbacks,
) {
    if device.is_null() || memory.is_null() {
        return;
    }
    
    // Forward to real driver
    if let Some(icd) = super::forward::get_icd_if_enabled() {
        if let Some(free_memory) = icd.free_memory {
            free_memory(device, memory, pAllocator);
        }
    }
}

/// Map memory for CPU access
#[no_mangle]
pub unsafe extern "C" fn vkMapMemory(
    device: VkDevice,
    memory: VkDeviceMemory,
    offset: VkDeviceSize,
    size: VkDeviceSize,
    flags: VkMemoryMapFlags,
    ppData: *mut *mut libc::c_void,
) -> VkResult {
    if device.is_null() || memory.is_null() || ppData.is_null() {
        return VkResult::ErrorInitializationFailed;
    }
    
    // Forward to real driver
    if let Some(icd) = super::forward::get_icd_if_enabled() {
        if let Some(map_memory) = icd.map_memory {
            return map_memory(device, memory, offset, size, flags, ppData);
        }
    }
    
    // No ICD available
    VkResult::ErrorInitializationFailed
}

/// Unmap memory
#[no_mangle]
pub unsafe extern "C" fn vkUnmapMemory(
    device: VkDevice,
    memory: VkDeviceMemory,
) {
    if device.is_null() || memory.is_null() {
        return;
    }
    
    // Forward to real driver
    if let Some(icd) = super::forward::get_icd_if_enabled() {
        if let Some(unmap_memory) = icd.unmap_memory {
            unmap_memory(device, memory);
        }
    }
}