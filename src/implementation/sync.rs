//! Synchronization primitives implementation
//! 
//! Implements fences, semaphores, and events for GPU synchronization

use crate::sys::*;
use crate::core::*;
use crate::ffi::*;

/// Create a fence
// SAFETY: This function is called from C code. Caller must ensure:
// 1. device is a valid VkDevice
// 2. pCreateInfo points to a valid VkFenceCreateInfo structure
// 3. pAllocator is either null or points to valid allocation callbacks
// 4. pFence points to valid memory for writing the fence handle
// 5. Fences are thread-safe and can be used across multiple threads
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
// SAFETY: This function is called from C code. Caller must ensure:
// 1. device is a valid VkDevice
// 2. fence is a valid VkFence created by vkCreateFence, or VK_NULL_HANDLE
// 3. pAllocator matches the allocator used in vkCreateFence (or both are null)
// 4. The fence is not currently being waited on by any thread
// 5. No queue operations are pending on this fence
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
// SAFETY: This function is called from C code. Caller must ensure:
// 1. device is a valid VkDevice
// 2. fenceCount is > 0 and matches the number of fences in pFences array
// 3. pFences points to an array of fenceCount valid VkFence handles
// 4. All fences are in the signaled state (cannot reset unsignaled fences)
// 5. No threads are currently waiting on these fences
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
// SAFETY: This function is called from C code. Caller must ensure:
// 1. device is a valid VkDevice
// 2. fence is a valid VkFence created by vkCreateFence
// 3. This function is thread-safe and can be called concurrently
// 4. The fence has not been destroyed
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
// SAFETY: This function is called from C code. Caller must ensure:
// 1. device is a valid VkDevice
// 2. fenceCount is > 0 and matches the number of fences in pFences array
// 3. pFences points to an array of fenceCount valid VkFence handles
// 4. waitAll is either VK_TRUE or VK_FALSE
// 5. timeout value is valid (can be UINT64_MAX for infinite wait)
// 6. This function may block the calling thread until timeout or fence signaling
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
// SAFETY: This function is called from C code. Caller must ensure:
// 1. device is a valid VkDevice
// 2. pCreateInfo points to a valid VkSemaphoreCreateInfo structure
// 3. pAllocator is either null or points to valid allocation callbacks
// 4. pSemaphore points to valid memory for writing the semaphore handle
// 5. Semaphores are used for GPU-GPU synchronization and queue ordering
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
// SAFETY: This function is called from C code. Caller must ensure:
// 1. device is a valid VkDevice
// 2. semaphore is a valid VkSemaphore created by vkCreateSemaphore, or VK_NULL_HANDLE
// 3. pAllocator matches the allocator used in vkCreateSemaphore (or both are null)
// 4. The semaphore is not pending in any queue operation
// 5. No command buffers reference this semaphore in wait or signal operations
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
// SAFETY: This function is called from C code. Caller must ensure:
// 1. device is a valid VkDevice
// 2. pCreateInfo points to a valid VkEventCreateInfo structure
// 3. pAllocator is either null or points to valid allocation callbacks
// 4. pEvent points to valid memory for writing the event handle
// 5. Events provide fine-grained synchronization within command buffers
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
// SAFETY: This function is called from C code. Caller must ensure:
// 1. device is a valid VkDevice
// 2. event is a valid VkEvent created by vkCreateEvent, or VK_NULL_HANDLE
// 3. pAllocator matches the allocator used in vkCreateEvent (or both are null)
// 4. No command buffers are currently waiting on or setting this event
// 5. All command buffers that reference this event have completed execution
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
// SAFETY: This function is called from C code. Caller must ensure:
// 1. device is a valid VkDevice
// 2. event is a valid VkEvent created by vkCreateEvent
// 3. This function can be called from the host to check event signaling state
// 4. The event has not been destroyed
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
// SAFETY: This function is called from C code. Caller must ensure:
// 1. device is a valid VkDevice
// 2. event is a valid VkEvent created by vkCreateEvent
// 3. Setting an event from the host signals all command buffers waiting on it
// 4. The event has not been destroyed
// 5. This can cause command buffers to proceed past vkCmdWaitEvents
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
// SAFETY: This function is called from C code. Caller must ensure:
// 1. device is a valid VkDevice
// 2. event is a valid VkEvent created by vkCreateEvent
// 3. Resetting an event puts it back into the unsignaled state
// 4. The event has not been destroyed
// 5. No command buffers should be waiting on this event when reset
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