//! Aggregated-mode stress test: concurrent create/submit/teardown across ICDs

use std::{env, thread, time::Duration};
use kronos_compute::sys::*;
use kronos_compute::core::*;
use kronos_compute::VkResult;

#[test]
#[ignore]
fn aggregate_concurrent_stress() {
    if env::var("KRONOS_RUN_ICD_TESTS").ok().as_deref() != Some("1") {
        eprintln!("skipping (set KRONOS_RUN_ICD_TESTS=1)\n");
        return;
    }
    env::set_var("KRONOS_AGGREGATE_ICD", "1");

    unsafe {
        // Create aggregated instance
        let name = std::ffi::CString::new("Aggregate Stress").unwrap();
        let app = VkApplicationInfo { sType: VkStructureType::ApplicationInfo, pNext: std::ptr::null(), pApplicationName: name.as_ptr(), applicationVersion: VK_MAKE_VERSION(1,0,0), pEngineName: name.as_ptr(), engineVersion: VK_MAKE_VERSION(1,0,0), apiVersion: VK_API_VERSION_1_0 };
        let ci = VkInstanceCreateInfo { sType: VkStructureType::InstanceCreateInfo, pNext: std::ptr::null(), flags: 0, pApplicationInfo: &app, enabledLayerCount: 0, ppEnabledLayerNames: std::ptr::null(), enabledExtensionCount: 0, ppEnabledExtensionNames: std::ptr::null() };
        let mut instance = VkInstance::NULL;
        let r = kronos_compute::vkCreateInstance(&ci, std::ptr::null(), &mut instance);
        assert_eq!(r, VkResult::Success);

        // Enumerate devices
        let mut count = 0u32;
        kronos_compute::vkEnumeratePhysicalDevices(instance, &mut count, std::ptr::null_mut());
        if count == 0 { kronos_compute::vkDestroyInstance(instance, std::ptr::null()); return; }
        let mut phys = vec![VkPhysicalDevice::NULL; count as usize];
        kronos_compute::vkEnumeratePhysicalDevices(instance, &mut count, phys.as_mut_ptr());

        let threads = env::var("KRONOS_STRESS_THREADS").ok().and_then(|s| s.parse().ok()).unwrap_or(4usize)
            .min(phys.len().max(1));
        let iterations = env::var("KRONOS_STRESS_ITERS").ok().and_then(|s| s.parse().ok()).unwrap_or(3usize);
        let mut handles = Vec::new();
        for t in 0..threads {
            let phys_list = phys.clone();
            handles.push(thread::spawn(move || unsafe {
                for i in 0..iterations {
                    let pd = phys_list[(t + i) % phys_list.len()];
                    // Find compute queue family
                    let mut qf_count = 0u32;
                    kronos_compute::vkGetPhysicalDeviceQueueFamilyProperties(pd, &mut qf_count, std::ptr::null_mut());
                    if qf_count == 0 { continue; }
                    let mut qf = vec![VkQueueFamilyProperties { queueFlags: VkQueueFlags::empty(), queueCount: 0, timestampValidBits: 0, minImageTransferGranularity: VkExtent3D { width: 0, height: 0, depth: 0 } }; qf_count as usize];
                    kronos_compute::vkGetPhysicalDeviceQueueFamilyProperties(pd, &mut qf_count, qf.as_mut_ptr());
                    let Some(idx) = qf.iter().position(|f| f.queueFlags.contains(VkQueueFlags::COMPUTE)) else { continue; };
                    let qidx = idx as u32;

                    // Create device
                    let prio = 1.0f32;
                    let qinfo = VkDeviceQueueCreateInfo { sType: VkStructureType::DeviceQueueCreateInfo, pNext: std::ptr::null(), flags: 0, queueFamilyIndex: qidx, queueCount: 1, pQueuePriorities: &prio };
                    let dinfo = VkDeviceCreateInfo { sType: VkStructureType::DeviceCreateInfo, pNext: std::ptr::null(), flags: 0, queueCreateInfoCount: 1, pQueueCreateInfos: &qinfo, enabledLayerCount: 0, ppEnabledLayerNames: std::ptr::null(), enabledExtensionCount: 0, ppEnabledExtensionNames: std::ptr::null(), pEnabledFeatures: std::ptr::null() };
                    let mut device = VkDevice::NULL;
                    if kronos_compute::vkCreateDevice(pd, &dinfo, std::ptr::null(), &mut device) != VkResult::Success { continue; }

                    // Queue
                    let mut queue = VkQueue::NULL;
                    kronos_compute::vkGetDeviceQueue(device, qidx, 0, &mut queue);
                    if queue.is_null() { kronos_compute::vkDestroyDevice(device, std::ptr::null()); continue; }

                    // Pool + CB
                    let pool_info = VkCommandPoolCreateInfo { sType: VkStructureType::CommandPoolCreateInfo, pNext: std::ptr::null(), flags: VkCommandPoolCreateFlags::empty(), queueFamilyIndex: qidx };
                    let mut pool = VkCommandPool::NULL;
                    if kronos_compute::vkCreateCommandPool(device, &pool_info, std::ptr::null(), &mut pool) != VkResult::Success { kronos_compute::vkDestroyDevice(device, std::ptr::null()); continue; }
                    let alloc = VkCommandBufferAllocateInfo { sType: VkStructureType::CommandBufferAllocateInfo, pNext: std::ptr::null(), commandPool: pool, level: VkCommandBufferLevel::Primary, commandBufferCount: 1 };
                    let mut cmd = VkCommandBuffer::NULL;
                    let _ = kronos_compute::vkAllocateCommandBuffers(device, &alloc, &mut cmd);

                    // Record + submit
                    let begin = VkCommandBufferBeginInfo { sType: VkStructureType::CommandBufferBeginInfo, pNext: std::ptr::null(), flags: VkCommandBufferUsageFlags::ONE_TIME_SUBMIT, pInheritanceInfo: std::ptr::null() };
                    if kronos_compute::vkBeginCommandBuffer(cmd, &begin) == VkResult::Success {
                        let _ = kronos_compute::vkEndCommandBuffer(cmd);
                        let submit = VkSubmitInfo { sType: VkStructureType::SubmitInfo, pNext: std::ptr::null(), waitSemaphoreCount: 0, pWaitSemaphores: std::ptr::null(), pWaitDstStageMask: std::ptr::null(), commandBufferCount: 1, pCommandBuffers: &cmd, signalSemaphoreCount: 0, pSignalSemaphores: std::ptr::null() };
                        let _ = kronos_compute::vkQueueSubmit(queue, 1, &submit, VkFence::NULL);
                        let _ = kronos_compute::vkQueueWaitIdle(queue);
                    }

                    // Cleanup
                    kronos_compute::vkDestroyCommandPool(device, pool, std::ptr::null());
                    kronos_compute::vkDestroyDevice(device, std::ptr::null());

                    // Stagger
                    thread::sleep(Duration::from_millis(10));
                }
            }));
        }
        for h in handles { let _ = h.join(); }
        kronos_compute::vkDestroyInstance(instance, std::ptr::null());
    }
}
