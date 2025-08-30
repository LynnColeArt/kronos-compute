//! ICD dispatch sanity test: prefer a selected ICD and submit an empty command buffer

use std::env;
use kronos_compute::sys::*;
use kronos_compute::core::*;

#[test]
#[ignore]
fn dispatch_empty_cmd_buffer() {
    if env::var("KRONOS_RUN_ICD_TESTS").ok().as_deref() != Some("1") {
        eprintln!("skipping (set KRONOS_RUN_ICD_TESTS=1 to run)\n");
        return;
    }

    unsafe {
        // Prefer first ICD (hardware preference will already filter software if available)
        kronos_compute::implementation::icd_loader::set_preferred_icd_index(0);

        // Create instance
        let app_name = std::ffi::CString::new("ICD Dispatch Test").unwrap();
        let app_info = VkApplicationInfo {
            sType: VkStructureType::ApplicationInfo,
            pNext: std::ptr::null(),
            pApplicationName: app_name.as_ptr(),
            applicationVersion: VK_MAKE_VERSION(1, 0, 0),
            pEngineName: app_name.as_ptr(),
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
        let res = kronos_compute::vkCreateInstance(&create_info, std::ptr::null(), &mut instance);
        assert_eq!(res, VkResult::Success, "vkCreateInstance failed: {res:?}");

        // Enumerate physical devices
        let mut device_count = 0;
        kronos_compute::vkEnumeratePhysicalDevices(instance, &mut device_count, std::ptr::null_mut());
        assert!(device_count > 0, "no physical devices found");
        let mut devices = vec![VkPhysicalDevice::NULL; device_count as usize];
        kronos_compute::vkEnumeratePhysicalDevices(instance, &mut device_count, devices.as_mut_ptr());
        let physical = devices[0];

        // Find compute family
        let mut qf_count = 0;
        kronos_compute::vkGetPhysicalDeviceQueueFamilyProperties(physical, &mut qf_count, std::ptr::null_mut());
        let mut qf_props = vec![VkQueueFamilyProperties { queueFlags: VkQueueFlags::empty(), queueCount: 0, timestampValidBits: 0, minImageTransferGranularity: VkExtent3D { width: 0, height: 0, depth: 0 } }; qf_count as usize];
        kronos_compute::vkGetPhysicalDeviceQueueFamilyProperties(physical, &mut qf_count, qf_props.as_mut_ptr());
        let compute_q = qf_props.iter().position(|f| f.queueFlags.contains(VkQueueFlags::COMPUTE)).expect("no compute queue") as u32;

        // Create device
        let prio = 1.0f32;
        let qinfo = VkDeviceQueueCreateInfo { sType: VkStructureType::DeviceQueueCreateInfo, pNext: std::ptr::null(), flags: 0, queueFamilyIndex: compute_q, queueCount: 1, pQueuePriorities: &prio };
        let dinfo = VkDeviceCreateInfo {
            sType: VkStructureType::DeviceCreateInfo,
            pNext: std::ptr::null(),
            flags: 0,
            queueCreateInfoCount: 1,
            pQueueCreateInfos: &qinfo,
            enabledLayerCount: 0,
            ppEnabledLayerNames: std::ptr::null(),
            enabledExtensionCount: 0,
            ppEnabledExtensionNames: std::ptr::null(),
            pEnabledFeatures: std::ptr::null(),
        };
        let mut device = VkDevice::NULL;
        let res = kronos_compute::vkCreateDevice(physical, &dinfo, std::ptr::null(), &mut device);
        assert_eq!(res, VkResult::Success, "vkCreateDevice failed: {res:?}");

        // Get queue
        let mut queue = VkQueue::NULL;
        kronos_compute::vkGetDeviceQueue(device, compute_q, 0, &mut queue);
        assert!(!queue.is_null());

        // Command pool + buffer
        let pool_info = VkCommandPoolCreateInfo { sType: VkStructureType::CommandPoolCreateInfo, pNext: std::ptr::null(), flags: VkCommandPoolCreateFlags::empty(), queueFamilyIndex: compute_q };
        let mut pool = VkCommandPool::NULL;
        let res = kronos_compute::vkCreateCommandPool(device, &pool_info, std::ptr::null(), &mut pool);
        assert_eq!(res, VkResult::Success);

        let alloc_info = VkCommandBufferAllocateInfo { sType: VkStructureType::CommandBufferAllocateInfo, pNext: std::ptr::null(), commandPool: pool, level: VkCommandBufferLevel::Primary, commandBufferCount: 1 };
        let mut cmd = VkCommandBuffer::NULL;
        let _ = kronos_compute::vkAllocateCommandBuffers(device, &alloc_info, &mut cmd);

        // Record empty CB
        let begin = VkCommandBufferBeginInfo { sType: VkStructureType::CommandBufferBeginInfo, pNext: std::ptr::null(), flags: VkCommandBufferUsageFlags::ONE_TIME_SUBMIT, pInheritanceInfo: std::ptr::null() };
        let res = kronos_compute::vkBeginCommandBuffer(cmd, &begin);
        assert_eq!(res, VkResult::Success);
        let res = kronos_compute::vkEndCommandBuffer(cmd);
        assert_eq!(res, VkResult::Success);

        // Submit + wait
        let submit = VkSubmitInfo { sType: VkStructureType::SubmitInfo, pNext: std::ptr::null(), waitSemaphoreCount: 0, pWaitSemaphores: std::ptr::null(), pWaitDstStageMask: std::ptr::null(), commandBufferCount: 1, pCommandBuffers: &cmd, signalSemaphoreCount: 0, pSignalSemaphores: std::ptr::null() };
        let res = kronos_compute::vkQueueSubmit(queue, 1, &submit, VkFence::NULL);
        assert_eq!(res, VkResult::Success, "vkQueueSubmit failed: {res:?}");
        let _ = kronos_compute::vkQueueWaitIdle(queue);

        // Cleanup
        kronos_compute::vkDestroyCommandPool(device, pool, std::ptr::null());
        kronos_compute::vkDestroyDevice(device, std::ptr::null());
        kronos_compute::vkDestroyInstance(instance, std::ptr::null());

        // Clear preference for other tests
        kronos_compute::implementation::icd_loader::clear_preferred_icd();
    }
}

