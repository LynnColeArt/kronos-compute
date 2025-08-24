//! Test synchronization primitives

use kronos::*;
use kronos::ffi::*;
use std::ffi::CString;
use std::ptr;
use std::thread;
use std::time::{Duration, Instant};

// Import the implementation functions
extern "C" {
    fn vkCreateInstance(
        pCreateInfo: *const VkInstanceCreateInfo,
        pAllocator: *const VkAllocationCallbacks,
        pInstance: *mut VkInstance,
    ) -> VkResult;
    
    fn vkDestroyInstance(
        instance: VkInstance,
        pAllocator: *const VkAllocationCallbacks,
    );
    
    fn vkEnumeratePhysicalDevices(
        instance: VkInstance,
        pPhysicalDeviceCount: *mut u32,
        pPhysicalDevices: *mut VkPhysicalDevice,
    ) -> VkResult;
    
    fn vkCreateDevice(
        physicalDevice: VkPhysicalDevice,
        pCreateInfo: *const VkDeviceCreateInfo,
        pAllocator: *const VkAllocationCallbacks,
        pDevice: *mut VkDevice,
    ) -> VkResult;
    
    fn vkDestroyDevice(
        device: VkDevice,
        pAllocator: *const VkAllocationCallbacks,
    );
    
    fn vkCreateFence(
        device: VkDevice,
        pCreateInfo: *const VkFenceCreateInfo,
        pAllocator: *const VkAllocationCallbacks,
        pFence: *mut VkFence,
    ) -> VkResult;
    
    fn vkDestroyFence(
        device: VkDevice,
        fence: VkFence,
        pAllocator: *const VkAllocationCallbacks,
    );
    
    fn vkResetFences(
        device: VkDevice,
        fenceCount: u32,
        pFences: *const VkFence,
    ) -> VkResult;
    
    fn vkGetFenceStatus(
        device: VkDevice,
        fence: VkFence,
    ) -> VkResult;
    
    fn vkWaitForFences(
        device: VkDevice,
        fenceCount: u32,
        pFences: *const VkFence,
        waitAll: VkBool32,
        timeout: u64,
    ) -> VkResult;
    
    fn vkCreateSemaphore(
        device: VkDevice,
        pCreateInfo: *const VkSemaphoreCreateInfo,
        pAllocator: *const VkAllocationCallbacks,
        pSemaphore: *mut VkSemaphore,
    ) -> VkResult;
    
    fn vkDestroySemaphore(
        device: VkDevice,
        semaphore: VkSemaphore,
        pAllocator: *const VkAllocationCallbacks,
    );
    
    fn vkCreateEvent(
        device: VkDevice,
        pCreateInfo: *const VkEventCreateInfo,
        pAllocator: *const VkAllocationCallbacks,
        pEvent: *mut VkEvent,
    ) -> VkResult;
    
    fn vkDestroyEvent(
        device: VkDevice,
        event: VkEvent,
        pAllocator: *const VkAllocationCallbacks,
    );
    
    fn vkGetEventStatus(
        device: VkDevice,
        event: VkEvent,
    ) -> VkResult;
    
    fn vkSetEvent(
        device: VkDevice,
        event: VkEvent,
    ) -> VkResult;
    
    fn vkResetEvent(
        device: VkDevice,
        event: VkEvent,
    ) -> VkResult;
    
    fn vkGetDeviceQueue(
        device: VkDevice,
        queueFamilyIndex: u32,
        queueIndex: u32,
        pQueue: *mut VkQueue,
    );
    
    fn vkQueueSubmit(
        queue: VkQueue,
        submitCount: u32,
        pSubmits: *const VkSubmitInfo,
        fence: VkFence,
    ) -> VkResult;
}

fn main() {
    println!("Kronos Synchronization Test");
    println!("===========================\n");
    
    unsafe {
        // Setup instance and device
        let app_name = CString::new("Kronos Sync Test").unwrap();
        let app_info = VkApplicationInfo {
            sType: VkStructureType::ApplicationInfo,
            pNext: ptr::null(),
            pApplicationName: app_name.as_ptr(),
            applicationVersion: VK_MAKE_VERSION(1, 0, 0),
            pEngineName: ptr::null(),
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
        vkCreateInstance(&create_info, ptr::null(), &mut instance);
        
        let mut device_count = 1;
        let mut physical_device = VkPhysicalDevice::NULL;
        vkEnumeratePhysicalDevices(instance, &mut device_count, &mut physical_device);
        
        let queue_priorities = [1.0f32];
        let queue_create_info = VkDeviceQueueCreateInfo {
            sType: VkStructureType::DeviceQueueCreateInfo,
            pNext: ptr::null(),
            flags: 0,
            queueFamilyIndex: 0,
            queueCount: 1,
            pQueuePriorities: queue_priorities.as_ptr(),
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
        vkCreateDevice(physical_device, &device_create_info, ptr::null(), &mut device);
        
        let mut queue = VkQueue::NULL;
        vkGetDeviceQueue(device, 0, 0, &mut queue);
        
        println!("✓ Device setup complete\n");
        
        // Test 1: Fence operations
        println!("Testing fences...");
        
        // Create fence (initially unsignaled)
        let fence_info = VkFenceCreateInfo {
            sType: VkStructureType::FenceCreateInfo,
            pNext: ptr::null(),
            flags: VkFenceCreateFlags::empty(),
        };
        
        let mut fence = VkFence::NULL;
        let result = vkCreateFence(device, &fence_info, ptr::null(), &mut fence);
        assert_eq!(result, VkResult::Success);
        println!("✓ Created unsignaled fence");
        
        // Check status (should be unsignaled)
        let status = vkGetFenceStatus(device, fence);
        assert_eq!(status, VkResult::NotReady);
        println!("✓ Fence is initially unsignaled");
        
        // Create signaled fence
        let signaled_fence_info = VkFenceCreateInfo {
            sType: VkStructureType::FenceCreateInfo,
            pNext: ptr::null(),
            flags: VkFenceCreateFlags::SIGNALED,
        };
        
        let mut signaled_fence = VkFence::NULL;
        vkCreateFence(device, &signaled_fence_info, ptr::null(), &mut signaled_fence);
        
        let status = vkGetFenceStatus(device, signaled_fence);
        assert_eq!(status, VkResult::Success);
        println!("✓ Created signaled fence");
        
        // Test fence wait with timeout
        let start = Instant::now();
        let result = vkWaitForFences(device, 1, &fence, VK_TRUE, 100_000_000); // 100ms
        let elapsed = start.elapsed();
        assert_eq!(result, VkResult::Timeout);
        assert!(elapsed >= Duration::from_millis(100));
        println!("✓ Fence wait timeout works (waited {:?})", elapsed);
        
        // Submit work that signals fence
        let submit_info = VkSubmitInfo {
            sType: VkStructureType::SubmitInfo,
            pNext: ptr::null(),
            waitSemaphoreCount: 0,
            pWaitSemaphores: ptr::null(),
            pWaitDstStageMask: ptr::null(),
            commandBufferCount: 0,
            pCommandBuffers: ptr::null(),
            signalSemaphoreCount: 0,
            pSignalSemaphores: ptr::null(),
        };
        
        vkQueueSubmit(queue, 1, &submit_info, fence);
        
        // Now wait should succeed immediately
        let result = vkWaitForFences(device, 1, &fence, VK_TRUE, u64::MAX);
        assert_eq!(result, VkResult::Success);
        println!("✓ Fence signaled by queue submission");
        
        // Reset fence
        vkResetFences(device, 1, &fence);
        let status = vkGetFenceStatus(device, fence);
        assert_eq!(status, VkResult::NotReady);
        println!("✓ Fence reset to unsignaled");
        
        // Test 2: Semaphore operations
        println!("\nTesting semaphores...");
        
        let sem_info = VkSemaphoreCreateInfo::default();
        let mut sem1 = VkSemaphore::NULL;
        let mut sem2 = VkSemaphore::NULL;
        
        vkCreateSemaphore(device, &sem_info, ptr::null(), &mut sem1);
        vkCreateSemaphore(device, &sem_info, ptr::null(), &mut sem2);
        println!("✓ Created semaphores");
        
        // Test semaphore signaling through queue
        let submit1 = VkSubmitInfo {
            sType: VkStructureType::SubmitInfo,
            pNext: ptr::null(),
            waitSemaphoreCount: 0,
            pWaitSemaphores: ptr::null(),
            pWaitDstStageMask: ptr::null(),
            commandBufferCount: 0,
            pCommandBuffers: ptr::null(),
            signalSemaphoreCount: 1,
            pSignalSemaphores: &sem1,
        };
        
        vkQueueSubmit(queue, 1, &submit1, VkFence::NULL);
        println!("✓ Signaled semaphore via queue submission");
        
        // Submit that waits on sem1 and signals sem2
        let wait_stages = VkPipelineStageFlags::COMPUTE_SHADER;
        let submit2 = VkSubmitInfo {
            sType: VkStructureType::SubmitInfo,
            pNext: ptr::null(),
            waitSemaphoreCount: 1,
            pWaitSemaphores: &sem1,
            pWaitDstStageMask: &wait_stages,
            commandBufferCount: 0,
            pCommandBuffers: ptr::null(),
            signalSemaphoreCount: 1,
            pSignalSemaphores: &sem2,
        };
        
        vkQueueSubmit(queue, 1, &submit2, VkFence::NULL);
        println!("✓ Chained semaphore dependencies");
        
        // Test 3: Event operations
        println!("\nTesting events...");
        
        let event_info = VkEventCreateInfo::default();
        let mut event = VkEvent::NULL;
        
        vkCreateEvent(device, &event_info, ptr::null(), &mut event);
        
        // Check initial status
        let status = vkGetEventStatus(device, event);
        assert_eq!(status, VkResult::EventReset);
        println!("✓ Event initially reset");
        
        // Set event
        vkSetEvent(device, event);
        let status = vkGetEventStatus(device, event);
        assert_eq!(status, VkResult::EventSet);
        println!("✓ Event set successfully");
        
        // Reset event
        vkResetEvent(device, event);
        let status = vkGetEventStatus(device, event);
        assert_eq!(status, VkResult::EventReset);
        println!("✓ Event reset successfully");
        
        // Test 4: Multi-threading
        println!("\nTesting thread safety...");
        
        let fence_mt = {
            let mut f = VkFence::NULL;
            vkCreateFence(device, &fence_info, ptr::null(), &mut f);
            f
        };
        
        // Spawn thread that waits on fence
        let device_clone = device;
        let wait_thread = thread::spawn(move || {
            unsafe {
                let start = Instant::now();
                let result = vkWaitForFences(device_clone, 1, &fence_mt, VK_TRUE, u64::MAX);
                (result, start.elapsed())
            }
        });
        
        // Give thread time to start waiting
        thread::sleep(Duration::from_millis(50));
        
        // Signal fence from main thread
        vkQueueSubmit(queue, 1, &submit_info, fence_mt);
        
        // Join thread and check result
        let (result, elapsed) = wait_thread.join().unwrap();
        assert_eq!(result, VkResult::Success);
        println!("✓ Cross-thread fence signaling works (waited {:?})", elapsed);
        
        // Cleanup
        println!("\nCleaning up...");
        vkDestroyEvent(device, event, ptr::null());
        vkDestroySemaphore(device, sem1, ptr::null());
        vkDestroySemaphore(device, sem2, ptr::null());
        vkDestroyFence(device, fence, ptr::null());
        vkDestroyFence(device, signaled_fence, ptr::null());
        vkDestroyFence(device, fence_mt, ptr::null());
        vkDestroyDevice(device, ptr::null());
        vkDestroyInstance(instance, ptr::null());
        
        println!("✓ Cleanup complete");
        println!("\n✓ All synchronization tests passed!");
    }
}

// Version macros
const fn VK_MAKE_VERSION(major: u32, minor: u32, patch: u32) -> u32 {
    (major << 22) | (minor << 12) | patch
}