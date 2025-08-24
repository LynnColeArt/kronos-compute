//! Device creation and queue management

use std::sync::{Arc, Mutex};
use std::collections::HashMap;
use crate::sys::*;
use crate::core::*;
use crate::ffi::*;
use super::{LOADER, Device, PhysicalDevice};

lazy_static::lazy_static! {
    pub(super) static ref DEVICES: Mutex<HashMap<u64, Arc<Mutex<Device>>>> = Mutex::new(HashMap::new());
}

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
    
    // Check if we should forward to real driver
    let mode = super::BACKEND_MODE.lock().unwrap();
    match *mode {
        super::BackendMode::RealICD => {
            if let Some(icd) = super::icd_loader::get_icd() {
                if let Some(create_device_fn) = icd.create_device {
                    let result = create_device_fn(physicalDevice, pCreateInfo, pAllocator, pDevice);
                    
                    // If successful, load device functions
                    if result == VkResult::Success {
                        drop(mode);
                        let mut icd_mut = super::icd_loader::ICD_LOADER.lock().unwrap();
                        if let Some(icd) = icd_mut.as_mut() {
                            super::icd_loader::load_device_functions(icd, *pDevice);
                        }
                    }
                    
                    return result;
                }
            }
            return VkResult::ErrorInitializationFailed;
        }
        super::BackendMode::Mock => {
            // Continue with mock implementation
        }
    }
    drop(mode);
    
    let create_info = &*pCreateInfo;
    
    // Validate structure
    if create_info.sType != VkStructureType::DeviceCreateInfo {
        return VkResult::ErrorInitializationFailed;
    }
    
    // For compute-only, we ignore:
    // - Graphics-specific extensions
    // - Graphics-specific features
    // - Layers (in production we'd validate these)
    
    // Generate device handle
    let mut loader = LOADER.lock().unwrap();
    let handle = VkDevice::from_raw(loader.allocate_handle());
    
    // Create device
    let device = Device {
        handle,
        physical_device: Arc::new(create_mock_physical_device()),
        queues: HashMap::new(),
        memory_allocations: HashMap::new(),
        buffers: HashMap::new(),
    };
    
    // Create queues based on create info
    let mut device = Arc::new(Mutex::new(device));
    
    // Initialize queues
    for i in 0..create_info.queueCreateInfoCount {
        let queue_info = &*create_info.pQueueCreateInfos.add(i as usize);
        
        if queue_info.sType != VkStructureType::DeviceQueueCreateInfo {
            continue;
        }
        
        let family_index = queue_info.queueFamilyIndex;
        let queue_count = queue_info.queueCount;
        
        // Create queue handles
        let mut device_locked = device.lock().unwrap();
        for queue_index in 0..queue_count {
            let queue_handle = VkQueue::from_raw(loader.allocate_handle());
            device_locked.queues.insert((family_index, queue_index), queue_handle);
        }
    }
    
    // Store device
    DEVICES.lock().unwrap().insert(handle.as_raw(), device);
    
    *pDevice = handle;
    VkResult::Success
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
    
    // Remove from global device list
    DEVICES.lock().unwrap().remove(&device.as_raw());
    
    // In a real implementation:
    // - Wait for all queues to idle
    // - Free all device resources
    // - Notify driver
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
    
    let devices = DEVICES.lock().unwrap();
    if let Some(device_arc) = devices.get(&device.as_raw()) {
        let device_locked = device_arc.lock().unwrap();
        if let Some(&queue) = device_locked.queues.get(&(queueFamilyIndex, queueIndex)) {
            *pQueue = queue;
        } else {
            *pQueue = VkQueue::NULL;
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
    
    // Forward to real driver if enabled
    if let Some(icd) = super::forward::get_icd_if_enabled() {
        if let Some(queue_submit) = icd.queue_submit {
            return queue_submit(queue, submitCount, pSubmits, fence);
        }
    }
    
    // In a real implementation:
    // 1. Validate submit info
    // 2. Build command stream
    // 3. Submit to GPU
    // 4. Handle synchronization
    
    // Process each submit
    for i in 0..submitCount {
        let submit = &*pSubmits.add(i as usize);
        
        if submit.sType != VkStructureType::SubmitInfo {
            return VkResult::ErrorInitializationFailed;
        }
        
        // Wait on wait semaphores
        if submit.waitSemaphoreCount > 0 && !submit.pWaitSemaphores.is_null() {
            let wait_semaphores = std::slice::from_raw_parts(
                submit.pWaitSemaphores,
                submit.waitSemaphoreCount as usize
            );
            for &semaphore in wait_semaphores {
                super::sync::wait_and_reset_semaphore(semaphore);
            }
        }
        
        // Process each command buffer
        for j in 0..submit.commandBufferCount {
            let _cmd_buffer = *submit.pCommandBuffers.add(j as usize);
            // In real implementation: execute commands
        }
        
        // Signal signal semaphores
        if submit.signalSemaphoreCount > 0 && !submit.pSignalSemaphores.is_null() {
            let signal_semaphores = std::slice::from_raw_parts(
                submit.pSignalSemaphores,
                submit.signalSemaphoreCount as usize
            );
            for &semaphore in signal_semaphores {
                super::sync::signal_semaphore(semaphore);
            }
        }
    }
    
    // Signal fence if provided
    if !fence.is_null() {
        super::sync::signal_fence(fence);
    }
    
    VkResult::Success
}

/// Wait for queue to become idle
#[no_mangle]
pub unsafe extern "C" fn vkQueueWaitIdle(queue: VkQueue) -> VkResult {
    if queue.is_null() {
        return VkResult::ErrorDeviceLost;
    }
    
    // In a real implementation:
    // - Wait for all submitted work to complete
    // - Block until GPU is idle
    
    // For now, immediate return (simulating no pending work)
    VkResult::Success
}

/// Wait for device to become idle
#[no_mangle]
pub unsafe extern "C" fn vkDeviceWaitIdle(device: VkDevice) -> VkResult {
    if device.is_null() {
        return VkResult::ErrorDeviceLost;
    }
    
    let devices = DEVICES.lock().unwrap();
    if let Some(device_arc) = devices.get(&device.as_raw()) {
        let device_locked = device_arc.lock().unwrap();
        
        // Wait for all queues
        for (_, &queue) in &device_locked.queues {
            let result = vkQueueWaitIdle(queue);
            if result != VkResult::Success {
                return result;
            }
        }
    }
    
    VkResult::Success
}

// Helper to create mock physical device
fn create_mock_physical_device() -> PhysicalDevice {
    use super::instance::{create_mock_properties, create_mock_memory_properties};
    
    PhysicalDevice {
        handle: VkPhysicalDevice::from_raw(1),
        properties: create_mock_properties(),
        memory_properties: create_mock_memory_properties(),
        queue_families: vec![
            VkQueueFamilyProperties {
                queueFlags: VkQueueFlags::COMPUTE | VkQueueFlags::TRANSFER,
                queueCount: 8,
                timestampValidBits: 64,
                minImageTransferGranularity: VkExtent3D::default(),
            }
        ],
    }
}