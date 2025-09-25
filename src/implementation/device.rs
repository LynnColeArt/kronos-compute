//! Device creation and queue management

use crate::sys::*;
use crate::core::*;
use crate::ffi::*;
use crate::implementation::icd_loader;

/// Create a logical device
// SAFETY: This function is called from C code. Caller must ensure:
// 1. physicalDevice is a valid VkPhysicalDevice from vkEnumeratePhysicalDevices
// 2. pCreateInfo points to a valid VkDeviceCreateInfo structure
// 3. pAllocator is either null or points to valid allocation callbacks
// 4. pDevice points to valid memory for writing the device handle
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

    // Aggregated-aware: prefer ICD owning the physical device
    if let Some(icd_arc) = icd_loader::icd_for_physical_device(physicalDevice) {
        if let Some(create_device_fn) = icd_arc.create_device {
            let result = create_device_fn(physicalDevice, pCreateInfo, pAllocator, pDevice);
            if result == VkResult::Success {
                log::info!("Device creation successful for physical device {:?}, new device: {:?}", physicalDevice, *pDevice);
                // Load device-level functions into a cloned ICD and register device → ICD mapping
                let mut cloned = (*icd_arc).clone();
                match icd_loader::load_device_functions_inner(&mut cloned, *pDevice) {
                    Ok(()) => {
                        log::info!("Successfully loaded device functions for device {:?}", *pDevice);
                        // Check if create_buffer was loaded
                        if cloned.create_buffer.is_some() {
                            log::info!("create_buffer function loaded successfully");
                        } else {
                            log::warn!("create_buffer function NOT loaded!");
                        }
                    }
                    Err(e) => {
                        log::error!("Failed to load device functions: {:?}", e);
                    }
                }
                let updated = std::sync::Arc::new(cloned);
                icd_loader::register_device_icd(*pDevice, &updated);
                log::info!("Registered device {:?} with ICD", *pDevice);
            }
            return result;
        }
    }

    // Fallback to single-ICD driver
    if let Some(icd) = super::forward::get_icd_if_enabled() {
        if let Some(create_device_fn) = icd.create_device {
            let result = create_device_fn(physicalDevice, pCreateInfo, pAllocator, pDevice);
            if result == VkResult::Success {
                let _ = super::icd_loader::update_device_functions(*pDevice);
            }
            return result;
        }
    }

    VkResult::ErrorInitializationFailed
}

/// Destroy a logical device
// SAFETY: This function is called from C code. Caller must ensure:
// 1. device is a valid VkDevice created by vkCreateDevice
// 2. pAllocator matches the allocator used in vkCreateDevice (or both are null)
// 3. All objects created from this device have been destroyed
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

    // Unregister device from provenance registry (aggregated mode)
    crate::implementation::icd_loader::unregister_device(device);
}

/// Get a device queue
// SAFETY: This function is called from C code. Caller must ensure:
// 1. device is a valid VkDevice
// 2. queueFamilyIndex and queueIndex are valid for this device
// 3. pQueue points to valid memory for writing the queue handle
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

    // Route via owning ICD if known
    if let Some(icd) = icd_loader::icd_for_device(device) {
        if let Some(f) = icd.get_device_queue {
            f(device, queueFamilyIndex, queueIndex, pQueue);
            if let Some(queue) = pQueue.as_ref() {
                // Register queue → ICD mapping
                icd_loader::register_queue_icd(unsafe { *queue }, &icd);
            }
            return;
        }
    }
    // Fallback
    if let Some(icd) = super::forward::get_icd_if_enabled() {
        if let Some(get_device_queue) = icd.get_device_queue {
            get_device_queue(device, queueFamilyIndex, queueIndex, pQueue);
        }
    }
}

/// Submit work to a queue
// SAFETY: This function is called from C code. Caller must ensure:
// 1. queue is a valid VkQueue obtained from vkGetDeviceQueue
// 2. If submitCount > 0, pSubmits points to an array of valid VkSubmitInfo structures
// 3. fence is either VK_NULL_HANDLE or a valid VkFence
// 4. All command buffers, semaphores, and other resources referenced are valid
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

    // Route via queue owner if known
    if let Some(icd) = icd_loader::icd_for_queue(queue) {
        if let Some(f) = icd.queue_submit { return f(queue, submitCount, pSubmits, fence); }
    }
    // Fallback
    if let Some(icd) = super::forward::get_icd_if_enabled() {
        if let Some(f) = icd.queue_submit { return f(queue, submitCount, pSubmits, fence); }
    }
    VkResult::ErrorInitializationFailed
}

/// Wait for queue to become idle
#[no_mangle]
pub unsafe extern "C" fn vkQueueWaitIdle(queue: VkQueue) -> VkResult {
    if queue.is_null() {
        return VkResult::ErrorDeviceLost;
    }

    if let Some(icd) = icd_loader::icd_for_queue(queue) {
        if let Some(f) = icd.queue_wait_idle { return f(queue); }
    }
    if let Some(icd) = super::forward::get_icd_if_enabled() {
        if let Some(f) = icd.queue_wait_idle { return f(queue); }
    }
    VkResult::ErrorInitializationFailed
}

/// Wait for device to become idle
#[no_mangle]
pub unsafe extern "C" fn vkDeviceWaitIdle(device: VkDevice) -> VkResult {
    if device.is_null() {
        return VkResult::ErrorDeviceLost;
    }

    if let Some(icd) = icd_loader::icd_for_device(device) {
        if let Some(f) = icd.device_wait_idle { return f(device); }
    }
    if let Some(icd) = super::forward::get_icd_if_enabled() {
        if let Some(f) = icd.device_wait_idle { return f(device); }
    }
    VkResult::ErrorInitializationFailed
}
