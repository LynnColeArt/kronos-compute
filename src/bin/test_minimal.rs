//! Minimal test to debug where benchmark hangs

use kronos_compute::*;
use kronos_compute::implementation::initialize_kronos;
use std::ptr;

fn main() {
    unsafe {
        println!("1. Initializing Kronos...");
        initialize_kronos().expect("Failed to initialize");
        
        println!("2. Creating instance...");
        let app_info = VkApplicationInfo {
            sType: VkStructureType::ApplicationInfo,
            pNext: ptr::null(),
            pApplicationName: b"Test\0".as_ptr() as *const i8,
            applicationVersion: 0,
            pEngineName: b"Test\0".as_ptr() as *const i8,
            engineVersion: 0,
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
        vkCreateInstance(&create_info, ptr::null(), &mut instance);
        println!("3. Instance created");
        
        // Get devices
        println!("4. Enumerating devices...");
        let mut count = 0;
        vkEnumeratePhysicalDevices(instance, &mut count, ptr::null_mut());
        println!("   Found {} devices", count);
        
        if count == 0 {
            println!("No devices found!");
            vkDestroyInstance(instance, ptr::null());
            return;
        }
        
        let mut devices = vec![VkPhysicalDevice::NULL; count as usize];
        vkEnumeratePhysicalDevices(instance, &mut count, devices.as_mut_ptr());
        let physical_device = devices[0];
        println!("5. Got physical device");
        
        // Create logical device
        println!("6. Creating logical device...");
        let queue_priority = 1.0f32;
        let queue_create_info = VkDeviceQueueCreateInfo {
            sType: VkStructureType::DeviceQueueCreateInfo,
            pNext: ptr::null(),
            flags: 0,
            queueFamilyIndex: 0,
            queueCount: 1,
            pQueuePriorities: &queue_priority,
        };
        
        let device_create_info = VkDeviceCreateInfo {
            sType: VkStructureType::DeviceCreateInfo,
            pNext: ptr::null(),
            flags: 0,
            queueCreateInfoCount: 1,
            pQueueCreateInfos: &queue_create_info,
            enabledLayerCount: 0,
            ppEnabledLayerNames: ptr::null(),
            enabledExtensionCount: 0,
            ppEnabledExtensionNames: ptr::null(),
            pEnabledFeatures: ptr::null(),
        };
        
        let mut device = VkDevice::NULL;
        let result = vkCreateDevice(physical_device, &device_create_info, ptr::null(), &mut device);
        println!("7. Device creation result: {:?}", result);
        
        if result == VkResult::Success {
            println!("8. Getting queue...");
            let mut queue = VkQueue::NULL;
            vkGetDeviceQueue(device, 0, 0, &mut queue);
            println!("9. Got queue");
            
            // Skip allocator test for now
            println!("10. Skipping allocator test");
            
            // Cleanup
            vkDestroyDevice(device, ptr::null());
        }
        
        vkDestroyInstance(instance, ptr::null());
        println!("12. Done!");
    }
}