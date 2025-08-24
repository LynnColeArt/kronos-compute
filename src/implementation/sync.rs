//! Synchronization primitives implementation
//! 
//! Implements fences, semaphores, and events for GPU synchronization

use crate::sys::*;
use crate::core::*;
use crate::ffi::*;

/// Create a fence
#[no_mangle]
pub unsafe extern "C" fn vkCreateFence(
    device: VkDevice,
    pCreateInfo: *const VkFenceCreateInfo,
    pAllocator: *const VkAllocationCallbacks,
    pFence: *mut VkFence,
) -> VkResult {
    if device.is_null() || pCreateInfo.is_null() || pFence.is_null() {
        return VkResult::ErrorInitializationFailed;
    }
    
    // Forward to real ICD
    if let Some(icd) = super::forward::get_icd_if_enabled() {
        if let Some(create_fence) = icd.create_fence {
            return create_fence(device, pCreateInfo, pAllocator, pFence);
        }
    }
    
    // No ICD available
    VkResult::ErrorInitializationFailed
}

/// Destroy a fence
#[no_mangle]
pub unsafe extern "C" fn vkDestroyFence(
    device: VkDevice,
    fence: VkFence,
    pAllocator: *const VkAllocationCallbacks,
) {
    if device.is_null() || fence.is_null() {
        return;
    }
    
    // Forward to real ICD
    if let Some(icd) = super::forward::get_icd_if_enabled() {
        if let Some(destroy_fence) = icd.destroy_fence {
            destroy_fence(device, fence, pAllocator);
        }
    }
}

/// Reset fences
#[no_mangle]
pub unsafe extern "C" fn vkResetFences(
    device: VkDevice,
    fenceCount: u32,
    pFences: *const VkFence,
) -> VkResult {
    if device.is_null() || fenceCount == 0 || pFences.is_null() {
        return VkResult::ErrorInitializationFailed;
    }
    
    // Forward to real ICD
    if let Some(icd) = super::forward::get_icd_if_enabled() {
        if let Some(reset_fences) = icd.reset_fences {
            return reset_fences(device, fenceCount, pFences);
        }
    }
    
    // No ICD available
    VkResult::ErrorInitializationFailed
}

/// Get fence status
#[no_mangle]
pub unsafe extern "C" fn vkGetFenceStatus(
    device: VkDevice,
    fence: VkFence,
) -> VkResult {
    if device.is_null() || fence.is_null() {
        return VkResult::ErrorDeviceLost;
    }
    
    // Forward to real ICD
    if let Some(icd) = super::forward::get_icd_if_enabled() {
        if let Some(get_fence_status) = icd.get_fence_status {
            return get_fence_status(device, fence);
        }
    }
    
    // No ICD available
    VkResult::ErrorInitializationFailed
}

/// Wait for fences
#[no_mangle]
pub unsafe extern "C" fn vkWaitForFences(
    device: VkDevice,
    fenceCount: u32,
    pFences: *const VkFence,
    waitAll: VkBool32,
    timeout: u64,
) -> VkResult {
    if device.is_null() || fenceCount == 0 || pFences.is_null() {
        return VkResult::ErrorInitializationFailed;
    }
    
    // Forward to real ICD
    if let Some(icd) = super::forward::get_icd_if_enabled() {
        if let Some(wait_for_fences) = icd.wait_for_fences {
            return wait_for_fences(device, fenceCount, pFences, waitAll, timeout);
        }
    }
    
    // No ICD available
    VkResult::ErrorInitializationFailed
}

/// Create a semaphore
#[no_mangle]
pub unsafe extern "C" fn vkCreateSemaphore(
    device: VkDevice,
    pCreateInfo: *const VkSemaphoreCreateInfo,
    pAllocator: *const VkAllocationCallbacks,
    pSemaphore: *mut VkSemaphore,
) -> VkResult {
    if device.is_null() || pCreateInfo.is_null() || pSemaphore.is_null() {
        return VkResult::ErrorInitializationFailed;
    }
    
    // Forward to real ICD
    if let Some(icd) = super::forward::get_icd_if_enabled() {
        if let Some(create_semaphore) = icd.create_semaphore {
            return create_semaphore(device, pCreateInfo, pAllocator, pSemaphore);
        }
    }
    
    // No ICD available
    VkResult::ErrorInitializationFailed
}

/// Destroy a semaphore
#[no_mangle]
pub unsafe extern "C" fn vkDestroySemaphore(
    device: VkDevice,
    semaphore: VkSemaphore,
    pAllocator: *const VkAllocationCallbacks,
) {
    if device.is_null() || semaphore.is_null() {
        return;
    }
    
    // Forward to real ICD
    if let Some(icd) = super::forward::get_icd_if_enabled() {
        if let Some(destroy_semaphore) = icd.destroy_semaphore {
            destroy_semaphore(device, semaphore, pAllocator);
        }
    }
}

/// Create an event
#[no_mangle]
pub unsafe extern "C" fn vkCreateEvent(
    device: VkDevice,
    pCreateInfo: *const VkEventCreateInfo,
    pAllocator: *const VkAllocationCallbacks,
    pEvent: *mut VkEvent,
) -> VkResult {
    if device.is_null() || pCreateInfo.is_null() || pEvent.is_null() {
        return VkResult::ErrorInitializationFailed;
    }
    
    // Forward to real ICD
    if let Some(icd) = super::forward::get_icd_if_enabled() {
        if let Some(create_event) = icd.create_event {
            return create_event(device, pCreateInfo, pAllocator, pEvent);
        }
    }
    
    // No ICD available
    VkResult::ErrorInitializationFailed
}

/// Destroy an event
#[no_mangle]
pub unsafe extern "C" fn vkDestroyEvent(
    device: VkDevice,
    event: VkEvent,
    pAllocator: *const VkAllocationCallbacks,
) {
    if device.is_null() || event.is_null() {
        return;
    }
    
    // Forward to real ICD
    if let Some(icd) = super::forward::get_icd_if_enabled() {
        if let Some(destroy_event) = icd.destroy_event {
            destroy_event(device, event, pAllocator);
        }
    }
}

/// Get event status
#[no_mangle]
pub unsafe extern "C" fn vkGetEventStatus(
    device: VkDevice,
    event: VkEvent,
) -> VkResult {
    if device.is_null() || event.is_null() {
        return VkResult::ErrorDeviceLost;
    }
    
    // Forward to real ICD
    if let Some(icd) = super::forward::get_icd_if_enabled() {
        if let Some(get_event_status) = icd.get_event_status {
            return get_event_status(device, event);
        }
    }
    
    // No ICD available
    VkResult::ErrorInitializationFailed
}

/// Set event
#[no_mangle]
pub unsafe extern "C" fn vkSetEvent(
    device: VkDevice,
    event: VkEvent,
) -> VkResult {
    if device.is_null() || event.is_null() {
        return VkResult::ErrorDeviceLost;
    }
    
    // Forward to real ICD
    if let Some(icd) = super::forward::get_icd_if_enabled() {
        if let Some(set_event) = icd.set_event {
            return set_event(device, event);
        }
    }
    
    // No ICD available
    VkResult::ErrorInitializationFailed
}

/// Reset event
#[no_mangle]
pub unsafe extern "C" fn vkResetEvent(
    device: VkDevice,
    event: VkEvent,
) -> VkResult {
    if device.is_null() || event.is_null() {
        return VkResult::ErrorDeviceLost;
    }
    
    // Forward to real ICD
    if let Some(icd) = super::forward::get_icd_if_enabled() {
        if let Some(reset_event) = icd.reset_event {
            return reset_event(device, event);
        }
    }
    
    // No ICD available
    VkResult::ErrorInitializationFailed
}