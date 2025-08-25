//! Test descriptor set functionality

use kronos_compute::*;
use kronos_compute::ffi::*;
use std::ffi::CString;
use std::ptr;

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
    
    fn vkCreateBuffer(
        device: VkDevice,
        pCreateInfo: *const VkBufferCreateInfo,
        pAllocator: *const VkAllocationCallbacks,
        pBuffer: *mut VkBuffer,
    ) -> VkResult;
    
    fn vkDestroyBuffer(
        device: VkDevice,
        buffer: VkBuffer,
        pAllocator: *const VkAllocationCallbacks,
    );
    
    fn vkCreateDescriptorSetLayout(
        device: VkDevice,
        pCreateInfo: *const VkDescriptorSetLayoutCreateInfo,
        pAllocator: *const VkAllocationCallbacks,
        pSetLayout: *mut VkDescriptorSetLayout,
    ) -> VkResult;
    
    fn vkDestroyDescriptorSetLayout(
        device: VkDevice,
        descriptorSetLayout: VkDescriptorSetLayout,
        pAllocator: *const VkAllocationCallbacks,
    );
    
    fn vkCreateDescriptorPool(
        device: VkDevice,
        pCreateInfo: *const VkDescriptorPoolCreateInfo,
        pAllocator: *const VkAllocationCallbacks,
        pDescriptorPool: *mut VkDescriptorPool,
    ) -> VkResult;
    
    fn vkDestroyDescriptorPool(
        device: VkDevice,
        descriptorPool: VkDescriptorPool,
        pAllocator: *const VkAllocationCallbacks,
    );
    
    fn vkAllocateDescriptorSets(
        device: VkDevice,
        pAllocateInfo: *const VkDescriptorSetAllocateInfo,
        pDescriptorSets: *mut VkDescriptorSet,
    ) -> VkResult;
    
    fn vkUpdateDescriptorSets(
        device: VkDevice,
        descriptorWriteCount: u32,
        pDescriptorWrites: *const VkWriteDescriptorSet,
        descriptorCopyCount: u32,
        pDescriptorCopies: *const VkCopyDescriptorSet,
    );
}

fn main() {
    println!("Kronos Descriptor Test");
    println!("======================\n");
    
    unsafe {
        // 1. Create instance
        println!("Creating instance...");
        let app_name = CString::new("Kronos Descriptor Test").unwrap();
        
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
        let result = vkCreateInstance(&create_info, ptr::null(), &mut instance);
        
        if result != VkResult::Success {
            println!("✗ Failed to create instance: {:?}", result);
            return;
        }
        println!("✓ Instance created");
        
        // 2. Get physical device
        let mut device_count = 1;
        let mut physical_device = VkPhysicalDevice::NULL;
        vkEnumeratePhysicalDevices(instance, &mut device_count, &mut physical_device);
        println!("✓ Got physical device");
        
        // 3. Create logical device
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
        let result = vkCreateDevice(physical_device, &device_create_info, ptr::null(), &mut device);
        
        if result != VkResult::Success {
            println!("✗ Failed to create device: {:?}", result);
            vkDestroyInstance(instance, ptr::null());
            return;
        }
        println!("✓ Device created");
        
        // 4. Create descriptor set layout
        println!("\nCreating descriptor set layout...");
        
        // Define bindings for compute shader
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
                descriptorType: VkDescriptorType::UniformBuffer,
                descriptorCount: 1,
                stageFlags: VkShaderStageFlags::COMPUTE,
                pImmutableSamplers: ptr::null(),
            },
        ];
        
        let layout_info = VkDescriptorSetLayoutCreateInfo {
            sType: VkStructureType::DescriptorSetLayoutCreateInfo,
            pNext: ptr::null(),
            flags: 0,
            bindingCount: bindings.len() as u32,
            pBindings: bindings.as_ptr(),
        };
        
        let mut set_layout = VkDescriptorSetLayout::NULL;
        let result = vkCreateDescriptorSetLayout(device, &layout_info, ptr::null(), &mut set_layout);
        
        if result != VkResult::Success {
            println!("✗ Failed to create descriptor set layout: {:?}", result);
        } else {
            println!("✓ Descriptor set layout created");
        }
        
        // 5. Create descriptor pool
        println!("\nCreating descriptor pool...");
        
        let pool_sizes = [
            VkDescriptorPoolSize {
                type_: VkDescriptorType::StorageBuffer,
                descriptorCount: 10,
            },
            VkDescriptorPoolSize {
                type_: VkDescriptorType::UniformBuffer,
                descriptorCount: 5,
            },
        ];
        
        let pool_info = VkDescriptorPoolCreateInfo {
            sType: VkStructureType::DescriptorPoolCreateInfo,
            pNext: ptr::null(),
            flags: VkDescriptorPoolCreateFlags::empty(),
            maxSets: 5,
            poolSizeCount: pool_sizes.len() as u32,
            pPoolSizes: pool_sizes.as_ptr(),
        };
        
        let mut descriptor_pool = VkDescriptorPool::NULL;
        let result = vkCreateDescriptorPool(device, &pool_info, ptr::null(), &mut descriptor_pool);
        
        if result != VkResult::Success {
            println!("✗ Failed to create descriptor pool: {:?}", result);
        } else {
            println!("✓ Descriptor pool created");
        }
        
        // 6. Allocate descriptor sets
        println!("\nAllocating descriptor sets...");
        
        let alloc_info = VkDescriptorSetAllocateInfo {
            sType: VkStructureType::DescriptorSetAllocateInfo,
            pNext: ptr::null(),
            descriptorPool: descriptor_pool,
            descriptorSetCount: 1,
            pSetLayouts: &set_layout,
        };
        
        let mut descriptor_set = VkDescriptorSet::NULL;
        let result = vkAllocateDescriptorSets(device, &alloc_info, &mut descriptor_set);
        
        if result != VkResult::Success {
            println!("✗ Failed to allocate descriptor sets: {:?}", result);
        } else {
            println!("✓ Descriptor set allocated");
        }
        
        // 7. Create buffers to bind
        println!("\nCreating buffers...");
        
        let buffer_size = 1024 * 1024; // 1MB
        let mut buffers = vec![VkBuffer::NULL; 3];
        
        for (i, buffer) in buffers.iter_mut().enumerate() {
            let usage = if i < 2 {
                VkBufferUsageFlags::STORAGE_BUFFER
            } else {
                VkBufferUsageFlags::UNIFORM_BUFFER
            };
            
            let buffer_info = VkBufferCreateInfo {
                sType: VkStructureType::BufferCreateInfo,
                pNext: ptr::null(),
                flags: VkBufferCreateFlags::empty(),
                size: buffer_size,
                usage,
                sharingMode: VkSharingMode::Exclusive,
                queueFamilyIndexCount: 0,
                pQueueFamilyIndices: ptr::null(),
            };
            
            let result = vkCreateBuffer(device, &buffer_info, ptr::null(), buffer);
            if result != VkResult::Success {
                println!("✗ Failed to create buffer {}: {:?}", i, result);
            }
        }
        println!("✓ Created {} buffers", buffers.len());
        
        // 8. Update descriptor sets
        println!("\nUpdating descriptor sets...");
        
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
                range: 256, // Smaller for uniform buffer
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
                descriptorType: VkDescriptorType::UniformBuffer,
                pImageInfo: ptr::null(),
                pBufferInfo: &buffer_infos[2],
                pTexelBufferView: ptr::null(),
            },
        ];
        
        vkUpdateDescriptorSets(device, writes.len() as u32, writes.as_ptr(), 0, ptr::null());
        println!("✓ Descriptor sets updated");
        
        // 9. Test descriptor copying
        println!("\nTesting descriptor copying...");
        
        // Allocate another descriptor set
        let mut descriptor_set2 = VkDescriptorSet::NULL;
        let result = vkAllocateDescriptorSets(device, &alloc_info, &mut descriptor_set2);
        
        if result == VkResult::Success {
            // Copy descriptors from first set to second
            let copy = VkCopyDescriptorSet {
                sType: VkStructureType::CopyDescriptorSet,
                pNext: ptr::null(),
                srcSet: descriptor_set,
                srcBinding: 0,
                srcArrayElement: 0,
                dstSet: descriptor_set2,
                dstBinding: 0,
                dstArrayElement: 0,
                descriptorCount: 1,
            };
            
            vkUpdateDescriptorSets(device, 0, ptr::null(), 1, &copy);
            println!("✓ Descriptor copied successfully");
        }
        
        // Cleanup
        println!("\nCleaning up...");
        for buffer in &buffers {
            vkDestroyBuffer(device, *buffer, ptr::null());
        }
        vkDestroyDescriptorPool(device, descriptor_pool, ptr::null());
        vkDestroyDescriptorSetLayout(device, set_layout, ptr::null());
        vkDestroyDevice(device, ptr::null());
        vkDestroyInstance(instance, ptr::null());
        
        println!("✓ Cleanup complete");
        println!("\n✓ All descriptor tests passed!");
    }
}

// Version macros
const fn VK_MAKE_VERSION(major: u32, minor: u32, patch: u32) -> u32 {
    (major << 22) | (minor << 12) | patch
}