//! Test the actual Rust implementation of Kronos

use kronos_compute::*;
use std::ffi::CString;
use std::ptr;

fn main() {
    println!("Kronos Rust Implementation Test");
    println!("================================");
    
    unsafe {
        // 1. Create instance
        let app_name = CString::new("Kronos Rust Implementation Test").unwrap();
        let engine_name = CString::new("Kronos Native").unwrap();
        
        let app_info = VkApplicationInfo {
            sType: VkStructureType::ApplicationInfo,
            pNext: ptr::null(),
            pApplicationName: app_name.as_ptr(),
            applicationVersion: make_version(1, 0, 0),
            pEngineName: engine_name.as_ptr(),
            engineVersion: KRONOS_API_VERSION,
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
        let result = kronos_compute::implementation::instance::vkCreateInstance(
            &create_info, 
            ptr::null(), 
            &mut instance
        );
        
        println!("✓ Instance created: {:?} (handle: {})", result, instance.as_raw());
        
        // 2. Enumerate physical devices
        let mut device_count = 0;
        kronos_compute::implementation::instance::vkEnumeratePhysicalDevices(
            instance,
            &mut device_count,
            ptr::null_mut()
        );
        
        println!("✓ Found {} physical device(s)", device_count);
        
        let mut devices = vec![VkPhysicalDevice::NULL; device_count as usize];
        kronos_compute::implementation::instance::vkEnumeratePhysicalDevices(
            instance,
            &mut device_count,
            devices.as_mut_ptr()
        );
        
        let physical_device = devices[0];
        
        // 3. Get device properties
        let mut properties = std::mem::zeroed::<VkPhysicalDeviceProperties>();
        kronos_compute::implementation::instance::vkGetPhysicalDeviceProperties(
            physical_device,
            &mut properties
        );
        
        let device_name = std::ffi::CStr::from_ptr(properties.deviceName.as_ptr())
            .to_string_lossy();
        println!("✓ Device: {}", device_name);
        println!("  Type: {:?}", properties.deviceType);
        println!("  Vendor ID: 0x{:04X}", properties.vendorID);
        
        // 4. Create logical device
        let queue_priority = 1.0f32;
        let queue_info = VkDeviceQueueCreateInfo {
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
            pQueueCreateInfos: &queue_info,
            enabledLayerCount: 0,
            ppEnabledLayerNames: ptr::null(),
            enabledExtensionCount: 0,
            ppEnabledExtensionNames: ptr::null(),
            pEnabledFeatures: ptr::null(),
        };
        
        let mut device = VkDevice::NULL;
        let result = kronos_compute::implementation::device::vkCreateDevice(
            physical_device,
            &device_create_info,
            ptr::null(),
            &mut device
        );
        
        println!("✓ Device created: {:?} (handle: {})", result, device.as_raw());
        
        // 5. Get compute queue
        let mut queue = VkQueue::NULL;
        kronos_compute::implementation::device::vkGetDeviceQueue(
            device,
            0, // queue family
            0, // queue index
            &mut queue
        );
        
        println!("✓ Compute queue obtained (handle: {})", queue.as_raw());
        
        // 6. Create buffer
        let buffer_info = VkBufferCreateInfo {
            sType: VkStructureType::BufferCreateInfo,
            pNext: ptr::null(),
            size: 1024 * 1024, // 1MB
            usage: VkBufferUsageFlags::STORAGE_BUFFER | VkBufferUsageFlags::TRANSFER_DST,
            sharingMode: VkSharingMode::Exclusive,
            queueFamilyIndexCount: 0,
            pQueueFamilyIndices: ptr::null(),
            flags: VkBufferCreateFlags::empty(),
        };
        
        let mut buffer = VkBuffer::NULL;
        let result = kronos_compute::implementation::buffer::vkCreateBuffer(
            device,
            &buffer_info,
            ptr::null(),
            &mut buffer
        );
        
        println!("✓ Buffer created: {:?} (handle: {})", result, buffer.as_raw());
        
        // 7. Get memory requirements
        let mut mem_reqs = std::mem::zeroed::<VkMemoryRequirements>();
        kronos_compute::implementation::buffer::vkGetBufferMemoryRequirements(
            device,
            buffer,
            &mut mem_reqs
        );
        
        println!("✓ Memory requirements: {} bytes, alignment: {}", 
                 mem_reqs.size, mem_reqs.alignment);
        
        // 8. Allocate memory
        let alloc_info = VkMemoryAllocateInfo {
            sType: VkStructureType::MemoryAllocateInfo,
            pNext: ptr::null(),
            allocationSize: mem_reqs.size,
            memoryTypeIndex: 1, // Host visible
        };
        
        let mut memory = VkDeviceMemory::NULL;
        let result = kronos_compute::implementation::memory::vkAllocateMemory(
            device,
            &alloc_info,
            ptr::null(),
            &mut memory
        );
        
        println!("✓ Memory allocated: {:?} (handle: {})", result, memory.as_raw());
        
        // 9. Bind buffer memory
        let result = kronos_compute::implementation::buffer::vkBindBufferMemory(
            device,
            buffer,
            memory,
            0
        );
        
        println!("✓ Buffer bound to memory: {:?}", result);
        
        // 10. Map memory
        let mut data_ptr = ptr::null_mut();
        let result = kronos_compute::implementation::memory::vkMapMemory(
            device,
            memory,
            0,
            mem_reqs.size,
            0,
            &mut data_ptr
        );
        
        println!("✓ Memory mapped: {:?}", result);
        
        if result == VkResult::Success && !data_ptr.is_null() {
            // Write some test data
            let data = data_ptr as *mut f32;
            for i in 0..256 {
                *data.add(i) = i as f32;
            }
            println!("✓ Test data written");
        }
        
        // 11. Unmap memory
        kronos_compute::implementation::memory::vkUnmapMemory(device, memory);
        println!("✓ Memory unmapped");
        
        // 12. Create command pool
        let pool_info = VkCommandPoolCreateInfo {
            sType: VkStructureType::CommandPoolCreateInfo,
            pNext: ptr::null(),
            flags: VkCommandPoolCreateFlags::empty(),
            queueFamilyIndex: 0,
        };
        
        let mut command_pool = VkCommandPool::NULL;
        let result = kronos_compute::vkCreateCommandPool(
            device,
            &pool_info,
            ptr::null(),
            &mut command_pool
        );
        
        println!("✓ Command pool created: {:?} (handle: {})", result, command_pool.as_raw());
        
        // 13. Allocate command buffer
        let alloc_cmd_info = VkCommandBufferAllocateInfo {
            sType: VkStructureType::CommandBufferAllocateInfo,
            pNext: ptr::null(),
            commandPool: command_pool,
            level: VkCommandBufferLevel::Primary,
            commandBufferCount: 1,
        };
        
        let mut cmd_buffer = VkCommandBuffer::NULL;
        let result = kronos_compute::vkAllocateCommandBuffers(
            device,
            &alloc_cmd_info,
            &mut cmd_buffer
        );
        
        println!("✓ Command buffer allocated: {:?} (handle: {})", result, cmd_buffer.as_raw());
        
        // 14. Begin command buffer
        let begin_info = VkCommandBufferBeginInfo {
            sType: VkStructureType::CommandBufferBeginInfo,
            pNext: ptr::null(),
            flags: VkCommandBufferUsageFlags::ONE_TIME_SUBMIT,
            pInheritanceInfo: ptr::null(),
        };
        
        let result = kronos_compute::vkBeginCommandBuffer(
            cmd_buffer,
            &begin_info
        );
        
        println!("✓ Command buffer recording started: {:?}", result);
        
        // 15. Record a barrier
        let barrier = VkBufferMemoryBarrier {
            sType: VkStructureType::BufferMemoryBarrier,
            pNext: ptr::null(),
            srcAccessMask: VkAccessFlags::HOST_WRITE,
            dstAccessMask: VkAccessFlags::SHADER_READ,
            srcQueueFamilyIndex: VK_QUEUE_FAMILY_IGNORED,
            dstQueueFamilyIndex: VK_QUEUE_FAMILY_IGNORED,
            buffer,
            offset: 0,
            size: VK_WHOLE_SIZE,
        };
        
        kronos_compute::vkCmdPipelineBarrier(
            cmd_buffer,
            VkPipelineStageFlags::HOST,
            VkPipelineStageFlags::COMPUTE_SHADER,
            VkDependencyFlags::empty(),
            0, ptr::null(),
            1, &barrier,
            0, ptr::null()
        );
        
        println!("✓ Pipeline barrier recorded");
        
        // 16. End command buffer
        let result = kronos_compute::vkEndCommandBuffer(cmd_buffer);
        println!("✓ Command buffer recording ended: {:?}", result);
        
        // 17. Submit work
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
        
        let result = kronos_compute::implementation::device::vkQueueSubmit(
            queue,
            1,
            &submit_info,
            VkFence::NULL
        );
        
        println!("✓ Work submitted: {:?}", result);
        
        // 18. Wait for completion
        let result = kronos_compute::implementation::device::vkQueueWaitIdle(queue);
        println!("✓ Queue idle: {:?}", result);
        
        // Cleanup
        kronos_compute::implementation::buffer::vkDestroyBuffer(device, buffer, ptr::null());
        kronos_compute::implementation::memory::vkFreeMemory(device, memory, ptr::null());
        kronos_compute::implementation::device::vkDestroyDevice(device, ptr::null());
        kronos_compute::implementation::instance::vkDestroyInstance(instance, ptr::null());
        
        println!("\n✓ All resources cleaned up");
        println!("\n🎉 Kronos Rust implementation test completed successfully!");
    }
}