//! Instance creation and management

use std::sync::Arc;
use std::ffi::CStr;
use crate::sys::*;
use crate::core::*;
use crate::ffi::*;
use super::{LOADER, Instance, PhysicalDevice, PhysicalDeviceProperties};

/// Create a Kronos instance
#[no_mangle]
pub unsafe extern "C" fn vkCreateInstance(
    pCreateInfo: *const VkInstanceCreateInfo,
    pAllocator: *const VkAllocationCallbacks,
    pInstance: *mut VkInstance,
) -> VkResult {
    // Validate inputs
    if pCreateInfo.is_null() || pInstance.is_null() {
        return VkResult::ErrorInitializationFailed;
    }
    
    // Check backend mode
    let mode = super::BACKEND_MODE.lock().unwrap();
    match *mode {
        super::BackendMode::RealICD => {
            // Use real Vulkan driver
            if let Some(icd) = super::icd_loader::get_icd() {
                if let Some(create_instance_fn) = icd.create_instance {
                    let result = create_instance_fn(pCreateInfo, pAllocator, pInstance);
                    
                    // If successful, load instance functions
                    if result == VkResult::Success {
                        // Load instance functions in a separate scope
                        drop(mode);
                        let mut icd_mut = super::icd_loader::ICD_LOADER.lock().unwrap();
                        if let Some(icd) = icd_mut.as_mut() {
                            super::icd_loader::load_instance_functions(icd, *pInstance);
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
    
    // Validate structure type
    if create_info.sType != VkStructureType::InstanceCreateInfo {
        return VkResult::ErrorInitializationFailed;
    }
    
    // Extract application info safely
    let (app_name, engine_name) = if !create_info.pApplicationInfo.is_null() {
        let app_info = &*create_info.pApplicationInfo;
        
        let app_name = if !app_info.pApplicationName.is_null() {
            CStr::from_ptr(app_info.pApplicationName)
                .to_string_lossy()
                .into_owned()
        } else {
            String::new()
        };
        
        let engine_name = if !app_info.pEngineName.is_null() {
            CStr::from_ptr(app_info.pEngineName)
                .to_string_lossy()
                .into_owned()
        } else {
            String::new()
        };
        
        (Some(app_name), Some(engine_name))
    } else {
        (None, None)
    };
    
    // For compute-only, we ignore layers and extensions
    
    let mut loader = LOADER.lock().unwrap();
    let handle = VkInstance::from_raw(loader.allocate_handle());
    
    // Create instance object
    let instance = Instance {
        handle,
        app_name,
        engine_name,
        physical_devices: discover_physical_devices(),
    };
    
    loader.instances.insert(handle.as_raw(), Arc::new(instance));
    
    // Return handle
    *pInstance = handle;
    
    VkResult::Success
}

/// Destroy instance
#[no_mangle]
pub unsafe extern "C" fn vkDestroyInstance(
    instance: VkInstance,
    pAllocator: *const VkAllocationCallbacks,
) {
    if instance.is_null() {
        return;
    }
    
    let mut loader = LOADER.lock().unwrap();
    loader.instances.remove(&instance.as_raw());
}

/// Enumerate physical devices (GPUs)
#[no_mangle]
pub unsafe extern "C" fn vkEnumeratePhysicalDevices(
    instance: VkInstance,
    pPhysicalDeviceCount: *mut u32,
    pPhysicalDevices: *mut VkPhysicalDevice,
) -> VkResult {
    if instance.is_null() || pPhysicalDeviceCount.is_null() {
        return VkResult::ErrorInitializationFailed;
    }
    
    let loader = LOADER.lock().unwrap();
    let instance = match loader.instances.get(&instance.as_raw()) {
        Some(inst) => inst,
        None => return VkResult::ErrorInitializationFailed,
    };
    
    let count = instance.physical_devices.len() as u32;
    
    if pPhysicalDevices.is_null() {
        // Query count only
        *pPhysicalDeviceCount = count;
    } else {
        // Return devices
        let requested = *pPhysicalDeviceCount;
        let returned = requested.min(count);
        
        for i in 0..returned as usize {
            *pPhysicalDevices.add(i) = instance.physical_devices[i].handle;
        }
        
        *pPhysicalDeviceCount = returned;
        
        if returned < count {
            return VkResult::Incomplete;
        }
    }
    
    VkResult::Success
}

/// Discover available compute devices
fn discover_physical_devices() -> Vec<Arc<PhysicalDevice>> {
    let mut devices = Vec::new();
    
    // For now, create a mock compute device
    let mock_device = PhysicalDevice {
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
    };
    
    devices.push(Arc::new(mock_device));
    devices
}

pub(super) fn create_mock_properties() -> PhysicalDeviceProperties {
    PhysicalDeviceProperties {
        api_version: VK_API_VERSION_1_0,
        driver_version: 1,
        vendor_id: 0x10DE, // NVIDIA
        device_id: 0x2182, // RTX 3090
        device_type: VkPhysicalDeviceType::DiscreteGpu,
        device_name: "Kronos Compute Device (RTX 3090)".to_string(),
        pipeline_cache_uuid: [0; 16],
        limits: VkPhysicalDeviceLimits {
            maxComputeSharedMemorySize: 49152,
            maxComputeWorkGroupCount: [2147483647, 65535, 65535],
            maxComputeWorkGroupInvocations: 1024,
            maxComputeWorkGroupSize: [1024, 1024, 64],
        },
        sparse_properties: VkPhysicalDeviceSparseProperties {
            residencyStandard2DBlockShape: VK_TRUE,
            residencyStandard2DMultisampleBlockShape: VK_FALSE,
            residencyStandard3DBlockShape: VK_TRUE,
            residencyAlignedMipSize: VK_FALSE,
            residencyNonResidentStrict: VK_TRUE,
        },
    }
}

pub(super) fn create_mock_memory_properties() -> VkPhysicalDeviceMemoryProperties {
    let mut props = VkPhysicalDeviceMemoryProperties {
        memoryTypeCount: 3,
        memoryTypes: [VkMemoryType { 
            propertyFlags: VkMemoryPropertyFlags::empty(), 
            heapIndex: 0 
        }; 32],
        memoryHeapCount: 2,
        memoryHeaps: [VkMemoryHeap { size: 0, flags: 0 }; 16],
    };
    
    // Device local memory (GPU)
    props.memoryTypes[0] = VkMemoryType {
        propertyFlags: VkMemoryPropertyFlags::DEVICE_LOCAL,
        heapIndex: 0,
    };
    
    // Host visible + coherent (CPU accessible)
    props.memoryTypes[1] = VkMemoryType {
        propertyFlags: VkMemoryPropertyFlags::HOST_VISIBLE | VkMemoryPropertyFlags::HOST_COHERENT,
        heapIndex: 1,
    };
    
    // Host visible + cached (CPU cached)
    props.memoryTypes[2] = VkMemoryType {
        propertyFlags: VkMemoryPropertyFlags::HOST_VISIBLE | 
                       VkMemoryPropertyFlags::HOST_COHERENT | 
                       VkMemoryPropertyFlags::HOST_CACHED,
        heapIndex: 1,
    };
    
    // Memory heaps
    props.memoryHeaps[0] = VkMemoryHeap {
        size: 24 * 1024 * 1024 * 1024, // 24 GB device memory
        flags: VK_MEMORY_HEAP_DEVICE_LOCAL_BIT,
    };
    
    props.memoryHeaps[1] = VkMemoryHeap {
        size: 64 * 1024 * 1024 * 1024, // 64 GB system memory
        flags: 0,
    };
    
    props
}

/// Get physical device properties
#[no_mangle]
pub unsafe extern "C" fn vkGetPhysicalDeviceProperties(
    physicalDevice: VkPhysicalDevice,
    pProperties: *mut VkPhysicalDeviceProperties,
) {
    if physicalDevice.is_null() || pProperties.is_null() {
        return;
    }
    
    // Convert our thread-safe properties to C-compatible struct
    let props = create_mock_properties();
    let mut c_props = std::mem::zeroed::<VkPhysicalDeviceProperties>();
    
    c_props.apiVersion = props.api_version;
    c_props.driverVersion = props.driver_version;
    c_props.vendorID = props.vendor_id;
    c_props.deviceID = props.device_id;
    c_props.deviceType = props.device_type;
    c_props.limits = props.limits;
    c_props.sparseProperties = props.sparse_properties;
    
    // Copy device name
    let name_bytes = props.device_name.as_bytes();
    let copy_len = name_bytes.len().min(255);
    for i in 0..copy_len {
        c_props.deviceName[i] = name_bytes[i] as i8;
    }
    c_props.deviceName[copy_len] = 0; // Null terminate
    
    // Copy UUID
    c_props.pipelineCacheUUID = props.pipeline_cache_uuid;
    
    *pProperties = c_props;
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
    
    *pMemoryProperties = create_mock_memory_properties();
}

/// Get physical device queue family properties
#[no_mangle]
pub unsafe extern "C" fn vkGetPhysicalDeviceQueueFamilyProperties(
    physicalDevice: VkPhysicalDevice,
    pQueueFamilyPropertyCount: *mut u32,
    pQueueFamilyProperties: *mut VkQueueFamilyProperties,
) {
    if physicalDevice.is_null() || pQueueFamilyPropertyCount.is_null() {
        return;
    }
    
    // Single compute queue family
    if pQueueFamilyProperties.is_null() {
        *pQueueFamilyPropertyCount = 1;
    } else {
        if *pQueueFamilyPropertyCount >= 1 {
            *pQueueFamilyProperties = VkQueueFamilyProperties {
                queueFlags: VkQueueFlags::COMPUTE | VkQueueFlags::TRANSFER,
                queueCount: 8,
                timestampValidBits: 64,
                minImageTransferGranularity: VkExtent3D::default(),
            };
            *pQueueFamilyPropertyCount = 1;
        }
    }
}

// Add missing constant
const VK_MEMORY_HEAP_DEVICE_LOCAL_BIT: u32 = 0x00000001;