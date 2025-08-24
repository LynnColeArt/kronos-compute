//! Simple compute example using Kronos Rust API

use kronos::*;
use std::ffi::CString;
use std::ptr;

fn main() {
    println!("Kronos Rust Compute Example");
    println!("===========================");
    
    // This is a demonstration of the Rust API structure
    // In a real implementation, we would link to the Kronos loader
    
    unsafe {
        // 1. Create instance
        let app_name = CString::new("Kronos Rust Example").unwrap();
        let engine_name = CString::new("Kronos").unwrap();
        
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
        
        // In a real implementation, we would call:
        // let mut instance = VkInstance::NULL;
        // let result = vkCreateInstance(&create_info, ptr::null(), &mut instance);
        
        println!("Instance create info prepared");
        println!("  App: {:?}", app_name);
        println!("  Engine: {:?}", engine_name);
        println!("  API Version: {}.{}.{}", 
            (VK_API_VERSION_1_0 >> 22) & 0x3FF,
            (VK_API_VERSION_1_0 >> 12) & 0x3FF,
            VK_API_VERSION_1_0 & 0xFFF
        );
        
        // 2. Demonstrate queue family properties
        let queue_family = VkQueueFamilyProperties {
            queueFlags: VkQueueFlags::COMPUTE | VkQueueFlags::TRANSFER,
            queueCount: 1,
            timestampValidBits: 64,
            minImageTransferGranularity: VkExtent3D::default(),
        };
        
        println!("\nQueue family properties:");
        println!("  Flags: {:?}", queue_family.queueFlags);
        println!("  Count: {}", queue_family.queueCount);
        
        // 3. Demonstrate memory type cache
        let mem_cache = VkMemoryTypeCache {
            hostVisibleCoherent: 2,
            deviceLocal: 0,
            hostVisibleCached: 3,
            deviceLocalLazy: 1,
        };
        
        println!("\nMemory type cache (O(1) lookup):");
        println!("  Host Visible Coherent: type {}", mem_cache.hostVisibleCoherent);
        println!("  Device Local: type {}", mem_cache.deviceLocal);
        
        // 4. Demonstrate buffer creation
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
        
        println!("\nBuffer creation info:");
        println!("  Size: {} bytes", buffer_info.size);
        println!("  Usage: {:?}", buffer_info.usage);
        println!("  Structure size: {} bytes (optimized packing)", 
            std::mem::size_of::<VkBufferCreateInfo>());
        
        // 5. Demonstrate compute pipeline
        let shader_stage = VkPipelineShaderStageCreateInfo {
            sType: VkStructureType::PipelineShaderStageCreateInfo,
            pNext: ptr::null(),
            flags: 0,
            stage: VkShaderStageFlagBits::Compute,
            module: VkShaderModule::NULL,
            pName: b"main\0".as_ptr() as *const i8,
            pSpecializationInfo: ptr::null(),
        };
        
        let pipeline_info = VkComputePipelineCreateInfo {
            sType: VkStructureType::ComputePipelineCreateInfo,
            pNext: ptr::null(),
            flags: VkPipelineCreateFlags::empty(),
            stage: shader_stage,
            layout: VkPipelineLayout::NULL,
            basePipelineHandle: VkPipeline::NULL,
            basePipelineIndex: -1,
        };
        
        println!("\nCompute pipeline info:");
        println!("  Stage: {:?}", pipeline_info.stage.stage);
        println!("  Entry point: main");
        
        // 6. Show structure sizes comparison
        println!("\nStructure sizes (Kronos optimized):");
        println!("  VkPhysicalDeviceFeatures: {} bytes", 
            std::mem::size_of::<VkPhysicalDeviceFeatures>());
        println!("  VkBufferCreateInfo: {} bytes", 
            std::mem::size_of::<VkBufferCreateInfo>());
        println!("  VkComputePipelineCreateInfo: {} bytes", 
            std::mem::size_of::<VkComputePipelineCreateInfo>());
    }
    
    println!("\n✓ Kronos Rust API demonstration complete!");
}