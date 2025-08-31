//! REAL Kronos instance implementation - NO ICD forwarding!

use crate::sys::*;
use crate::core::*;
use crate::ffi::*;
use std::ptr;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Mutex;
use std::collections::HashMap;

// Instance handle counter
static INSTANCE_COUNTER: AtomicU64 = AtomicU64::new(1);

// Registry of active instances
lazy_static::lazy_static! {
    static ref INSTANCES: Mutex<HashMap<u64, InstanceData>> = Mutex::new(HashMap::new());
}

struct InstanceData {
    app_info: ApplicationInfo,
    enabled_extensions: Vec<String>,
}

struct ApplicationInfo {
    app_name: String,
    app_version: u32,
    engine_name: String,
    engine_version: u32,
    api_version: u32,
}

/// Create a Kronos instance - REAL implementation, no ICD forwarding
#[no_mangle]
pub unsafe extern "C" fn vkCreateInstance(
    pCreateInfo: *const VkInstanceCreateInfo,
    _pAllocator: *const VkAllocationCallbacks,
    pInstance: *mut VkInstance,
) -> VkResult {
    if pCreateInfo.is_null() || pInstance.is_null() {
        return VkResult::ErrorInitializationFailed;
    }
    
    let create_info = &*pCreateInfo;
    
    // Extract application info
    let app_info = if !create_info.pApplicationInfo.is_null() {
        let info = &*create_info.pApplicationInfo;
        ApplicationInfo {
            app_name: c_str_to_string(info.pApplicationName).unwrap_or_default(),
            app_version: info.applicationVersion,
            engine_name: c_str_to_string(info.pEngineName).unwrap_or_default(),
            engine_version: info.engineVersion,
            api_version: info.apiVersion,
        }
    } else {
        ApplicationInfo {
            app_name: String::new(),
            app_version: 0,
            engine_name: String::new(),
            engine_version: 0,
            api_version: VK_API_VERSION_1_0,
        }
    };
    
    // Check API version - we only support compute
    if app_info.api_version > VK_API_VERSION_1_3 {
        return VkResult::ErrorIncompatibleDriver;
    }
    
    // Parse enabled extensions
    let mut extensions = Vec::new();
    for i in 0..create_info.enabledExtensionCount {
        let ext_name = *create_info.ppEnabledExtensionNames.add(i as usize);
        if let Some(name) = c_str_to_string(ext_name) {
            // We don't support any extensions for compute-only
            log::warn!("Extension requested but not supported: {}", name);
            // Don't fail, just ignore
            extensions.push(name);
        }
    }
    
    // Create instance handle
    let handle = INSTANCE_COUNTER.fetch_add(1, Ordering::SeqCst);
    
    // Store instance data
    let instance_data = InstanceData {
        app_info,
        enabled_extensions: extensions,
    };
    
    INSTANCES.lock().unwrap().insert(handle, instance_data);
    
    // Return handle
    *pInstance = VkInstance::from_raw(handle);
    
    log::info!("Created Kronos instance {:?} - compute-only, no ICD", handle);
    
    VkResult::Success
}

/// Destroy instance
#[no_mangle]
pub unsafe extern "C" fn vkDestroyInstance(
    instance: VkInstance,
    _pAllocator: *const VkAllocationCallbacks,
) {
    if instance.is_null() {
        return;
    }
    
    let handle = instance.as_raw();
    INSTANCES.lock().unwrap().remove(&handle);
    
    log::info!("Destroyed Kronos instance {:?}", handle);
}

/// Enumerate physical devices - return our virtual compute device
#[no_mangle]
pub unsafe extern "C" fn vkEnumeratePhysicalDevices(
    instance: VkInstance,
    pPhysicalDeviceCount: *mut u32,
    pPhysicalDevices: *mut VkPhysicalDevice,
) -> VkResult {
    if instance.is_null() || pPhysicalDeviceCount.is_null() {
        return VkResult::ErrorInitializationFailed;
    }
    
    // Verify instance exists
    let handle = instance.as_raw();
    if !INSTANCES.lock().unwrap().contains_key(&handle) {
        return VkResult::ErrorDeviceLost;
    }
    
    // We have exactly 1 virtual compute device
    if pPhysicalDevices.is_null() {
        *pPhysicalDeviceCount = 1;
        return VkResult::Success;
    }
    
    let count = *pPhysicalDeviceCount;
    if count == 0 {
        return VkResult::Incomplete;
    }
    
    // Return our virtual device
    *pPhysicalDevices = VkPhysicalDevice::from_raw(1); // Fixed ID for our device
    *pPhysicalDeviceCount = 1;
    
    VkResult::Success
}

// Helper to convert C string to Rust String
unsafe fn c_str_to_string(ptr: *const i8) -> Option<String> {
    if ptr.is_null() {
        return None;
    }
    std::ffi::CStr::from_ptr(ptr)
        .to_str()
        .ok()
        .map(|s| s.to_string())
}