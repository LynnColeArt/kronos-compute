//! Test AMD driver by path

use kronos_compute::*;
use std::ptr;

fn main() {
    env_logger::init();
    
    println!("Testing AMD Driver by Path");
    println!("=========================");
    
    // Set AMD preference by path
    kronos_compute::implementation::icd_loader::set_preferred_icd_path("/usr/lib/x86_64-linux-gnu/libvulkan_radeon.so");
    
    // Initialize
    if let Err(e) = initialize_kronos() {
        println!("Failed to initialize: {:?}", e);
        return;
    }
    
    // Check loaded ICD
    if let Some(info) = kronos_compute::implementation::icd_loader::selected_icd_info() {
        println!("Loaded ICD: {}", info.library_path.display());
        
        // Create instance
        let app_info = VkApplicationInfo {
            sType: VkStructureType::ApplicationInfo,
            pNext: ptr::null(),
            pApplicationName: b"AMD Test\0".as_ptr() as *const i8,
            applicationVersion: VK_MAKE_VERSION(1, 0, 0),
            pEngineName: b"Kronos\0".as_ptr() as *const i8,
            engineVersion: VK_MAKE_VERSION(1, 0, 0),
            apiVersion: VK_API_VERSION_1_0,
        };
        
        let create_info = VkInstanceCreateInfo {
            sType: VkStructureType::InstanceCreateInfo,
            pNext: ptr::null(),
            flags: 0,
            pApplicationInfo: &app_info,
            enabledLayerCount: 0,
            ppEnabledLayerNames: ptr::null(),
            enabledExtensionCount: 0,
            ppEnabledExtensionNames: ptr::null(),
        };
        
        let mut instance = VkInstance::NULL;
        unsafe {
            let result = vkCreateInstance(&create_info, ptr::null(), &mut instance);
            if result != VkResult::Success {
                println!("Failed to create instance: {:?}", result);
                return;
            }
            println!("✓ Instance created");
            
            // Enumerate devices
            let mut count = 0;
            let result = vkEnumeratePhysicalDevices(instance, &mut count, ptr::null_mut());
            println!("First enumeration: result={:?}, count={}", result, count);
            
            if count > 0 {
                let mut devices = vec![VkPhysicalDevice::NULL; count as usize];
                let result = vkEnumeratePhysicalDevices(instance, &mut count, devices.as_mut_ptr());
                println!("Second enumeration: result={:?}, got {} devices", result, count);
                
                // Get device properties
                for (i, &device) in devices.iter().enumerate() {
                    let mut props = VkPhysicalDeviceProperties::default();
                    vkGetPhysicalDeviceProperties(device, &mut props);
                    let name_bytes = &props.deviceName;
                    let null_pos = name_bytes.iter().position(|&c| c == 0).unwrap_or(name_bytes.len());
                    let name_u8: Vec<u8> = name_bytes[..null_pos].iter().map(|&c| c as u8).collect();
                    let name = std::str::from_utf8(&name_u8).unwrap_or("Unknown");
                    println!("Device {}: {}", i, name);
                }
            }
            
            println!("About to destroy instance...");
            vkDestroyInstance(instance, ptr::null());
            println!("Instance destroyed successfully");
        }
    } else {
        println!("No ICD loaded!");
    }
}