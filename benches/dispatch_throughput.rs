//! Benchmark compute dispatch throughput

use criterion::{black_box, criterion_group, criterion_main, Criterion, BenchmarkId, Throughput};
use kronos::*;
use std::ffi::CString;
use std::ptr;

struct ComputeContext {
    instance: VkInstance,
    device: VkDevice,
    queue: VkQueue,
    command_pool: VkCommandPool,
    command_buffer: VkCommandBuffer,
    pipeline_layout: VkPipelineLayout,
}

unsafe fn create_compute_context() -> Option<ComputeContext> {
    // Initialize Kronos
    let _ = kronos::initialize_kronos();
    
    // Create instance
    let app_name = CString::new("Dispatch Benchmark").unwrap();
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
    
    // Find compute queue
    let mut queue_family_count = 0u32;
    vkGetPhysicalDeviceQueueFamilyProperties(physical_device, &mut queue_family_count, ptr::null_mut());
    let mut queue_families = vec![VkQueueFamilyProperties::default(); queue_family_count as usize];
    vkGetPhysicalDeviceQueueFamilyProperties(physical_device, &mut queue_family_count, queue_families.as_mut_ptr());
    
    let compute_queue_family = queue_families.iter()
        .position(|qf| qf.queueFlags.contains(VkQueueFlags::COMPUTE))? as u32;
    
    // Create device
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
    
    // Get queue
    let mut queue = VkQueue::NULL;
    vkGetDeviceQueue(device, compute_queue_family, 0, &mut queue);
    
    // Create command pool
    let pool_create_info = VkCommandPoolCreateInfo {
        sType: VkStructureType::CommandPoolCreateInfo,
        pNext: ptr::null(),
        flags: VkCommandPoolCreateFlags::RESET_COMMAND_BUFFER,
        queueFamilyIndex: compute_queue_family,
    };
    
    let mut command_pool = VkCommandPool::NULL;
    if vkCreateCommandPool(device, &pool_create_info, ptr::null(), &mut command_pool) != VkResult::Success {
        vkDestroyDevice(device, ptr::null());
        vkDestroyInstance(instance, ptr::null());
        return None;
    }
    
    // Allocate command buffer
    let alloc_info = VkCommandBufferAllocateInfo {
        sType: VkStructureType::CommandBufferAllocateInfo,
        pNext: ptr::null(),
        commandPool: command_pool,
        level: VkCommandBufferLevel::Primary,
        commandBufferCount: 1,
    };
    
    let mut command_buffer = VkCommandBuffer::NULL;
    if vkAllocateCommandBuffers(device, &alloc_info, &mut command_buffer) != VkResult::Success {
        vkDestroyCommandPool(device, command_pool, ptr::null());
        vkDestroyDevice(device, ptr::null());
        vkDestroyInstance(instance, ptr::null());
        return None;
    }
    
    // Create minimal pipeline layout
    let layout_create_info = VkPipelineLayoutCreateInfo {
        sType: VkStructureType::PipelineLayoutCreateInfo,
        pNext: ptr::null(),
        flags: 0,
        setLayoutCount: 0,
        pSetLayouts: ptr::null(),
        pushConstantRangeCount: 0,
        pPushConstantRanges: ptr::null(),
    };
    
    let mut pipeline_layout = VkPipelineLayout::NULL;
    if vkCreatePipelineLayout(device, &layout_create_info, ptr::null(), &mut pipeline_layout) != VkResult::Success {
        vkDestroyCommandPool(device, command_pool, ptr::null());
        vkDestroyDevice(device, ptr::null());
        vkDestroyInstance(instance, ptr::null());
        return None;
    }
    
    Some(ComputeContext {
        instance,
        device,
        queue,
        command_pool,
        command_buffer,
        pipeline_layout,
    })
}

unsafe fn destroy_compute_context(ctx: ComputeContext) {
    vkDestroyPipelineLayout(ctx.device, ctx.pipeline_layout, ptr::null());
    vkDestroyCommandPool(ctx.device, ctx.command_pool, ptr::null());
    vkDestroyDevice(ctx.device, ptr::null());
    vkDestroyInstance(ctx.instance, ptr::null());
}

/// Benchmark single dispatch recording
fn bench_single_dispatch(c: &mut Criterion) {
    unsafe {
        if let Some(ctx) = create_compute_context() {
            c.bench_function("single_dispatch_recording", |b| {
                b.iter(|| {
                    // Begin command buffer
                    let begin_info = VkCommandBufferBeginInfo {
                        sType: VkStructureType::CommandBufferBeginInfo,
                        pNext: ptr::null(),
                        flags: VkCommandBufferUsageFlags::ONE_TIME_SUBMIT,
                        pInheritanceInfo: ptr::null(),
                    };
                    
                    vkBeginCommandBuffer(ctx.command_buffer, &begin_info);
                    
                    // Record dispatch (would bind pipeline in real scenario)
                    vkCmdDispatch(ctx.command_buffer, 64, 64, 1);
                    
                    // End command buffer
                    vkEndCommandBuffer(ctx.command_buffer);
                    
                    black_box(ctx.command_buffer);
                });
            });
            
            destroy_compute_context(ctx);
        }
    }
}

/// Benchmark multiple dispatches in a single command buffer
fn bench_batch_dispatch(c: &mut Criterion) {
    unsafe {
        if let Some(ctx) = create_compute_context() {
            let mut group = c.benchmark_group("batch_dispatch_recording");
            
            for dispatch_count in [1, 10, 100, 1000].iter() {
                group.throughput(Throughput::Elements(*dispatch_count as u64));
                group.bench_with_input(
                    BenchmarkId::new("dispatches", dispatch_count),
                    dispatch_count,
                    |b, &count| {
                        b.iter(|| {
                            let begin_info = VkCommandBufferBeginInfo {
                                sType: VkStructureType::CommandBufferBeginInfo,
                                pNext: ptr::null(),
                                flags: VkCommandBufferUsageFlags::ONE_TIME_SUBMIT,
                                pInheritanceInfo: ptr::null(),
                            };
                            
                            vkBeginCommandBuffer(ctx.command_buffer, &begin_info);
                            
                            // Record multiple dispatches
                            for _ in 0..count {
                                vkCmdDispatch(ctx.command_buffer, 64, 64, 1);
                            }
                            
                            vkEndCommandBuffer(ctx.command_buffer);
                            
                            black_box(ctx.command_buffer);
                        });
                    },
                );
            }
            group.finish();
            
            destroy_compute_context(ctx);
        }
    }
}

/// Benchmark dispatch with barriers
fn bench_dispatch_with_barriers(c: &mut Criterion) {
    unsafe {
        if let Some(ctx) = create_compute_context() {
            c.bench_function("dispatch_with_barriers", |b| {
                b.iter(|| {
                    let begin_info = VkCommandBufferBeginInfo {
                        sType: VkStructureType::CommandBufferBeginInfo,
                        pNext: ptr::null(),
                        flags: VkCommandBufferUsageFlags::ONE_TIME_SUBMIT,
                        pInheritanceInfo: ptr::null(),
                    };
                    
                    vkBeginCommandBuffer(ctx.command_buffer, &begin_info);
                    
                    // Dispatch -> Barrier -> Dispatch pattern
                    vkCmdDispatch(ctx.command_buffer, 64, 64, 1);
                    
                    // Pipeline barrier
                    vkCmdPipelineBarrier(
                        ctx.command_buffer,
                        VkPipelineStageFlags::COMPUTE_SHADER,
                        VkPipelineStageFlags::COMPUTE_SHADER,
                        VkDependencyFlags::empty(),
                        0, ptr::null(),
                        0, ptr::null(),
                        0, ptr::null(),
                    );
                    
                    vkCmdDispatch(ctx.command_buffer, 64, 64, 1);
                    
                    vkEndCommandBuffer(ctx.command_buffer);
                    
                    black_box(ctx.command_buffer);
                });
            });
            
            destroy_compute_context(ctx);
        }
    }
}

/// Benchmark command buffer submission
fn bench_queue_submission(c: &mut Criterion) {
    unsafe {
        if let Some(ctx) = create_compute_context() {
            // Pre-record command buffer
            let begin_info = VkCommandBufferBeginInfo {
                sType: VkStructureType::CommandBufferBeginInfo,
                pNext: ptr::null(),
                flags: VkCommandBufferUsageFlags::empty(),
                pInheritanceInfo: ptr::null(),
            };
            
            vkBeginCommandBuffer(ctx.command_buffer, &begin_info);
            vkCmdDispatch(ctx.command_buffer, 64, 64, 1);
            vkEndCommandBuffer(ctx.command_buffer);
            
            // Create fence for synchronization
            let fence_info = VkFenceCreateInfo {
                sType: VkStructureType::FenceCreateInfo,
                pNext: ptr::null(),
                flags: VkFenceCreateFlags::empty(),
            };
            
            let mut fence = VkFence::NULL;
            if vkCreateFence(ctx.device, &fence_info, ptr::null(), &mut fence) == VkResult::Success {
                c.bench_function("queue_submission", |b| {
                    b.iter(|| {
                        let submit_info = VkSubmitInfo {
                            sType: VkStructureType::SubmitInfo,
                            pNext: ptr::null(),
                            waitSemaphoreCount: 0,
                            pWaitSemaphores: ptr::null(),
                            pWaitDstStageMask: ptr::null(),
                            commandBufferCount: 1,
                            pCommandBuffers: &ctx.command_buffer,
                            signalSemaphoreCount: 0,
                            pSignalSemaphores: ptr::null(),
                        };
                        
                        vkQueueSubmit(ctx.queue, 1, &submit_info, fence);
                        vkWaitForFences(ctx.device, 1, &fence, VK_TRUE, u64::MAX);
                        vkResetFences(ctx.device, 1, &fence);
                        
                        black_box(fence);
                    });
                });
                
                vkDestroyFence(ctx.device, fence, ptr::null());
            }
            
            destroy_compute_context(ctx);
        }
    }
}

criterion_group!(
    benches,
    bench_single_dispatch,
    bench_batch_dispatch,
    bench_dispatch_with_barriers,
    bench_queue_submission
);
criterion_main!(benches);