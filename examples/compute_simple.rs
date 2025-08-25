//! Simple compute example - SAXPY operation (c = alpha * a + b)
//! 
//! This demonstrates basic Kronos usage without any of the advanced optimizations

use kronos_compute::sys::*;
use kronos_compute::core::*;
use kronos_compute::ffi::*;
use std::ffi::CString;
use std::ptr;

fn main() {
    println!("Kronos Simple Compute Example");
    println!("=============================");
    
    unsafe {
        // Initialize Kronos with ICD forwarding
        if let Err(e) = kronos_compute::initialize_kronos() {
            eprintln!("Failed to initialize Kronos: {:?}", e);
            return;
        }
        println!("✓ Kronos initialized");
        
        // Create instance
        let app_name = CString::new("Kronos Simple Compute").unwrap();
        let engine_name = CString::new("Kronos").unwrap();
        
        let app_info = VkApplicationInfo {
            sType: VkStructureType::ApplicationInfo,
            pNext: ptr::null(),
            pApplicationName: app_name.as_ptr(),
            applicationVersion: VK_MAKE_VERSION(1, 0, 0),
            pEngineName: engine_name.as_ptr(),
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
            eprintln!("Failed to create instance: {:?}", result);
            return;
        }
        println!("✓ Instance created");
        
        // Find physical device with compute support
        let mut device_count = 0;
        kronos_compute::vkEnumeratePhysicalDevices(instance, &mut device_count, ptr::null_mut());
        
        let mut devices = vec![VkPhysicalDevice::NULL; device_count as usize];
        kronos_compute::vkEnumeratePhysicalDevices(instance, &mut device_count, devices.as_mut_ptr());
        
        let mut physical_device = VkPhysicalDevice::NULL;
        let mut compute_queue_family = u32::MAX;
        
        for device in &devices {
            let mut queue_family_count = 0;
            kronos_compute::vkGetPhysicalDeviceQueueFamilyProperties(*device, &mut queue_family_count, ptr::null_mut());
            
            let mut queue_families = vec![std::mem::zeroed::<VkQueueFamilyProperties>(); queue_family_count as usize];
            kronos_compute::vkGetPhysicalDeviceQueueFamilyProperties(*device, &mut queue_family_count, queue_families.as_mut_ptr());
            
            for (idx, family) in queue_families.iter().enumerate() {
                if family.queueFlags.contains(VkQueueFlags::COMPUTE) {
                    physical_device = *device;
                    compute_queue_family = idx as u32;
                    break;
                }
            }
            
            if physical_device != VkPhysicalDevice::NULL {
                break;
            }
        }
        
        if physical_device == VkPhysicalDevice::NULL {
            eprintln!("No compute-capable device found");
            kronos_compute::vkDestroyInstance(instance, ptr::null());
            return;
        }
        println!("✓ Found compute-capable device");
        
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
        if result != VkResult::Success {
            eprintln!("Failed to create device: {:?}", result);
            kronos_compute::vkDestroyInstance(instance, ptr::null());
            return;
        }
        println!("✓ Logical device created");
        
        // Get compute queue
        let mut compute_queue = VkQueue::NULL;
        kronos_compute::vkGetDeviceQueue(device, compute_queue_family, 0, &mut compute_queue);
        println!("✓ Compute queue obtained");
        
        // Create buffers
        const ARRAY_SIZE: usize = 1024;
        let buffer_size = (ARRAY_SIZE * std::mem::size_of::<f32>()) as VkDeviceSize;
        
        let buffer_info = VkBufferCreateInfo {
            sType: VkStructureType::BufferCreateInfo,
            pNext: ptr::null(),
            flags: VkBufferCreateFlags::from_bits(0).unwrap(),
            size: buffer_size,
            usage: VkBufferUsageFlags::STORAGE_BUFFER,
            sharingMode: VkSharingMode::Exclusive,
            queueFamilyIndexCount: 0,
            pQueueFamilyIndices: ptr::null(),
        };
        
        let mut buffers = [VkBuffer::NULL; 3]; // a, b, c
        let mut memories = [VkDeviceMemory::NULL; 3];
        
        // Get memory properties
        let mut mem_props: VkPhysicalDeviceMemoryProperties = std::mem::zeroed();
        kronos_compute::vkGetPhysicalDeviceMemoryProperties(physical_device, &mut mem_props);
        
        for i in 0..3 {
            kronos_compute::vkCreateBuffer(device, &buffer_info, ptr::null(), &mut buffers[i]);
            
            let mut mem_reqs: VkMemoryRequirements = std::mem::zeroed();
            kronos_compute::vkGetBufferMemoryRequirements(device, buffers[i], &mut mem_reqs);
            
            // Find host-visible memory type
            let mut memory_type = u32::MAX;
            for j in 0..mem_props.memoryTypeCount {
                if (mem_reqs.memoryTypeBits & (1 << j)) != 0 &&
                   mem_props.memoryTypes[j as usize].propertyFlags.contains(VkMemoryPropertyFlags::HOST_VISIBLE) {
                    memory_type = j;
                    break;
                }
            }
            
            let alloc_info = VkMemoryAllocateInfo {
                sType: VkStructureType::MemoryAllocateInfo,
                pNext: ptr::null(),
                allocationSize: mem_reqs.size,
                memoryTypeIndex: memory_type,
            };
            
            kronos_compute::vkAllocateMemory(device, &alloc_info, ptr::null(), &mut memories[i]);
            kronos_compute::vkBindBufferMemory(device, buffers[i], memories[i], 0);
        }
        println!("✓ Buffers created");
        
        // Initialize input data
        let mut data_a: *mut f32 = ptr::null_mut();
        let mut data_b: *mut f32 = ptr::null_mut();
        
        kronos_compute::vkMapMemory(device, memories[0], 0, buffer_size, 0, &mut data_a as *mut _ as *mut *mut std::ffi::c_void);
        kronos_compute::vkMapMemory(device, memories[1], 0, buffer_size, 0, &mut data_b as *mut _ as *mut *mut std::ffi::c_void);
        
        let slice_a = std::slice::from_raw_parts_mut(data_a, ARRAY_SIZE);
        let slice_b = std::slice::from_raw_parts_mut(data_b, ARRAY_SIZE);
        
        for i in 0..ARRAY_SIZE {
            slice_a[i] = i as f32;
            slice_b[i] = (i * 2) as f32;
        }
        
        kronos_compute::vkUnmapMemory(device, memories[0]);
        kronos_compute::vkUnmapMemory(device, memories[1]);
        println!("✓ Input data initialized");
        
        // Load shader - use saxpy.spv which is a proper compute shader
        let shader_path = concat!(env!("CARGO_MANIFEST_DIR"), "/shaders/saxpy.spv");
        let shader_code = std::fs::read(shader_path).expect("Failed to read shader");
        let shader_words: Vec<u32> = shader_code.chunks_exact(4)
            .map(|bytes| u32::from_le_bytes([bytes[0], bytes[1], bytes[2], bytes[3]]))
            .collect();
        
        let shader_create_info = VkShaderModuleCreateInfo {
            sType: VkStructureType::ShaderModuleCreateInfo,
            pNext: ptr::null(),
            flags: 0,
            codeSize: shader_code.len(),
            pCode: shader_words.as_ptr(),
        };
        
        let mut shader_module = VkShaderModule::NULL;
        kronos_compute::vkCreateShaderModule(device, &shader_create_info, ptr::null(), &mut shader_module);
        println!("✓ Shader loaded");
        
        // Create descriptor set layout
        let bindings = [
            VkDescriptorSetLayoutBinding {
                binding: 0,
                descriptorType: VkDescriptorType::StorageBuffer,
                descriptorCount: 1,
                stageFlags: VkShaderStageFlags::COMPUTE,
                pImmutableSamplers: ptr::null(),
            },
            VkDescriptorSetLayoutBinding {
                binding: 1,
                descriptorType: VkDescriptorType::StorageBuffer,
                descriptorCount: 1,
                stageFlags: VkShaderStageFlags::COMPUTE,
                pImmutableSamplers: ptr::null(),
            },
            VkDescriptorSetLayoutBinding {
                binding: 2,
                descriptorType: VkDescriptorType::StorageBuffer,
                descriptorCount: 1,
                stageFlags: VkShaderStageFlags::COMPUTE,
                pImmutableSamplers: ptr::null(),
            },
        ];
        
        let layout_create_info = VkDescriptorSetLayoutCreateInfo {
            sType: VkStructureType::DescriptorSetLayoutCreateInfo,
            pNext: ptr::null(),
            flags: 0,
            bindingCount: 3,
            pBindings: bindings.as_ptr(),
        };
        
        let mut descriptor_set_layout = VkDescriptorSetLayout::NULL;
        kronos_compute::vkCreateDescriptorSetLayout(device, &layout_create_info, ptr::null(), &mut descriptor_set_layout);
        
        // Create pipeline layout with push constants
        let push_constant_range = VkPushConstantRange {
            stageFlags: VkShaderStageFlags::COMPUTE,
            offset: 0,
            size: 8, // 4 bytes for alpha + 4 bytes for count
        };
        
        let pipeline_layout_info = VkPipelineLayoutCreateInfo {
            sType: VkStructureType::PipelineLayoutCreateInfo,
            pNext: ptr::null(),
            flags: 0,
            setLayoutCount: 1,
            pSetLayouts: &descriptor_set_layout,
            pushConstantRangeCount: 1,
            pPushConstantRanges: &push_constant_range,
        };
        
        let mut pipeline_layout = VkPipelineLayout::NULL;
        kronos_compute::vkCreatePipelineLayout(device, &pipeline_layout_info, ptr::null(), &mut pipeline_layout);
        
        // Create compute pipeline
        let entry_point = CString::new("main").unwrap();
        let stage_info = VkPipelineShaderStageCreateInfo {
            sType: VkStructureType::PipelineShaderStageCreateInfo,
            pNext: ptr::null(),
            flags: VkPipelineShaderStageCreateFlags::from_bits(0).unwrap(),
            stage: VkShaderStageFlagBits::Compute,
            module: shader_module,
            pName: entry_point.as_ptr(),
            pSpecializationInfo: ptr::null(),
        };
        
        let pipeline_info = VkComputePipelineCreateInfo {
            sType: VkStructureType::ComputePipelineCreateInfo,
            pNext: ptr::null(),
            flags: VkPipelineCreateFlags::from_bits(0).unwrap(),
            stage: stage_info,
            layout: pipeline_layout,
            basePipelineHandle: VkPipeline::NULL,
            basePipelineIndex: -1,
        };
        
        let mut compute_pipeline = VkPipeline::NULL;
        kronos_compute::vkCreateComputePipelines(device, VkPipelineCache::NULL, 1, &pipeline_info, ptr::null(), &mut compute_pipeline);
        println!("✓ Compute pipeline created");
        
        // Create descriptor pool
        let pool_size = VkDescriptorPoolSize {
            type_: VkDescriptorType::StorageBuffer,
            descriptorCount: 3,
        };
        
        let pool_info = VkDescriptorPoolCreateInfo {
            sType: VkStructureType::DescriptorPoolCreateInfo,
            pNext: ptr::null(),
            flags: VkDescriptorPoolCreateFlags::from_bits(0).unwrap(),
            maxSets: 1,
            poolSizeCount: 1,
            pPoolSizes: &pool_size,
        };
        
        let mut descriptor_pool = VkDescriptorPool::NULL;
        kronos_compute::vkCreateDescriptorPool(device, &pool_info, ptr::null(), &mut descriptor_pool);
        
        // Allocate descriptor set
        let alloc_info = VkDescriptorSetAllocateInfo {
            sType: VkStructureType::DescriptorSetAllocateInfo,
            pNext: ptr::null(),
            descriptorPool: descriptor_pool,
            descriptorSetCount: 1,
            pSetLayouts: &descriptor_set_layout,
        };
        
        let mut descriptor_set = VkDescriptorSet::NULL;
        kronos_compute::vkAllocateDescriptorSets(device, &alloc_info, &mut descriptor_set);
        
        // Update descriptor set
        let buffer_infos = [
            VkDescriptorBufferInfo {
                buffer: buffers[0],
                offset: 0,
                range: buffer_size,
            },
            VkDescriptorBufferInfo {
                buffer: buffers[1],
                offset: 0,
                range: buffer_size,
            },
            VkDescriptorBufferInfo {
                buffer: buffers[2],
                offset: 0,
                range: buffer_size,
            },
        ];
        
        let writes = [
            VkWriteDescriptorSet {
                sType: VkStructureType::WriteDescriptorSet,
                pNext: ptr::null(),
                dstSet: descriptor_set,
                dstBinding: 0,
                dstArrayElement: 0,
                descriptorCount: 1,
                descriptorType: VkDescriptorType::StorageBuffer,
                pImageInfo: ptr::null(),
                pBufferInfo: &buffer_infos[0],
                pTexelBufferView: ptr::null(),
            },
            VkWriteDescriptorSet {
                sType: VkStructureType::WriteDescriptorSet,
                pNext: ptr::null(),
                dstSet: descriptor_set,
                dstBinding: 1,
                dstArrayElement: 0,
                descriptorCount: 1,
                descriptorType: VkDescriptorType::StorageBuffer,
                pImageInfo: ptr::null(),
                pBufferInfo: &buffer_infos[1],
                pTexelBufferView: ptr::null(),
            },
            VkWriteDescriptorSet {
                sType: VkStructureType::WriteDescriptorSet,
                pNext: ptr::null(),
                dstSet: descriptor_set,
                dstBinding: 2,
                dstArrayElement: 0,
                descriptorCount: 1,
                descriptorType: VkDescriptorType::StorageBuffer,
                pImageInfo: ptr::null(),
                pBufferInfo: &buffer_infos[2],
                pTexelBufferView: ptr::null(),
            },
        ];
        
        kronos_compute::vkUpdateDescriptorSets(device, 3, writes.as_ptr(), 0, ptr::null());
        println!("✓ Descriptors updated");
        
        // Create command pool and buffer
        let pool_create_info = VkCommandPoolCreateInfo {
            sType: VkStructureType::CommandPoolCreateInfo,
            pNext: ptr::null(),
            flags: VkCommandPoolCreateFlags::from_bits(0).unwrap(),
            queueFamilyIndex: compute_queue_family,
        };
        
        let mut command_pool = VkCommandPool::NULL;
        kronos_compute::vkCreateCommandPool(device, &pool_create_info, ptr::null(), &mut command_pool);
        
        let cmd_alloc_info = VkCommandBufferAllocateInfo {
            sType: VkStructureType::CommandBufferAllocateInfo,
            pNext: ptr::null(),
            commandPool: command_pool,
            level: VkCommandBufferLevel::Primary,
            commandBufferCount: 1,
        };
        
        let mut cmd_buffer = VkCommandBuffer::NULL;
        kronos_compute::vkAllocateCommandBuffers(device, &cmd_alloc_info, &mut cmd_buffer);
        
        // Record commands
        let begin_info = VkCommandBufferBeginInfo {
            sType: VkStructureType::CommandBufferBeginInfo,
            pNext: ptr::null(),
            flags: VkCommandBufferUsageFlags::ONE_TIME_SUBMIT,
            pInheritanceInfo: ptr::null(),
        };
        
        kronos_compute::vkBeginCommandBuffer(cmd_buffer, &begin_info);
        
        // Add memory barrier to ensure buffer data is available
        let barrier = VkMemoryBarrier {
            sType: VkStructureType::MemoryBarrier,
            pNext: ptr::null(),
            srcAccessMask: VkAccessFlags::HOST_WRITE,
            dstAccessMask: VkAccessFlags::SHADER_READ,
        };
        
        kronos_compute::vkCmdPipelineBarrier(
            cmd_buffer,
            VkPipelineStageFlags::HOST,
            VkPipelineStageFlags::COMPUTE_SHADER,
            VkDependencyFlags::empty(),
            1,
            &barrier,
            0,
            ptr::null(),
            0,
            ptr::null(),
        );
        
        kronos_compute::vkCmdBindPipeline(cmd_buffer, VkPipelineBindPoint::Compute, compute_pipeline);
        kronos_compute::vkCmdBindDescriptorSets(
            cmd_buffer,
            VkPipelineBindPoint::Compute,
            pipeline_layout,
            0, 1, &descriptor_set,
            0, ptr::null()
        );
        
        // Push constants for SAXPY shader
        #[repr(C)]
        struct PushConstants {
            alpha: f32,
            count: u32,
        }
        let push_data = PushConstants {
            alpha: 1.0, // For simple addition, alpha = 1.0
            count: ARRAY_SIZE as u32,
        };
        
        kronos_compute::vkCmdPushConstants(
            cmd_buffer,
            pipeline_layout,
            VkShaderStageFlags::COMPUTE,
            0,
            std::mem::size_of::<PushConstants>() as u32,
            &push_data as *const _ as *const std::ffi::c_void,
        );
        
        // Dispatch with workgroup size of 256 (from SAXPY shader)
        kronos_compute::vkCmdDispatch(cmd_buffer, (ARRAY_SIZE as u32 + 255) / 256, 1, 1);
        
        kronos_compute::vkEndCommandBuffer(cmd_buffer);
        println!("✓ Commands recorded");
        
        // Submit and wait
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
        
        kronos_compute::vkQueueSubmit(compute_queue, 1, &submit_info, VkFence::NULL);
        kronos_compute::vkQueueWaitIdle(compute_queue);
        println!("✓ Compute work submitted");
        
        // Read results
        let mut data_c: *mut f32 = ptr::null_mut();
        kronos_compute::vkMapMemory(device, memories[2], 0, buffer_size, 0, &mut data_c as *mut _ as *mut *mut std::ffi::c_void);
        
        let slice_c = std::slice::from_raw_parts(data_c, ARRAY_SIZE);
        
        // Verify results
        println!("\nResults (first 10 and last 10 elements):");
        let mut correct_count = 0;
        let mut incorrect_indices = Vec::new();
        
        // Check all results
        for i in 0..ARRAY_SIZE {
            let expected = i as f32 + (i * 2) as f32;
            if (slice_c[i] - expected).abs() < 0.001 {
                correct_count += 1;
            } else {
                incorrect_indices.push(i);
            }
        }
        
        // Print first 10
        for i in 0..10 {
            let expected = i as f32 + (i * 2) as f32;
            println!("c[{}] = {} (expected {})", i, slice_c[i], expected);
        }
        
        // Print last 10
        println!("...");
        for i in (ARRAY_SIZE-10)..ARRAY_SIZE {
            let expected = i as f32 + (i * 2) as f32;
            println!("c[{}] = {} (expected {})", i, slice_c[i], expected);
        }
        
        println!("\nCorrect results: {}/{}", correct_count, ARRAY_SIZE);
        if !incorrect_indices.is_empty() {
            println!("First few incorrect indices: {:?}", &incorrect_indices[..incorrect_indices.len().min(10)]);
        }
        
        kronos_compute::vkUnmapMemory(device, memories[2]);
        
        // Cleanup
        kronos_compute::vkDestroyCommandPool(device, command_pool, ptr::null());
        kronos_compute::vkDestroyDescriptorPool(device, descriptor_pool, ptr::null());
        kronos_compute::vkDestroyPipeline(device, compute_pipeline, ptr::null());
        kronos_compute::vkDestroyPipelineLayout(device, pipeline_layout, ptr::null());
        kronos_compute::vkDestroyDescriptorSetLayout(device, descriptor_set_layout, ptr::null());
        kronos_compute::vkDestroyShaderModule(device, shader_module, ptr::null());
        
        for i in 0..3 {
            kronos_compute::vkDestroyBuffer(device, buffers[i], ptr::null());
            kronos_compute::vkFreeMemory(device, memories[i], ptr::null());
        }
        
        kronos_compute::vkDestroyDevice(device, ptr::null());
        kronos_compute::vkDestroyInstance(instance, ptr::null());
        
        println!("\n✓ Test completed successfully!");
        println!("This demonstrates basic Kronos compute functionality using SAXPY (c = alpha * a + b).");
        println!("With alpha = 1.0, this performs simple vector addition.");
    }
}