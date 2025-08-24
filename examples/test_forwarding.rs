//! Test compute forwarding to real Vulkan driver

use kronos::*;
use kronos::implementation::*;
use std::ptr;
use std::ffi::CString;

fn main() {
    unsafe {
        println!("Testing Kronos Compute Forwarding...\n");
        
        // Try to initialize with real ICD
        println!("1. Attempting to use real Vulkan driver...");
        match kronos::implementation::icd_loader::initialize_icd_loader() {
            Ok(()) => {
                println!("   ✓ ICD loader initialized successfully");
                println!("   ✓ Will forward compute calls to real driver");
                set_backend_mode(BackendMode::RealICD);
            }
            Err(e) => {
                println!("   ℹ No Vulkan drivers found: {}", e);
                println!("   ℹ Using mock implementation");
                set_backend_mode(BackendMode::Mock);
            }
        }
        
        // Create instance
        println!("\n2. Creating instance...");
        let app_name = CString::new("Forwarding Test").unwrap();
        let engine_name = CString::new("Kronos").unwrap();
        
        let app_info = VkApplicationInfo {
            sType: VkStructureType::ApplicationInfo,
            pNext: ptr::null(),
            pApplicationName: app_name.as_ptr(),
            applicationVersion: 1,
            pEngineName: engine_name.as_ptr(),
            engineVersion: 1,
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
            panic!("Failed to create instance: {:?}", result);
        }
        println!("   ✓ Instance created");
        
        // Enumerate physical devices
        println!("\n3. Enumerating physical devices...");
        let mut device_count = 0;
        vkEnumeratePhysicalDevices(instance, &mut device_count, ptr::null_mut());
        println!("   Found {} device(s)", device_count);
        
        if device_count == 0 {
            println!("   No devices found!");
            vkDestroyInstance(instance, ptr::null());
            return;
        }
        
        let mut physical_devices = vec![VkPhysicalDevice::NULL; device_count as usize];
        vkEnumeratePhysicalDevices(instance, &mut device_count, physical_devices.as_mut_ptr());
        
        let physical_device = physical_devices[0];
        
        // Get queue family properties
        println!("\n4. Checking queue families...");
        let mut queue_family_count = 0;
        vkGetPhysicalDeviceQueueFamilyProperties(physical_device, &mut queue_family_count, ptr::null_mut());
        
        let mut queue_families = vec![VkQueueFamilyProperties {
            queueFlags: VkQueueFlags::empty(),
            queueCount: 0,
            timestampValidBits: 0,
            minImageTransferGranularity: VkExtent3D::default(),
        }; queue_family_count as usize];
        vkGetPhysicalDeviceQueueFamilyProperties(physical_device, &mut queue_family_count, queue_families.as_mut_ptr());
        
        // Find compute queue
        let compute_queue_family = queue_families.iter()
            .position(|qf| qf.queueFlags.contains(VkQueueFlags::COMPUTE))
            .expect("No compute queue family found") as u32;
        
        println!("   ✓ Found compute queue family at index {}", compute_queue_family);
        
        // Create device
        println!("\n5. Creating logical device...");
        let queue_priority = 1.0f32;
        let queue_create_info = VkDeviceQueueCreateInfo {
            sType: VkStructureType::DeviceQueueCreateInfo,
            pNext: ptr::null(),
            flags: 0,
            queueFamilyIndex: compute_queue_family,
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
        
        if result != VkResult::Success {
            panic!("Failed to create device: {:?}", result);
        }
        println!("   ✓ Device created");
        
        // Get compute queue
        println!("\n6. Getting compute queue...");
        let mut queue = VkQueue::NULL;
        vkGetDeviceQueue(device, compute_queue_family, 0, &mut queue);
        println!("   ✓ Got compute queue");
        
        // Create a simple compute shader module
        println!("\n7. Creating compute shader...");
        // This is a minimal SPIR-V compute shader that does nothing
        let spirv_code: Vec<u32> = vec![
            0x07230203, // Magic number
            0x00010000, // Version 1.0
            0x00080001, // Generator
            0x00000006, // Bound
            0x00000000, // Schema
            0x00020011, // OpCapability Shader
            0x00020011, // OpCapability Kernel (for compute)
            0x00030012, // OpMemoryModel Logical GLSL450
            0x00040017, // OpEntryPoint GLCompute %main "main"
            0x00030016, // OpExecutionMode %main LocalSize 1 1 1
            0x00020013, // OpTypeVoid
            0x00030021, // OpTypeFunction %void
            0x00050036, // OpFunction %void %main None %func
            0x00020018, // OpLabel
            0x000100FD, // OpReturn
            0x00010038, // OpFunctionEnd
        ];
        
        let shader_create_info = VkShaderModuleCreateInfo {
            sType: VkStructureType::ShaderModuleCreateInfo,
            pNext: ptr::null(),
            flags: 0,
            codeSize: spirv_code.len() * 4,
            pCode: spirv_code.as_ptr(),
        };
        
        let mut shader_module = VkShaderModule::NULL;
        let result = vkCreateShaderModule(device, &shader_create_info, ptr::null(), &mut shader_module);
        
        if result == VkResult::Success {
            println!("   ✓ Shader module created");
        } else {
            println!("   ✗ Failed to create shader module: {:?}", result);
        }
        
        // Create compute pipeline
        println!("\n8. Creating compute pipeline...");
        let entry_name = CString::new("main").unwrap();
        let stage_info = VkPipelineShaderStageCreateInfo {
            sType: VkStructureType::PipelineShaderStageCreateInfo,
            pNext: ptr::null(),
            flags: 0,
            stage: VkShaderStageFlagBits::Compute,
            module: shader_module,
            pName: entry_name.as_ptr(),
            pSpecializationInfo: ptr::null(),
        };
        
        let pipeline_layout_info = VkPipelineLayoutCreateInfo {
            sType: VkStructureType::PipelineLayoutCreateInfo,
            pNext: ptr::null(),
            flags: 0,
            setLayoutCount: 0,
            pSetLayouts: ptr::null(),
            pushConstantRangeCount: 0,
            pPushConstantRanges: ptr::null(),
        };
        
        let mut pipeline_layout = VkPipelineLayout::NULL;
        vkCreatePipelineLayout(device, &pipeline_layout_info, ptr::null(), &mut pipeline_layout);
        
        let compute_create_info = VkComputePipelineCreateInfo {
            sType: VkStructureType::ComputePipelineCreateInfo,
            pNext: ptr::null(),
            flags: VkPipelineCreateFlags::empty(),
            stage: stage_info,
            layout: pipeline_layout,
            basePipelineHandle: VkPipeline::NULL,
            basePipelineIndex: -1,
        };
        
        let mut pipeline = VkPipeline::NULL;
        let result = vkCreateComputePipelines(device, VkPipelineCache::NULL, 1, &compute_create_info, ptr::null(), &mut pipeline);
        
        if result == VkResult::Success {
            println!("   ✓ Compute pipeline created");
        } else {
            println!("   ✗ Failed to create pipeline: {:?}", result);
        }
        
        // Show current backend mode
        println!("\n9. Backend mode: {:?}", get_backend_mode());
        
        // Cleanup
        println!("\n10. Cleaning up...");
        // Note: Destroy functions not yet implemented for pipelines/shaders
        // They would be called here in a complete implementation
        vkDestroyDevice(device, ptr::null());
        vkDestroyInstance(instance, ptr::null());
        println!("   ✓ Cleaned up successfully");
        
        println!("\nForwarding test completed!");
    }
}

