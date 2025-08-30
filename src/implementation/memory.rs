//! Memory allocation and management

use crate::sys::*;
use crate::core::*;
use crate::ffi::*;
use crate::implementation::icd_loader;

/// Allocate device memory
// SAFETY: This function is called from C code. Caller must ensure:
// 1. device is a valid VkDevice
// 2. pAllocateInfo points to a valid VkMemoryAllocateInfo structure
// 3. pAllocator is either null or points to valid allocation callbacks
// 4. pMemory points to valid memory for writing the memory handle
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
    
    if let Some(icd) = icd_loader::icd_for_device(device) {
        if let Some(f) = icd.allocate_memory { return f(device, pAllocateInfo, pAllocator, pMemory); }
    }
    if let Some(icd) = super::forward::get_icd_if_enabled() {
        if let Some(allocate_memory) = icd.allocate_memory { return allocate_memory(device, pAllocateInfo, pAllocator, pMemory); }
    }
    VkResult::ErrorInitializationFailed
}

/// Free device memory
// SAFETY: This function is called from C code. Caller must ensure:
// 1. device is a valid VkDevice
// 2. memory is a valid VkDeviceMemory allocated with vkAllocateMemory
// 3. pAllocator matches the allocator used in vkAllocateMemory (or both are null)
// 4. The memory is not currently mapped
#[no_mangle]
pub unsafe extern "C" fn vkFreeMemory(
    device: VkDevice,
    memory: VkDeviceMemory,
    pAllocator: *const VkAllocationCallbacks,
) {
    if device.is_null() || memory.is_null() {
        return;
    }
    
    if let Some(icd) = icd_loader::icd_for_device(device) {
        if let Some(f) = icd.free_memory { f(device, memory, pAllocator); }
        return;
    }
    if let Some(icd) = super::forward::get_icd_if_enabled() {
        if let Some(free_memory) = icd.free_memory { free_memory(device, memory, pAllocator); }
    }
}

/// Map memory for CPU access
// SAFETY: This function is called from C code. Caller must ensure:
// 1. device is a valid VkDevice
// 2. memory is a valid VkDeviceMemory with the VK_MEMORY_PROPERTY_HOST_VISIBLE_BIT
// 3. offset and size are within the allocated memory range
// 4. ppData points to valid memory for writing the mapped pointer
// 5. The memory is not already mapped
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
    
    if let Some(icd) = icd_loader::icd_for_device(device) {
        if let Some(f) = icd.map_memory { return f(device, memory, offset, size, flags, ppData); }
    }
    if let Some(icd) = super::forward::get_icd_if_enabled() {
        if let Some(map_memory) = icd.map_memory { return map_memory(device, memory, offset, size, flags, ppData); }
    }
    VkResult::ErrorInitializationFailed
}

/// Unmap memory
// SAFETY: This function is called from C code. Caller must ensure:
// 1. device is a valid VkDevice
// 2. memory is a valid VkDeviceMemory that is currently mapped
// 3. Any host writes to the mapped memory are complete
#[no_mangle]
pub unsafe extern "C" fn vkUnmapMemory(
    device: VkDevice,
    memory: VkDeviceMemory,
) {
    if device.is_null() || memory.is_null() {
        return;
    }
    
    if let Some(icd) = icd_loader::icd_for_device(device) {
        if let Some(f) = icd.unmap_memory { f(device, memory); }
        return;
    }
    if let Some(icd) = super::forward::get_icd_if_enabled() {
        if let Some(unmap_memory) = icd.unmap_memory { unmap_memory(device, memory); }
    }
}
