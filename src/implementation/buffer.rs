//! Buffer creation and management

use crate::sys::*;
use crate::core::*;
use crate::ffi::*;

/// Create a buffer
// SAFETY: This function is called from C code. Caller must ensure:
// 1. device is a valid VkDevice created by vkCreateDevice
// 2. pCreateInfo points to a valid VkBufferCreateInfo structure
// 3. pAllocator is either null or points to valid allocation callbacks
// 4. pBuffer points to valid memory for writing the buffer handle
// 5. All fields in pCreateInfo are valid (size > 0, valid usage flags, etc.)
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
// SAFETY: This function is called from C code. Caller must ensure:
// 1. device is a valid VkDevice
// 2. buffer is a valid VkBuffer created by vkCreateBuffer, or VK_NULL_HANDLE
// 3. pAllocator matches the allocator used in vkCreateBuffer (or both are null)
// 4. The buffer is not currently bound to memory or in use by any operations
// 5. All command buffers using this buffer have completed execution
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
// SAFETY: This function is called from C code. Caller must ensure:
// 1. device is a valid VkDevice
// 2. buffer is a valid VkBuffer created by vkCreateBuffer
// 3. pMemoryRequirements points to valid memory for a VkMemoryRequirements structure
// 4. The buffer has not been destroyed
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
// SAFETY: This function is called from C code. Caller must ensure:
// 1. device is a valid VkDevice
// 2. buffer is a valid VkBuffer that has not been bound to memory yet
// 3. memory is a valid VkDeviceMemory allocated with vkAllocateMemory
// 4. memoryOffset + buffer.size <= memory.size (fits within allocated memory)
// 5. The memory type is compatible with the buffer's memory requirements
// 6. Neither buffer nor memory are currently in use by GPU operations
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