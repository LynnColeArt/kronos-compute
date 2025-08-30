//! Aggregated-mode end-to-end test (ignored by default)

use std::env;
use kronos_compute::sys::*;
use kronos_compute::core::*;

#[test]
#[ignore]
fn aggregate_enumerate_and_dispatch() {
    if env::var("KRONOS_RUN_ICD_TESTS").ok().as_deref() != Some("1") {
        eprintln!("skipping (set KRONOS_RUN_ICD_TESTS=1)\n");
        return;
    }
    env::set_var("KRONOS_AGGREGATE_ICD", "1");

    unsafe {
        // Create instance
        let name = std::ffi::CString::new("Aggregate E2E").unwrap();
        let app = VkApplicationInfo {
            sType: VkStructureType::ApplicationInfo,
            pNext: std::ptr::null(),
            pApplicationName: name.as_ptr(),
            applicationVersion: VK_MAKE_VERSION(1,0,0),
            pEngineName: name.as_ptr(),
            engineVersion: VK_MAKE_VERSION(1,0,0),
            apiVersion: VK_API_VERSION_1_0,
        };
        let ci = VkInstanceCreateInfo { sType: VkStructureType::InstanceCreateInfo, pNext: std::ptr::null(), flags: 0, pApplicationInfo: &app, enabledLayerCount: 0, ppEnabledLayerNames: std::ptr::null(), enabledExtensionCount: 0, ppEnabledExtensionNames: std::ptr::null() };
        let mut instance = VkInstance::NULL;
        let r = kronos_compute::vkCreateInstance(&ci, std::ptr::null(), &mut instance);
        assert_eq!(r, VkResult::Success);

        // Enumerate physical devices (aggregate)
        let mut count = 0u32;
        kronos_compute::vkEnumeratePhysicalDevices(instance, &mut count, std::ptr::null_mut());
        if count < 2 {
            eprintln!("Only {} device(s) aggregated; skipping multi-ICD create", count);
            // Still destroy instance
            kronos_compute::vkDestroyInstance(instance, std::ptr::null());
            return;
        }
        let mut phys = vec![VkPhysicalDevice::NULL; count as usize];
        kronos_compute::vkEnumeratePhysicalDevices(instance, &mut count, phys.as_mut_ptr());

        let pick = phys[1]; // choose the second device to exercise cross-ICD selection

        // Find compute queue family
        let mut qf_count = 0;
        kronos_compute::vkGetPhysicalDeviceQueueFamilyProperties(pick, &mut qf_count, std::ptr::null_mut());
        let mut qf = vec![VkQueueFamilyProperties { queueFlags: VkQueueFlags::empty(), queueCount: 0, timestampValidBits: 0, minImageTransferGranularity: VkExtent3D { width: 0, height: 0, depth: 0 } }; qf_count as usize];
        kronos_compute::vkGetPhysicalDeviceQueueFamilyProperties(pick, &mut qf_count, qf.as_mut_ptr());
        let q_index = qf.iter().position(|f| f.queueFlags.contains(VkQueueFlags::COMPUTE)).expect("no compute queue") as u32;

        // Create device on the chosen physical
        let prio = 1.0f32;
        let qinfo = VkDeviceQueueCreateInfo { sType: VkStructureType::DeviceQueueCreateInfo, pNext: std::ptr::null(), flags: 0, queueFamilyIndex: q_index, queueCount: 1, pQueuePriorities: &prio };
        let dinfo = VkDeviceCreateInfo { sType: VkStructureType::DeviceCreateInfo, pNext: std::ptr::null(), flags: 0, queueCreateInfoCount: 1, pQueueCreateInfos: &qinfo, enabledLayerCount: 0, ppEnabledLayerNames: std::ptr::null(), enabledExtensionCount: 0, ppEnabledExtensionNames: std::ptr::null(), pEnabledFeatures: std::ptr::null() };
        let mut device = VkDevice::NULL;
        let r = kronos_compute::vkCreateDevice(pick, &dinfo, std::ptr::null(), &mut device);
        assert_eq!(r, VkResult::Success);

        // Get queue
        let mut queue = VkQueue::NULL;
        kronos_compute::vkGetDeviceQueue(device, q_index, 0, &mut queue);

        // Create command pool/buffer and submit empty CB
        let pool_info = VkCommandPoolCreateInfo { sType: VkStructureType::CommandPoolCreateInfo, pNext: std::ptr::null(), flags: VkCommandPoolCreateFlags::empty(), queueFamilyIndex: q_index };
        let mut pool = VkCommandPool::NULL;
        let _ = kronos_compute::vkCreateCommandPool(device, &pool_info, std::ptr::null(), &mut pool);
        let alloc = VkCommandBufferAllocateInfo { sType: VkStructureType::CommandBufferAllocateInfo, pNext: std::ptr::null(), commandPool: pool, level: VkCommandBufferLevel::Primary, commandBufferCount: 1 };
        let mut cmd = VkCommandBuffer::NULL;
        let _ = kronos_compute::vkAllocateCommandBuffers(device, &alloc, &mut cmd);
        let begin = VkCommandBufferBeginInfo { sType: VkStructureType::CommandBufferBeginInfo, pNext: std::ptr::null(), flags: VkCommandBufferUsageFlags::ONE_TIME_SUBMIT, pInheritanceInfo: std::ptr::null() };
        assert_eq!(kronos_compute::vkBeginCommandBuffer(cmd, &begin), VkResult::Success);
        assert_eq!(kronos_compute::vkEndCommandBuffer(cmd), VkResult::Success);
        let submit = VkSubmitInfo { sType: VkStructureType::SubmitInfo, pNext: std::ptr::null(), waitSemaphoreCount: 0, pWaitSemaphores: std::ptr::null(), pWaitDstStageMask: std::ptr::null(), commandBufferCount: 1, pCommandBuffers: &cmd, signalSemaphoreCount: 0, pSignalSemaphores: std::ptr::null() };
        assert_eq!(kronos_compute::vkQueueSubmit(queue, 1, &submit, VkFence::NULL), VkResult::Success);
        let _ = kronos_compute::vkQueueWaitIdle(queue);

        // Cleanup
        kronos_compute::vkDestroyCommandPool(device, pool, std::ptr::null());
        kronos_compute::vkDestroyDevice(device, std::ptr::null());
        kronos_compute::vkDestroyInstance(instance, std::ptr::null());
    }
}

