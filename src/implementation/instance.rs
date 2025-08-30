//! Instance creation and management

use crate::sys::*;
use crate::core::*;
use crate::ffi::*;
use std::ptr;

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
    // Aggregated mode: create per-ICD instances and return a meta instance
    if crate::implementation::icd_loader::aggregated_mode_enabled() {
        let all = crate::implementation::icd_loader::discover_and_load_all_icds();
        let mut inners = Vec::new();
        for icd in all {
            if let Some(create_instance_fn) = icd.create_instance {
                let mut inner_inst = VkInstance::NULL;
                let res = create_instance_fn(pCreateInfo, pAllocator, &mut inner_inst);
                if res == VkResult::Success && !inner_inst.is_null() {
                    inners.push((icd.clone(), inner_inst));
                }
            }
        }
        if inners.is_empty() {
            return VkResult::ErrorInitializationFailed;
        }
        let meta_id = crate::implementation::icd_loader::new_meta_instance_id();
        *pInstance = VkInstance::from_raw(meta_id);
        crate::implementation::icd_loader::set_meta_instance(meta_id, inners);
        return VkResult::Success;
    }
    
    // Try to use real Vulkan driver (single ICD)
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
    // Aggregated mode: destroy all inner instances
    if crate::implementation::icd_loader::aggregated_mode_enabled() {
        if let Some(inners) = crate::implementation::icd_loader::take_meta_instance(instance.as_raw()) {
            for (icd, inner) in inners {
                if let Some(f) = icd.destroy_instance { f(inner, pAllocator); }
            }
            return;
        }
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
    // Aggregated mode: sum counts across all inner instances for this meta instance
    if crate::implementation::icd_loader::aggregated_mode_enabled() {
        if let Some(inners) = crate::implementation::icd_loader::meta_instance_for(instance.as_raw()) {
            let mut total = 0u32;
            // First pass: count
            for (icd, inner) in &inners {
                if let Some(f) = icd.enumerate_physical_devices {
                    let mut count = 0u32;
                    let _ = f(*inner, &mut count, ptr::null_mut());
                    total = total.saturating_add(count);
                }
            }
            if pPhysicalDevices.is_null() {
                *pPhysicalDeviceCount = total;
                return VkResult::Success;
            }
            // Second pass: fill up to provided capacity
            let cap = unsafe { *pPhysicalDeviceCount as usize };
            let mut filled = 0usize;
            for (icd, inner) in &inners {
                if let Some(f) = icd.enumerate_physical_devices {
                    if filled >= cap { break; }
                    let mut count = (cap - filled) as u32;
                    let buf_ptr = unsafe { pPhysicalDevices.add(filled) };
                    let res = f(*inner, &mut count, buf_ptr);
                    if res == VkResult::Success || res == VkResult::Incomplete {
                        // Register ownership
                        for i in 0..count as isize {
                            let pd = unsafe { *buf_ptr.offset(i) };
                            crate::implementation::icd_loader::register_physical_device_icd(pd, icd);
                        }
                        filled += count as usize;
                    }
                }
            }
            // Set actual filled count
            unsafe { *pPhysicalDeviceCount = filled as u32; }
            if filled < total as usize { return VkResult::Incomplete; }
            return VkResult::Success;
        }
    }
    
    // Forward to real ICD (single)
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
    // Route by owning ICD if known
    if let Some(icd) = crate::implementation::icd_loader::icd_for_physical_device(physicalDevice) {
        if let Some(f) = icd.get_physical_device_properties { f(physicalDevice, pProperties); }
        return;
    }
    // Fallback to single ICD
    if let Some(icd) = super::forward::get_icd_if_enabled() {
        if let Some(f) = icd.get_physical_device_properties { f(physicalDevice, pProperties); }
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
    if let Some(icd) = crate::implementation::icd_loader::icd_for_physical_device(physicalDevice) {
        if let Some(f) = icd.get_physical_device_memory_properties { f(physicalDevice, pMemoryProperties); }
        return;
    }
    if let Some(icd) = super::forward::get_icd_if_enabled() {
        if let Some(f) = icd.get_physical_device_memory_properties { f(physicalDevice, pMemoryProperties); }
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
    if let Some(icd) = crate::implementation::icd_loader::icd_for_physical_device(physicalDevice) {
        if let Some(f) = icd.get_physical_device_queue_family_properties { f(physicalDevice, pQueueFamilyPropertyCount, pQueueFamilyProperties); }
        return;
    }
    if let Some(icd) = super::forward::get_icd_if_enabled() {
        if let Some(f) = icd.get_physical_device_queue_family_properties { f(physicalDevice, pQueueFamilyPropertyCount, pQueueFamilyProperties); }
    }
}
