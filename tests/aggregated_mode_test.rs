//! Test aggregated mode multi-ICD enumeration

use kronos_compute::sys::*;
use kronos_compute::core::*;
use kronos_compute::VkResult;
use std::env;

#[test]
fn test_aggregated_mode_enumeration() {
    // Enable aggregated mode
    env::set_var("KRONOS_AGGREGATE_ICD", "1");
    
    unsafe {
        // Initialize ICD loader
        let _ = kronos_compute::implementation::icd_loader::initialize_icd_loader();
        
        // Get all available ICDs
        let all_icds = kronos_compute::implementation::icd_loader::get_all_icds();
        println!("ALL_ICDS contains {} ICDs", all_icds.len());
        
        for (idx, icd) in all_icds.iter().enumerate() {
            println!("  [{}] {}", idx, icd.library_path.display());
        }
        
        // Create instance in aggregated mode
        let app_info = VkApplicationInfo {
            sType: VkStructureType::ApplicationInfo,
            pNext: std::ptr::null(),
            pApplicationName: b"Aggregated Test\0".as_ptr() as *const i8,
            applicationVersion: VK_MAKE_VERSION(1, 0, 0),
            pEngineName: b"Kronos\0".as_ptr() as *const i8,
            engineVersion: VK_MAKE_VERSION(1, 0, 0),
            apiVersion: VK_API_VERSION_1_0,
        };
        
        let create_info = VkInstanceCreateInfo {
            sType: VkStructureType::InstanceCreateInfo,
            pNext: std::ptr::null(),
            flags: 0,
            pApplicationInfo: &app_info,
            enabledLayerCount: 0,
            ppEnabledLayerNames: std::ptr::null(),
            enabledExtensionCount: 0,
            ppEnabledExtensionNames: std::ptr::null(),
        };
        
        let mut instance = VkInstance::NULL;
        let result = kronos_compute::vkCreateInstance(&create_info, std::ptr::null(), &mut instance);
        
        if result == VkResult::Success {
            println!("\nInstance created successfully");
            
            // Enumerate physical devices
            let mut device_count = 0u32;
            kronos_compute::vkEnumeratePhysicalDevices(instance, &mut device_count, std::ptr::null_mut());
            println!("Found {} physical devices across all ICDs", device_count);
            
            if device_count > 0 {
                let mut devices = vec![VkPhysicalDevice::NULL; device_count as usize];
                kronos_compute::vkEnumeratePhysicalDevices(instance, &mut device_count, devices.as_mut_ptr());
                
                for (idx, &device) in devices.iter().enumerate() {
                    let mut props = VkPhysicalDeviceProperties {
                        apiVersion: 0,
                        driverVersion: 0,
                        vendorID: 0,
                        deviceID: 0,
                        deviceType: VkPhysicalDeviceType::Other,
                        deviceName: [0; 256],
                        pipelineCacheUUID: [0; 16],
                        limits: std::mem::zeroed(),
                        sparseProperties: std::mem::zeroed(),
                    };
                    
                    kronos_compute::vkGetPhysicalDeviceProperties(device, &mut props);
                    
                    let name_end = props.deviceName.iter().position(|&c| c == 0).unwrap_or(256);
                    let name_bytes: Vec<u8> = props.deviceName[..name_end].iter().map(|&c| c as u8).collect();
                    let device_name = String::from_utf8_lossy(&name_bytes);
                    
                    println!("  [{}] {}", idx, device_name.trim());
                }
            }
            
            kronos_compute::vkDestroyInstance(instance, std::ptr::null());
        } else {
            println!("Failed to create instance: {:?}", result);
        }
    }
}