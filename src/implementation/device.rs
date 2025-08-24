//! Device creation and queue management

use crate::sys::*;
use crate::core::*;
use crate::ffi::*;

/// Create a logical device
#[no_mangle]
pub unsafe extern "C" fn vkCreateDevice(
    physicalDevice: VkPhysicalDevice,
    pCreateInfo: *const VkDeviceCreateInfo,
    pAllocator: *const VkAllocationCallbacks,
    pDevice: *mut VkDevice,
) -> VkResult {
    if physicalDevice.is_null() || pCreateInfo.is_null() || pDevice.is_null() {
        return VkResult::ErrorInitializationFailed;
    }
    
    // Try to forward to real driver
    if let Some(icd) = super::icd_loader::get_icd() {
        if let Some(create_device_fn) = icd.create_device {
            let result = create_device_fn(physicalDevice, pCreateInfo, pAllocator, pDevice);
            
            // If successful, load device functions
            if result == VkResult::Success {
                let mut icd_mut = super::icd_loader::ICD_LOADER.lock().unwrap();
                if let Some(icd) = icd_mut.as_mut() {
                    super::icd_loader::load_device_functions(icd, *pDevice);
                }
            }
            
            return result;
        }
    }
    
    // No ICD available
    VkResult::ErrorInitializationFailed
}

/// Destroy a logical device
#[no_mangle]
pub unsafe extern "C" fn vkDestroyDevice(
    device: VkDevice,
    pAllocator: *const VkAllocationCallbacks,
) {
    if device.is_null() {
        return;
    }
    
    // Forward to real ICD
    if let Some(icd) = super::forward::get_icd_if_enabled() {
        if let Some(destroy_device) = icd.destroy_device {
            destroy_device(device, pAllocator);
        }
    }
}

/// Get a device queue
#[no_mangle]
pub unsafe extern "C" fn vkGetDeviceQueue(
    device: VkDevice,
    queueFamilyIndex: u32,
    queueIndex: u32,
    pQueue: *mut VkQueue,
) {
    if device.is_null() || pQueue.is_null() {
        return;
    }
    
    // Forward to real ICD
    if let Some(icd) = super::forward::get_icd_if_enabled() {
        if let Some(get_device_queue) = icd.get_device_queue {
            get_device_queue(device, queueFamilyIndex, queueIndex, pQueue);
        }
    }
}

/// Submit work to a queue
#[no_mangle]
pub unsafe extern "C" fn vkQueueSubmit(
    queue: VkQueue,
    submitCount: u32,
    pSubmits: *const VkSubmitInfo,
    fence: VkFence,
) -> VkResult {
    if queue.is_null() {
        return VkResult::ErrorDeviceLost;
    }
    
    // Forward to real driver
    if let Some(icd) = super::forward::get_icd_if_enabled() {
        if let Some(queue_submit) = icd.queue_submit {
            return queue_submit(queue, submitCount, pSubmits, fence);
        }
    }
    
    // No ICD available
    VkResult::ErrorInitializationFailed
}

/// Wait for queue to become idle
#[no_mangle]
pub unsafe extern "C" fn vkQueueWaitIdle(queue: VkQueue) -> VkResult {
    if queue.is_null() {
        return VkResult::ErrorDeviceLost;
    }
    
    // Forward to real driver
    if let Some(icd) = super::forward::get_icd_if_enabled() {
        if let Some(queue_wait_idle) = icd.queue_wait_idle {
            return queue_wait_idle(queue);
        }
    }
    
    // No ICD available
    VkResult::ErrorInitializationFailed
}

/// Wait for device to become idle
#[no_mangle]
pub unsafe extern "C" fn vkDeviceWaitIdle(device: VkDevice) -> VkResult {
    if device.is_null() {
        return VkResult::ErrorDeviceLost;
    }
    
    // Forward to real driver
    if let Some(icd) = super::forward::get_icd_if_enabled() {
        if let Some(device_wait_idle) = icd.device_wait_idle {
            return device_wait_idle(device);
        }
    }
    
    // No ICD available
    VkResult::ErrorInitializationFailed
}

