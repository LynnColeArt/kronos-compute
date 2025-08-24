//! Test ICD loader functionality

use kronos::*;
use kronos::implementation::*;
use std::ptr;
use std::ffi::CString;

fn main() {
    unsafe {
        println!("Testing Kronos ICD Loader...\n");
        
        // Initialize Kronos
        println!("1. Initializing Kronos...");
        match kronos::initialize_kronos() {
            Ok(()) => println!("   ✓ Kronos initialized successfully"),
            Err(e) => {
                println!("   ✗ Failed to initialize Kronos: {}", e);
                return;
            }
        }
        
        // Create instance
        println!("\n2. Creating Kronos instance...");
        let app_info = VkApplicationInfo {
            sType: VkStructureType::ApplicationInfo,
            pNext: ptr::null(),
            pApplicationName: CString::new("ICD Test").unwrap().as_ptr(),
            applicationVersion: 1,
            pEngineName: CString::new("Kronos").unwrap().as_ptr(),
            engineVersion: 1,
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
        let result = vkCreateInstance(&create_info, ptr::null(), &mut instance);
        
        if result != VkResult::Success {
            panic!("   ✗ Failed to create instance: {:?}", result);
        }
        println!("   ✓ Instance created successfully");
        
        // Enumerate physical devices
        println!("\n3. Enumerating physical devices...");
        let mut device_count = 0;
        vkEnumeratePhysicalDevices(instance, &mut device_count, ptr::null_mut());
        println!("   Found {} physical device(s)", device_count);
        
        if device_count > 0 {
            let mut devices = vec![VkPhysicalDevice::NULL; device_count as usize];
            vkEnumeratePhysicalDevices(instance, &mut device_count, devices.as_mut_ptr());
            
            // Get device properties
            for (i, &device) in devices.iter().enumerate() {
                let mut props = std::mem::zeroed::<VkPhysicalDeviceProperties>();
                vkGetPhysicalDeviceProperties(device, &mut props);
                
                let name = std::ffi::CStr::from_ptr(props.deviceName.as_ptr())
                    .to_string_lossy();
                println!("   Device {}: {}", i, name);
                println!("      Type: {:?}", props.deviceType);
                println!("      API Version: {}.{}.{}", 
                    (props.apiVersion >> 22) & 0x3FF,
                    (props.apiVersion >> 12) & 0x3FF,
                    props.apiVersion & 0xFFF);
            }
        }
        
                // Check backend mode
        println!("\n4. ICD loader test complete");
        
        // Clean up
        println!("\n5. Cleaning up...");
        vkDestroyInstance(instance, ptr::null());
        println!("   ✓ Instance destroyed");
        
        println!("\nTest completed successfully!");
    }
}