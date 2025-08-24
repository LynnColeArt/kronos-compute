//! Buffer creation and management

use crate::sys::*;
use crate::core::*;
use crate::ffi::*;

/// Create a buffer
#[no_mangle]
pub unsafe extern "C" fn vkCreateBuffer(
    device: VkDevice,
    pCreateInfo: *const VkBufferCreateInfo,
    pAllocator: *const VkAllocationCallbacks,
    pBuffer: *mut VkBuffer,
) -> VkResult {
    if device.is_null() || pCreateInfo.is_null() || pBuffer.is_null() {
        return VkResult::ErrorInitializationFailed;
    }
    
    // Forward to real ICD
    if let Some(icd) = super::forward::get_icd_if_enabled() {
        if let Some(create_buffer) = icd.create_buffer {
            return create_buffer(device, pCreateInfo, pAllocator, pBuffer);
        }
    }
    
    // No ICD available
    VkResult::ErrorInitializationFailed
}

/// Destroy a buffer
#[no_mangle]
pub unsafe extern "C" fn vkDestroyBuffer(
    device: VkDevice,
    buffer: VkBuffer,
    pAllocator: *const VkAllocationCallbacks,
) {
    if device.is_null() || buffer.is_null() {
        return;
    }
    
    // Forward to real ICD
    if let Some(icd) = super::forward::get_icd_if_enabled() {
        if let Some(destroy_buffer) = icd.destroy_buffer {
            destroy_buffer(device, buffer, pAllocator);
        }
    }
}

/// Get buffer memory requirements
#[no_mangle]
pub unsafe extern "C" fn vkGetBufferMemoryRequirements(
    device: VkDevice,
    buffer: VkBuffer,
    pMemoryRequirements: *mut VkMemoryRequirements,
) {
    if device.is_null() || buffer.is_null() || pMemoryRequirements.is_null() {
        return;
    }
    
    // Forward to real ICD
    if let Some(icd) = super::forward::get_icd_if_enabled() {
        if let Some(get_buffer_memory_requirements) = icd.get_buffer_memory_requirements {
            get_buffer_memory_requirements(device, buffer, pMemoryRequirements);
        }
    }
}

/// Bind buffer to memory
#[no_mangle]
pub unsafe extern "C" fn vkBindBufferMemory(
    device: VkDevice,
    buffer: VkBuffer,
    memory: VkDeviceMemory,
    memoryOffset: VkDeviceSize,
) -> VkResult {
    if device.is_null() || buffer.is_null() || memory.is_null() {
        return VkResult::ErrorInitializationFailed;
    }
    
    // Forward to real ICD
    if let Some(icd) = super::forward::get_icd_if_enabled() {
        if let Some(bind_buffer_memory) = icd.bind_buffer_memory {
            return bind_buffer_memory(device, buffer, memory, memoryOffset);
        }
    }
    
    // No ICD available
    VkResult::ErrorInitializationFailed
}