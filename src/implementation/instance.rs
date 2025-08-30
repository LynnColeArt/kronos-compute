//! Instance creation and management

use crate::sys::*;
use crate::core::*;
use crate::ffi::*;

/// Create a Kronos instance
// SAFETY: This function is called from C code. Caller must ensure:
// 1. pCreateInfo points to a valid VkInstanceCreateInfo structure
// 2. pAllocator is either null or points to valid allocation callbacks
// 3. pInstance points to valid memory for writing the instance handle
// 4. All pointers remain valid for the duration of this call
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
    
    // Try to use real Vulkan driver
    if let Some(icd) = super::icd_loader::get_icd() {
        if let Some(create_instance_fn) = icd.create_instance {
            let result = create_instance_fn(pCreateInfo, pAllocator, pInstance);
            
            // If successful, load instance functions
            if result == VkResult::Success {
                let _ = super::icd_loader::update_instance_functions(*pInstance);
            }
            
            return result;
        }
    }
    
    // No ICD available
    VkResult::ErrorInitializationFailed
}

/// Destroy instance
// SAFETY: This function is called from C code. Caller must ensure:
// 1. instance is a valid VkInstance created by vkCreateInstance
// 2. pAllocator matches the allocator used in vkCreateInstance (or both are null)
// 3. All objects created from this instance have been destroyed
#[no_mangle]
pub unsafe extern "C" fn vkDestroyInstance(
    instance: VkInstance,
    pAllocator: *const VkAllocationCallbacks,
) {
    if instance.is_null() {
        return;
    }
    
    // Forward to real ICD if available
    if let Some(icd) = super::forward::get_icd_if_enabled() {
        if let Some(destroy_instance) = icd.destroy_instance {
            destroy_instance(instance, pAllocator);
        }
    }
}

/// Enumerate physical devices (GPUs)
// SAFETY: This function is called from C code. Caller must ensure:
// 1. instance is a valid VkInstance
// 2. pPhysicalDeviceCount points to valid memory
// 3. If pPhysicalDevices is not null, it points to an array of at least *pPhysicalDeviceCount elements
#[no_mangle]
pub unsafe extern "C" fn vkEnumeratePhysicalDevices(
    instance: VkInstance,
    pPhysicalDeviceCount: *mut u32,
    pPhysicalDevices: *mut VkPhysicalDevice,
) -> VkResult {
    if instance.is_null() || pPhysicalDeviceCount.is_null() {
        return VkResult::ErrorInitializationFailed;
    }
    
    // Forward to real ICD
    if let Some(icd) = super::forward::get_icd_if_enabled() {
        if let Some(enumerate_physical_devices) = icd.enumerate_physical_devices {
            return enumerate_physical_devices(instance, pPhysicalDeviceCount, pPhysicalDevices);
        } else {
            log::warn!("ICD loaded but enumerate_physical_devices function pointer is null");
        }
    } else {
        log::warn!("No ICD available for enumerate_physical_devices");
    }
    
    // No ICD available
    VkResult::ErrorInitializationFailed
}

/// Get physical device properties
// SAFETY: This function is called from C code. Caller must ensure:
// 1. physicalDevice is a valid VkPhysicalDevice obtained from vkEnumeratePhysicalDevices
// 2. pProperties points to valid memory for a VkPhysicalDeviceProperties structure
#[no_mangle]
pub unsafe extern "C" fn vkGetPhysicalDeviceProperties(
    physicalDevice: VkPhysicalDevice,
    pProperties: *mut VkPhysicalDeviceProperties,
) {
    if physicalDevice.is_null() || pProperties.is_null() {
        return;
    }
    
    // Forward to real ICD
    if let Some(icd) = super::forward::get_icd_if_enabled() {
        if let Some(get_physical_device_properties) = icd.get_physical_device_properties {
            get_physical_device_properties(physicalDevice, pProperties);
        }
    }
}

/// Get physical device memory properties
// SAFETY: This function is called from C code. Caller must ensure:
// 1. physicalDevice is a valid VkPhysicalDevice obtained from vkEnumeratePhysicalDevices
// 2. pMemoryProperties points to valid memory for a VkPhysicalDeviceMemoryProperties structure
#[no_mangle]
pub unsafe extern "C" fn vkGetPhysicalDeviceMemoryProperties(
    physicalDevice: VkPhysicalDevice,
    pMemoryProperties: *mut VkPhysicalDeviceMemoryProperties,
) {
    if physicalDevice.is_null() || pMemoryProperties.is_null() {
        return;
    }
    
    // Forward to real ICD
    if let Some(icd) = super::forward::get_icd_if_enabled() {
        if let Some(get_physical_device_memory_properties) = icd.get_physical_device_memory_properties {
            get_physical_device_memory_properties(physicalDevice, pMemoryProperties);
        }
    }
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
    
    // Forward to real ICD
    if let Some(icd) = super::forward::get_icd_if_enabled() {
        if let Some(get_physical_device_queue_family_properties) = icd.get_physical_device_queue_family_properties {
            get_physical_device_queue_family_properties(physicalDevice, pQueueFamilyPropertyCount, pQueueFamilyProperties);
        }
    }
}
