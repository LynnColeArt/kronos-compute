//! Simple ICD information tool - shows available Vulkan devices

use kronos_compute::sys::*;
use kronos_compute::core::*;
use kronos_compute::*;
use std::ffi::CString;
use std::ptr;

fn main() {
    println!("Kronos ICD Information Tool");
    println!("===========================\n");
    
    unsafe {
        // Initialize Kronos
        match kronos_compute::initialize_kronos() {
            Ok(_) => println!("✓ Kronos ICD loader initialized"),
            Err(e) => {
                eprintln!("✗ Failed to initialize Kronos: {:?}", e);
                return;
            }
        }
        
        // Create instance
        let app_name = CString::new("ICD Info").unwrap();
        let app_info = VkApplicationInfo {
            sType: VkStructureType::ApplicationInfo,
            pNext: ptr::null(),
            pApplicationName: app_name.as_ptr(),
            applicationVersion: VK_MAKE_VERSION(1, 0, 0),
            pEngineName: app_name.as_ptr(),
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
        let result = kronos_compute::vkCreateInstance(&create_info, ptr::null(), &mut instance);
        if result != VkResult::Success {
            eprintln!("✗ Failed to create instance: {:?}", result);
            return;
        }
        println!("✓ Vulkan instance created\n");
        
        // Enumerate physical devices
        let mut device_count = 0;
        kronos_compute::vkEnumeratePhysicalDevices(instance, &mut device_count, ptr::null_mut());
        println!("Found {} physical device(s):\n", device_count);
        
        if device_count == 0 {
            println!("⚠️  No Vulkan devices found. Check your drivers.");
            kronos_compute::vkDestroyInstance(instance, ptr::null());
            return;
        }
        
        let mut devices = vec![VkPhysicalDevice::NULL; device_count as usize];
        kronos_compute::vkEnumeratePhysicalDevices(instance, &mut device_count, devices.as_mut_ptr());
        
        // Print info for each device
        for (idx, device) in devices.iter().enumerate() {
            let mut props: VkPhysicalDeviceProperties = std::mem::zeroed();
            kronos_compute::vkGetPhysicalDeviceProperties(*device, &mut props);
            
            let device_name = std::str::from_utf8(&props.deviceName)
                .unwrap_or("Unknown")
                .trim_end_matches('\0');
            
            println!("Device {}: {}", idx, device_name);
            println!("  Type: {:?}", props.deviceType);
            println!("  Vendor ID: 0x{:04X} ({})", props.vendorID, vendor_name(props.vendorID));
            println!("  Device ID: 0x{:04X}", props.deviceID);
            println!("  API Version: {}.{}.{}", 
                (props.apiVersion >> 22) & 0x3FF,
                (props.apiVersion >> 12) & 0x3FF,
                props.apiVersion & 0xFFF
            );
            println!("  Driver Version: {}.{}.{}", 
                (props.driverVersion >> 22) & 0x3FF,
                (props.driverVersion >> 12) & 0x3FF,
                props.driverVersion & 0xFFF
            );
            
            // Get memory info
            let mut mem_props: VkPhysicalDeviceMemoryProperties = std::mem::zeroed();
            kronos_compute::vkGetPhysicalDeviceMemoryProperties(*device, &mut mem_props);
            
            println!("  Memory Types: {}", mem_props.memoryTypeCount);
            let mut total_device_memory = 0u64;
            let mut total_host_memory = 0u64;
            
            for i in 0..mem_props.memoryHeapCount {
                let heap = &mem_props.memoryHeaps[i as usize];
                if heap.flags.contains(VkMemoryHeapFlags::DEVICE_LOCAL) {
                    total_device_memory += heap.size;
                } else {
                    total_host_memory += heap.size;
                }
            }
            
            println!("  Device Memory: {} MB", total_device_memory / (1024 * 1024));
            println!("  Host Memory: {} MB", total_host_memory / (1024 * 1024));
            
            // Get queue families
            let mut queue_family_count = 0;
            kronos_compute::vkGetPhysicalDeviceQueueFamilyProperties(*device, &mut queue_family_count, ptr::null_mut());
            println!("  Queue Families: {}", queue_family_count);
            
            let mut queue_families = vec![std::mem::zeroed::<VkQueueFamilyProperties>(); queue_family_count as usize];
            kronos_compute::vkGetPhysicalDeviceQueueFamilyProperties(*device, &mut queue_family_count, queue_families.as_mut_ptr());
            
            for (i, family) in queue_families.iter().enumerate() {
                let mut caps = Vec::new();
                if family.queueFlags.contains(VkQueueFlags::COMPUTE) {
                    caps.push("COMPUTE");
                }
                if family.queueFlags.contains(VkQueueFlags::TRANSFER) {
                    caps.push("TRANSFER");
                }
                if family.queueFlags.contains(VkQueueFlags::SPARSE_BINDING) {
                    caps.push("SPARSE");
                }
                
                println!("    Family {}: {} queue(s) - {}", 
                    i, family.queueCount, caps.join(", "));
            }
            
            // Check features relevant to compute
            let mut features: VkPhysicalDeviceFeatures = std::mem::zeroed();
            kronos_compute::vkGetPhysicalDeviceFeatures(*device, &mut features);
            
            println!("  Compute Features:");
            println!("    Robust buffer access: {}", features.robustBufferAccess != 0);
            println!("    Shader Int64: {}", features.shaderInt64 != 0);
            println!("    Shader Int16: {}", features.shaderInt16 != 0);
            
            // Vendor-specific optimization info
            let vendor = kronos_compute::implementation::barrier_policy::GpuVendor::from_vendor_id(props.vendorID);
            println!("  Kronos Optimizations: {:?} profile", vendor);
            
            println!();
        }
        
        // Cleanup
        kronos_compute::vkDestroyInstance(instance, ptr::null());
        println!("✓ Cleanup complete");
    }
}

fn vendor_name(id: u32) -> &'static str {
    match id {
        0x1002 => "AMD",
        0x10DE => "NVIDIA",
        0x8086 => "Intel",
        0x1010 => "ImgTec",
        0x13B5 => "ARM",
        0x5143 => "Qualcomm",
        _ => "Other"
    }
}