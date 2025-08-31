//! Test queue family enumeration directly

use kronos_compute::*;
use std::ptr;

fn main() {
    unsafe {
        println!("Testing Queue Family Enumeration");
        println!("================================");
        
        // Initialize Kronos
        if let Err(e) = initialize_kronos() {
            eprintln!("Failed to initialize Kronos: {:?}", e);
            return;
        }
        println!("✓ Kronos initialized");
        
        // Create instance
        let app_info = VkApplicationInfo {
            sType: VkStructureType::ApplicationInfo,
            pNext: ptr::null(),
            pApplicationName: b"Queue Test\0".as_ptr() as *const i8,
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
        let result = vkCreateInstance(&create_info, ptr::null(), &mut instance);
        if result != VkResult::Success {
            eprintln!("Failed to create instance: {:?}", result);
            return;
        }
        println!("✓ Instance created");
        
        // Enumerate physical devices
        let mut device_count = 0;
        vkEnumeratePhysicalDevices(instance, &mut device_count, ptr::null_mut());
        println!("Found {} physical devices", device_count);
        
        if device_count == 0 {
            println!("No devices found!");
            vkDestroyInstance(instance, ptr::null());
            return;
        }
        
        let mut devices = vec![VkPhysicalDevice::NULL; device_count as usize];
        vkEnumeratePhysicalDevices(instance, &mut device_count, devices.as_mut_ptr());
        
        // Test queue family enumeration on first device
        let device = devices[0];
        println!("\nTesting device 0: {:?}", device);
        
        let mut queue_family_count = 0;
        println!("Calling vkGetPhysicalDeviceQueueFamilyProperties (count query)...");
        vkGetPhysicalDeviceQueueFamilyProperties(device, &mut queue_family_count, ptr::null_mut());
        println!("Device has {} queue families", queue_family_count);
        
        if queue_family_count > 0 {
            let mut queue_families = vec![
                VkQueueFamilyProperties {
                    queueFlags: VkQueueFlags::empty(),
                    queueCount: 0,
                    timestampValidBits: 0,
                    minImageTransferGranularity: VkExtent3D { width: 0, height: 0, depth: 0 },
                };
                queue_family_count as usize
            ];
            println!("Calling vkGetPhysicalDeviceQueueFamilyProperties (data query)...");
            vkGetPhysicalDeviceQueueFamilyProperties(device, &mut queue_family_count, queue_families.as_mut_ptr());
            println!("✓ Got queue family properties");
            
            for (i, family) in queue_families.iter().enumerate() {
                println!("  Queue family {}: flags={:?}, count={}", i, family.queueFlags, family.queueCount);
            }
        }
        
        // Cleanup
        vkDestroyInstance(instance, ptr::null());
        println!("\n✓ Test completed successfully");
    }
}