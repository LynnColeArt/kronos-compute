//! REAL Kronos device implementation - NO ICD forwarding!

use crate::sys::*;
use crate::core::*;
use crate::ffi::*;
use std::ptr;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Mutex;
use std::collections::HashMap;

// Device handle counter
static DEVICE_COUNTER: AtomicU64 = AtomicU64::new(1);

// Registry of active devices
lazy_static::lazy_static! {
    static ref DEVICES: Mutex<HashMap<u64, DeviceData>> = Mutex::new(HashMap::new());
}

struct DeviceData {
    physical_device: VkPhysicalDevice,
    queue_family_index: u32,
    queue: VkQueue,
}

/// Get physical device properties - return info about our virtual compute device
#[no_mangle]
pub unsafe extern "C" fn vkGetPhysicalDeviceProperties(
    physicalDevice: VkPhysicalDevice,
    pProperties: *mut VkPhysicalDeviceProperties,
) {
    if physicalDevice.is_null() || pProperties.is_null() {
        return;
    }
    
    let props = &mut *pProperties;
    
    // API version - we support Vulkan 1.3 compute
    props.apiVersion = VK_API_VERSION_1_3;
    props.driverVersion = VK_MAKE_VERSION(0, 2, 3);
    props.vendorID = 0x1337; // Custom vendor ID
    props.deviceID = 0x0001;
    props.deviceType = VkPhysicalDeviceType::VirtualGpu;
    
    // Device name
    let name = b"Kronos Virtual Compute Device\0";
    let len = name.len().min(VK_MAX_PHYSICAL_DEVICE_NAME_SIZE as usize);
    props.deviceName[..len].copy_from_slice(&name[..len].iter().map(|&b| b as i8).collect::<Vec<_>>());
    
    // Limits - set reasonable compute limits
    props.limits.maxComputeSharedMemorySize = 49152; // 48KB
    props.limits.maxComputeWorkGroupCount = [65536, 65536, 65536];
    props.limits.maxComputeWorkGroupInvocations = 1024;
    props.limits.maxComputeWorkGroupSize = [1024, 1024, 64];
    
    // Sparse properties - we don't support sparse
    props.sparseProperties = VkPhysicalDeviceSparseProperties {
        residencyStandard2DBlockShape: VK_FALSE,
        residencyStandard2DMultisampleBlockShape: VK_FALSE,
        residencyStandard3DBlockShape: VK_FALSE,
        residencyAlignedMipSize: VK_FALSE,
        residencyNonResidentStrict: VK_FALSE,
    };
}

/// Get queue family properties - we have one compute queue
#[no_mangle]
pub unsafe extern "C" fn vkGetPhysicalDeviceQueueFamilyProperties(
    physicalDevice: VkPhysicalDevice,
    pQueueFamilyPropertyCount: *mut u32,
    pQueueFamilyProperties: *mut VkQueueFamilyProperties,
) {
    if physicalDevice.is_null() || pQueueFamilyPropertyCount.is_null() {
        return;
    }
    
    if pQueueFamilyProperties.is_null() {
        *pQueueFamilyPropertyCount = 1;
        return;
    }
    
    let count = *pQueueFamilyPropertyCount;
    if count == 0 {
        return;
    }
    
    // We have one queue family that supports compute
    let props = &mut *pQueueFamilyProperties;
    props.queueFlags = VkQueueFlags::COMPUTE | VkQueueFlags::TRANSFER;
    props.queueCount = 1;
    props.timestampValidBits = 64;
    props.minImageTransferGranularity = VkExtent3D { width: 1, height: 1, depth: 1 };
    
    *pQueueFamilyPropertyCount = 1;
}

/// Create logical device
#[no_mangle]
pub unsafe extern "C" fn vkCreateDevice(
    physicalDevice: VkPhysicalDevice,
    pCreateInfo: *const VkDeviceCreateInfo,
    _pAllocator: *const VkAllocationCallbacks,
    pDevice: *mut VkDevice,
) -> VkResult {
    if physicalDevice.is_null() || pCreateInfo.is_null() || pDevice.is_null() {
        return VkResult::ErrorInitializationFailed;
    }
    
    let create_info = &*pCreateInfo;
    
    // Verify queue creation
    if create_info.queueCreateInfoCount == 0 {
        return VkResult::ErrorInitializationFailed;
    }
    
    // Get first queue family (should be compute)
    let queue_info = &*create_info.pQueueCreateInfos;
    let queue_family_index = queue_info.queueFamilyIndex;
    
    // Create device handle
    let device_handle = DEVICE_COUNTER.fetch_add(1, Ordering::SeqCst);
    let queue_handle = DEVICE_COUNTER.fetch_add(1, Ordering::SeqCst);
    
    // Store device data
    let device_data = DeviceData {
        physical_device: physicalDevice,
        queue_family_index,
        queue: VkQueue::from_raw(queue_handle),
    };
    
    DEVICES.lock().unwrap().insert(device_handle, device_data);
    
    *pDevice = VkDevice::from_raw(device_handle);
    
    log::info!("Created Kronos device {:?} - pure Rust compute implementation", device_handle);
    
    VkResult::Success
}

/// Get device queue
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
    
    if queueIndex != 0 {
        return; // We only have one queue
    }
    
    let handle = device.as_raw();
    if let Some(device_data) = DEVICES.lock().unwrap().get(&handle) {
        if queueFamilyIndex == device_data.queue_family_index {
            *pQueue = device_data.queue;
        }
    }
}

/// Get physical device memory properties
#[no_mangle]
pub unsafe extern "C" fn vkGetPhysicalDeviceMemoryProperties(
    physicalDevice: VkPhysicalDevice,
    pMemoryProperties: *mut VkPhysicalDeviceMemoryProperties,
) {
    if physicalDevice.is_null() || pMemoryProperties.is_null() {
        return;
    }
    
    let props = &mut *pMemoryProperties;
    
    // We support one memory type - host visible and coherent
    props.memoryTypeCount = 1;
    props.memoryTypes[0] = VkMemoryType {
        propertyFlags: VkMemoryPropertyFlags::HOST_VISIBLE | VkMemoryPropertyFlags::HOST_COHERENT,
        heapIndex: 0,
    };
    
    // One memory heap - 16GB virtual memory
    props.memoryHeapCount = 1;
    props.memoryHeaps[0] = VkMemoryHeap {
        size: 16 * 1024 * 1024 * 1024, // 16GB
        flags: 0, // No specific heap flags
    };
}

/// Destroy device
#[no_mangle]
pub unsafe extern "C" fn vkDestroyDevice(
    device: VkDevice,
    _pAllocator: *const VkAllocationCallbacks,
) {
    if device.is_null() {
        return;
    }
    
    let handle = device.as_raw();
    DEVICES.lock().unwrap().remove(&handle);
    
    log::info!("Destroyed Kronos device {:?}", handle);
}