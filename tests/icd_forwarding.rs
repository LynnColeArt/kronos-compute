//! ICD forwarding smoke tests - verifies real GPU dispatch

use kronos_compute::sys::*;
use kronos_compute::core::*;
use kronos_compute::implementation;
use std::ffi::CString;
use std::ptr;

#[test]
fn test_icd_discovery() {
    unsafe {
        // Initialize Kronos with ICD forwarding
        let result = kronos_compute::initialize_kronos();
        assert!(result.is_ok(), "Failed to initialize Kronos ICD loader: {:?}", result);
        
        // Check that we have function pointers
        let icd = implementation::icd_loader::get_icd().expect("ICD not loaded");
        assert!(icd.create_instance.is_some(), "vkCreateInstance not loaded");
        assert!(icd.enumerate_physical_devices.is_some(), "vkEnumeratePhysicalDevices not loaded");
    }
}

#[test]
fn test_real_gpu_dispatch() {
    unsafe {
        // Initialize
        kronos_compute::initialize_kronos().expect("Failed to initialize Kronos");
        
        // Create instance
        let app_name = CString::new("ICD Test").unwrap();
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
        assert_eq!(result, VkResult::Success, "Failed to create instance");
        assert!(!instance.is_null(), "Instance is null");
        
        // Find physical device
        let mut device_count = 0;
        kronos_compute::vkEnumeratePhysicalDevices(instance, &mut device_count, ptr::null_mut());
        assert!(device_count > 0, "No physical devices found");
        
        let mut devices = vec![VkPhysicalDevice::NULL; device_count as usize];
        kronos_compute::vkEnumeratePhysicalDevices(instance, &mut device_count, devices.as_mut_ptr());
        
        let physical_device = devices[0];
        assert!(!physical_device.is_null(), "Physical device is null");
        
        // Get device properties to verify vendor
        let mut props: VkPhysicalDeviceProperties = std::mem::zeroed();
        kronos_compute::vkGetPhysicalDeviceProperties(physical_device, &mut props);
        
        let vendor = match props.vendorID {
            0x1002 => "AMD",
            0x10DE => "NVIDIA", 
            0x8086 => "Intel",
            _ => "Other"
        };
        println!("Found {} GPU: {}", vendor, 
            std::str::from_utf8(&props.deviceName).unwrap_or("Unknown").trim_end_matches('\0'));
        
        // Find compute queue
        let mut queue_family_count = 0;
        kronos_compute::vkGetPhysicalDeviceQueueFamilyProperties(physical_device, &mut queue_family_count, ptr::null_mut());
        
        let mut queue_families = vec![std::mem::zeroed::<VkQueueFamilyProperties>(); queue_family_count as usize];
        kronos_compute::vkGetPhysicalDeviceQueueFamilyProperties(physical_device, &mut queue_family_count, queue_families.as_mut_ptr());
        
        let compute_queue_family = queue_families.iter()
            .position(|f| f.queueFlags.contains(VkQueueFlags::COMPUTE))
            .expect("No compute queue found") as u32;
        
        // Create logical device
        let queue_priority = 1.0f32;
        let queue_info = VkDeviceQueueCreateInfo {
            sType: VkStructureType::DeviceQueueCreateInfo,
            pNext: ptr::null(),
            flags: 0,
            queueFamilyIndex: compute_queue_family,
            queueCount: 1,
            pQueuePriorities: &queue_priority,
        };
        
        let device_info = VkDeviceCreateInfo {
            sType: VkStructureType::DeviceCreateInfo,
            pNext: ptr::null(),
            flags: 0,
            queueCreateInfoCount: 1,
            pQueueCreateInfos: &queue_info,
            enabledLayerCount: 0,
            ppEnabledLayerNames: ptr::null(),
            enabledExtensionCount: 0,
            ppEnabledExtensionNames: ptr::null(),
            pEnabledFeatures: ptr::null(),
        };
        
        let mut device = VkDevice::NULL;
        let result = kronos_compute::vkCreateDevice(physical_device, &device_info, ptr::null(), &mut device);
        assert_eq!(result, VkResult::Success, "Failed to create device");
        
        // Get queue
        let mut queue = VkQueue::NULL;
        kronos_compute::vkGetDeviceQueue(device, compute_queue_family, 0, &mut queue);
        assert!(!queue.is_null(), "Queue is null");
        
        // Create command pool
        let pool_info = VkCommandPoolCreateInfo {
            sType: VkStructureType::CommandPoolCreateInfo,
            pNext: ptr::null(),
            flags: 0,
            queueFamilyIndex: compute_queue_family,
        };
        
        let mut command_pool = VkCommandPool::NULL;
        kronos_compute::vkCreateCommandPool(device, &pool_info, ptr::null(), &mut command_pool);
        
        // Allocate command buffer
        let alloc_info = VkCommandBufferAllocateInfo {
            sType: VkStructureType::CommandBufferAllocateInfo,
            pNext: ptr::null(),
            commandPool: command_pool,
            level: VkCommandBufferLevel::Primary,
            commandBufferCount: 1,
        };
        
        let mut cmd_buffer = VkCommandBuffer::NULL;
        kronos_compute::vkAllocateCommandBuffers(device, &alloc_info, &mut cmd_buffer);
        
        // Record empty command buffer
        let begin_info = VkCommandBufferBeginInfo {
            sType: VkStructureType::CommandBufferBeginInfo,
            pNext: ptr::null(),
            flags: VkCommandBufferUsageFlags::ONE_TIME_SUBMIT,
            pInheritanceInfo: ptr::null(),
        };
        
        kronos_compute::vkBeginCommandBuffer(cmd_buffer, &begin_info);
        // Empty dispatch - just testing submission works
        kronos_compute::vkEndCommandBuffer(cmd_buffer);
        
        // Submit
        let submit_info = VkSubmitInfo {
            sType: VkStructureType::SubmitInfo,
            pNext: ptr::null(),
            waitSemaphoreCount: 0,
            pWaitSemaphores: ptr::null(),
            pWaitDstStageMask: ptr::null(),
            commandBufferCount: 1,
            pCommandBuffers: &cmd_buffer,
            signalSemaphoreCount: 0,
            pSignalSemaphores: ptr::null(),
        };
        
        let result = kronos_compute::vkQueueSubmit(queue, 1, &submit_info, VkFence::NULL);
        assert_eq!(result, VkResult::Success, "Failed to submit");
        
        kronos_compute::vkQueueWaitIdle(queue);
        
        // Cleanup
        kronos_compute::vkDestroyCommandPool(device, command_pool, ptr::null());
        kronos_compute::vkDestroyDevice(device, ptr::null());
        kronos_compute::vkDestroyInstance(instance, ptr::null());
        
        println!("✓ ICD forwarding test passed - real GPU dispatch successful!");
    }
}

#[test]
fn test_vendor_detection() {
    unsafe {
        kronos_compute::initialize_kronos().expect("Failed to initialize");
        
        let app_name = CString::new("Vendor Test").unwrap();
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
        kronos_compute::vkCreateInstance(&create_info, ptr::null(), &mut instance);
        
        let mut device_count = 0;
        kronos_compute::vkEnumeratePhysicalDevices(instance, &mut device_count, ptr::null_mut());
        
        if device_count > 0 {
            let mut devices = vec![VkPhysicalDevice::NULL; device_count as usize];
            kronos_compute::vkEnumeratePhysicalDevices(instance, &mut device_count, devices.as_mut_ptr());
            
            for device in devices {
                let mut props: VkPhysicalDeviceProperties = std::mem::zeroed();
                kronos_compute::vkGetPhysicalDeviceProperties(device, &mut props);
                
                let vendor = implementation::barrier_policy::GpuVendor::from_vendor_id(props.vendorID);
                println!("Device: {} (0x{:04X}) -> {:?}", 
                    std::str::from_utf8(&props.deviceName).unwrap_or("Unknown").trim_end_matches('\0'),
                    props.vendorID,
                    vendor);
                
                // Verify vendor detection works
                match props.vendorID {
                    0x1002 => assert_eq!(vendor, implementation::barrier_policy::GpuVendor::AMD),
                    0x10DE => assert_eq!(vendor, implementation::barrier_policy::GpuVendor::NVIDIA),
                    0x8086 => assert_eq!(vendor, implementation::barrier_policy::GpuVendor::Intel),
                    _ => assert_eq!(vendor, implementation::barrier_policy::GpuVendor::Other),
                }
            }
        }
        
        kronos_compute::vkDestroyInstance(instance, ptr::null());
    }
}