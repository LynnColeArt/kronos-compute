//! Benchmark memory allocation and mapping operations

use criterion::{black_box, criterion_group, criterion_main, Criterion, BenchmarkId, Throughput};
use kronos::*;
use std::ffi::CString;
use std::ptr;

struct MemoryContext {
    instance: VkInstance,
    device: VkDevice,
    physical_device: VkPhysicalDevice,
    memory_properties: VkPhysicalDeviceMemoryProperties,
}

unsafe fn create_memory_context() -> Option<MemoryContext> {
    // Initialize Kronos
    let _ = kronos::initialize_kronos();
    
    // Create instance
    let app_name = CString::new("Memory Benchmark").unwrap();
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
    if vkCreateInstance(&create_info, ptr::null(), &mut instance) != VkResult::Success {
        return None;
    }
    
    // Get physical device
    let mut device_count = 0u32;
    vkEnumeratePhysicalDevices(instance, &mut device_count, ptr::null_mut());
    if device_count == 0 {
        vkDestroyInstance(instance, ptr::null());
        return None;
    }
    
    let mut physical_devices = vec![VkPhysicalDevice::NULL; device_count as usize];
    vkEnumeratePhysicalDevices(instance, &mut device_count, physical_devices.as_mut_ptr());
    let physical_device = physical_devices[0];
    
    // Get memory properties
    let mut memory_properties = VkPhysicalDeviceMemoryProperties::default();
    vkGetPhysicalDeviceMemoryProperties(physical_device, &mut memory_properties);
    
    // Create device
    let mut queue_family_count = 0u32;
    vkGetPhysicalDeviceQueueFamilyProperties(physical_device, &mut queue_family_count, ptr::null_mut());
    let mut queue_families = vec![VkQueueFamilyProperties::default(); queue_family_count as usize];
    vkGetPhysicalDeviceQueueFamilyProperties(physical_device, &mut queue_family_count, queue_families.as_mut_ptr());
    
    let compute_queue_family = queue_families.iter()
        .position(|qf| qf.queueFlags.contains(VkQueueFlags::COMPUTE))
        .unwrap_or(0) as u32;
    
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
    if vkCreateDevice(physical_device, &device_create_info, ptr::null(), &mut device) != VkResult::Success {
        vkDestroyInstance(instance, ptr::null());
        return None;
    }
    
    Some(MemoryContext {
        instance,
        device,
        physical_device,
        memory_properties,
    })
}

unsafe fn destroy_memory_context(ctx: MemoryContext) {
    vkDestroyDevice(ctx.device, ptr::null());
    vkDestroyInstance(ctx.instance, ptr::null());
}

/// Find memory type index
unsafe fn find_memory_type(
    memory_properties: &VkPhysicalDeviceMemoryProperties,
    type_filter: u32,
    properties: VkMemoryPropertyFlags,
) -> Option<u32> {
    for i in 0..memory_properties.memoryTypeCount {
        if (type_filter & (1 << i)) != 0 
            && memory_properties.memoryTypes[i as usize].propertyFlags.contains(properties) {
            return Some(i);
        }
    }
    None
}

/// Benchmark buffer creation
fn bench_buffer_creation(c: &mut Criterion) {
    unsafe {
        if let Some(ctx) = create_memory_context() {
            let mut group = c.benchmark_group("buffer_creation");
            
            for &size in &[1024, 1024 * 1024, 16 * 1024 * 1024] {
                group.throughput(Throughput::Bytes(size));
                group.bench_with_input(
                    BenchmarkId::new("size", size),
                    &size,
                    |b, &buffer_size| {
                        b.iter(|| {
                            let buffer_info = VkBufferCreateInfo {
                                sType: VkStructureType::BufferCreateInfo,
                                pNext: ptr::null(),
                                flags: VkBufferCreateFlags::empty(),
                                size: buffer_size,
                                usage: VkBufferUsageFlags::STORAGE_BUFFER | VkBufferUsageFlags::TRANSFER_DST,
                                sharingMode: VkSharingMode::Exclusive,
                                queueFamilyIndexCount: 0,
                                pQueueFamilyIndices: ptr::null(),
                            };
                            
                            let mut buffer = VkBuffer::NULL;
                            let result = vkCreateBuffer(ctx.device, &buffer_info, ptr::null(), &mut buffer);
                            
                            if result == VkResult::Success {
                                vkDestroyBuffer(ctx.device, buffer, ptr::null());
                            }
                            
                            black_box(result);
                        });
                    },
                );
            }
            group.finish();
            
            destroy_memory_context(ctx);
        }
    }
}

/// Benchmark memory allocation
fn bench_memory_allocation(c: &mut Criterion) {
    unsafe {
        if let Some(ctx) = create_memory_context() {
            // Create a buffer to get memory requirements
            let buffer_info = VkBufferCreateInfo {
                sType: VkStructureType::BufferCreateInfo,
                pNext: ptr::null(),
                flags: VkBufferCreateFlags::empty(),
                size: 1024 * 1024, // 1MB
                usage: VkBufferUsageFlags::STORAGE_BUFFER,
                sharingMode: VkSharingMode::Exclusive,
                queueFamilyIndexCount: 0,
                pQueueFamilyIndices: ptr::null(),
            };
            
            let mut buffer = VkBuffer::NULL;
            if vkCreateBuffer(ctx.device, &buffer_info, ptr::null(), &mut buffer) != VkResult::Success {
                destroy_memory_context(ctx);
                return;
            }
            
            let mut mem_requirements = VkMemoryRequirements::default();
            vkGetBufferMemoryRequirements(ctx.device, buffer, &mut mem_requirements);
            
            // Find suitable memory type
            let memory_type = find_memory_type(
                &ctx.memory_properties,
                mem_requirements.memoryTypeBits,
                VkMemoryPropertyFlags::DEVICE_LOCAL,
            );
            
            if let Some(memory_type_index) = memory_type {
                let mut group = c.benchmark_group("memory_allocation");
                
                for &size in &[1024 * 1024, 16 * 1024 * 1024, 64 * 1024 * 1024] {
                    group.throughput(Throughput::Bytes(size));
                    group.bench_with_input(
                        BenchmarkId::new("size", size),
                        &size,
                        |b, &alloc_size| {
                            b.iter(|| {
                                let alloc_info = VkMemoryAllocateInfo {
                                    sType: VkStructureType::MemoryAllocateInfo,
                                    pNext: ptr::null(),
                                    allocationSize: alloc_size,
                                    memoryTypeIndex: memory_type_index,
                                };
                                
                                let mut memory = VkDeviceMemory::NULL;
                                let result = vkAllocateMemory(ctx.device, &alloc_info, ptr::null(), &mut memory);
                                
                                if result == VkResult::Success {
                                    vkFreeMemory(ctx.device, memory, ptr::null());
                                }
                                
                                black_box(result);
                            });
                        },
                    );
                }
                group.finish();
            }
            
            vkDestroyBuffer(ctx.device, buffer, ptr::null());
            destroy_memory_context(ctx);
        }
    }
}

/// Benchmark memory mapping/unmapping
fn bench_memory_mapping(c: &mut Criterion) {
    unsafe {
        if let Some(ctx) = create_memory_context() {
            // Find host visible memory type
            let memory_type = find_memory_type(
                &ctx.memory_properties,
                !0u32,
                VkMemoryPropertyFlags::HOST_VISIBLE | VkMemoryPropertyFlags::HOST_COHERENT,
            );
            
            if let Some(memory_type_index) = memory_type {
                let allocation_size = 16 * 1024 * 1024; // 16MB
                
                let alloc_info = VkMemoryAllocateInfo {
                    sType: VkStructureType::MemoryAllocateInfo,
                    pNext: ptr::null(),
                    allocationSize: allocation_size,
                    memoryTypeIndex: memory_type_index,
                };
                
                let mut memory = VkDeviceMemory::NULL;
                if vkAllocateMemory(ctx.device, &alloc_info, ptr::null(), &mut memory) == VkResult::Success {
                    c.bench_function("memory_map_unmap", |b| {
                        b.iter(|| {
                            let mut data_ptr = ptr::null_mut();
                            let result = vkMapMemory(ctx.device, memory, 0, allocation_size, 0, &mut data_ptr);
                            
                            if result == VkResult::Success {
                                // Touch the memory
                                let data = data_ptr as *mut u8;
                                for i in (0..1024).step_by(64) {
                                    *data.add(i) = i as u8;
                                }
                                
                                vkUnmapMemory(ctx.device, memory);
                            }
                            
                            black_box(result);
                        });
                    });
                    
                    vkFreeMemory(ctx.device, memory, ptr::null());
                }
            }
            
            destroy_memory_context(ctx);
        }
    }
}

/// Benchmark buffer-memory binding
fn bench_buffer_binding(c: &mut Criterion) {
    unsafe {
        if let Some(ctx) = create_memory_context() {
            // Create multiple buffers
            let buffer_count = 100;
            let buffer_size = 1024 * 1024; // 1MB each
            
            let buffer_info = VkBufferCreateInfo {
                sType: VkStructureType::BufferCreateInfo,
                pNext: ptr::null(),
                flags: VkBufferCreateFlags::empty(),
                size: buffer_size,
                usage: VkBufferUsageFlags::STORAGE_BUFFER,
                sharingMode: VkSharingMode::Exclusive,
                queueFamilyIndexCount: 0,
                pQueueFamilyIndices: ptr::null(),
            };
            
            let mut buffers = Vec::new();
            let mut memories = Vec::new();
            
            // Create buffers and allocate memory
            for _ in 0..buffer_count {
                let mut buffer = VkBuffer::NULL;
                if vkCreateBuffer(ctx.device, &buffer_info, ptr::null(), &mut buffer) == VkResult::Success {
                    let mut mem_requirements = VkMemoryRequirements::default();
                    vkGetBufferMemoryRequirements(ctx.device, buffer, &mut mem_requirements);
                    
                    if let Some(memory_type_index) = find_memory_type(
                        &ctx.memory_properties,
                        mem_requirements.memoryTypeBits,
                        VkMemoryPropertyFlags::DEVICE_LOCAL,
                    ) {
                        let alloc_info = VkMemoryAllocateInfo {
                            sType: VkStructureType::MemoryAllocateInfo,
                            pNext: ptr::null(),
                            allocationSize: mem_requirements.size,
                            memoryTypeIndex: memory_type_index,
                        };
                        
                        let mut memory = VkDeviceMemory::NULL;
                        if vkAllocateMemory(ctx.device, &alloc_info, ptr::null(), &mut memory) == VkResult::Success {
                            buffers.push(buffer);
                            memories.push(memory);
                        } else {
                            vkDestroyBuffer(ctx.device, buffer, ptr::null());
                        }
                    } else {
                        vkDestroyBuffer(ctx.device, buffer, ptr::null());
                    }
                }
            }
            
            if !buffers.is_empty() {
                c.bench_function("buffer_memory_binding", |b| {
                    b.iter(|| {
                        for (buffer, memory) in buffers.iter().zip(memories.iter()) {
                            vkBindBufferMemory(ctx.device, *buffer, *memory, 0);
                        }
                        black_box(&buffers);
                    });
                });
            }
            
            // Cleanup
            for buffer in buffers {
                vkDestroyBuffer(ctx.device, buffer, ptr::null());
            }
            for memory in memories {
                vkFreeMemory(ctx.device, memory, ptr::null());
            }
            
            destroy_memory_context(ctx);
        }
    }
}

criterion_group!(
    benches,
    bench_buffer_creation,
    bench_memory_allocation,
    bench_memory_mapping,
    bench_buffer_binding
);
criterion_main!(benches);