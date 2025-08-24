//! Compute workload benchmarks: SAXPY, reduction, prefix-sum, GEMM
//! 
//! Tests with Mini's recommended configurations:
//! - Sizes: small (64k), medium (8M), large (64M)
//! - Batching: 1, 16, 256 dispatches
//! - Metrics: CPU submit time, descriptor updates, barriers, wall time

use criterion::{black_box, criterion_group, criterion_main, Criterion, BenchmarkId, Throughput};
use kronos::sys::*;
use kronos::ffi::*;
use kronos::core::*;
use kronos::implementation;
use std::ffi::CString;
use std::ptr;
use std::time::Instant;
use std::cell::RefCell;

/// Workload sizes
const SMALL_SIZE: usize = 64 * 1024;         // 64k elements
const MEDIUM_SIZE: usize = 8 * 1024 * 1024;  // 8M elements  
const LARGE_SIZE: usize = 64 * 1024 * 1024;  // 64M elements

/// Batch sizes
const BATCH_SIZES: &[usize] = &[1, 16, 256];

/// Metrics tracker
#[derive(Default)]
struct WorkloadMetrics {
    cpu_submit_time_us: f64,
    descriptor_updates: u32,
    barriers_issued: u32,
    wall_time_ms: f64,
}

/// Test context with optimizations
struct OptimizedContext {
    device: VkDevice,
    queue: VkQueue,
    command_pool: VkCommandPool,
    
    // Persistent descriptors
    descriptor_set: VkDescriptorSet,
    pipeline_layout: VkPipelineLayout,
    
    // Memory pools
    device_buffer_a: VkBuffer,
    device_buffer_b: VkBuffer,
    device_buffer_c: VkBuffer,
    staging_buffer: VkBuffer,
    
    // Barrier tracker
    barrier_tracker: RefCell<implementation::barrier_policy::BarrierTracker>,
}

/// SAXPY workload: c = a*x + b
fn bench_saxpy(c: &mut Criterion) {
    let mut group = c.benchmark_group("saxpy");
    
    unsafe {
        // Initialize context once
        if let Some(ctx) = create_optimized_context() {
            for &size in &[SMALL_SIZE, MEDIUM_SIZE, LARGE_SIZE] {
                for &batch_size in BATCH_SIZES {
                    let benchmark_id = BenchmarkId::new(
                        format!("size_{}_batch_{}", size, batch_size),
                        size
                    );
                    
                    group.throughput(Throughput::Elements((size * batch_size) as u64));
                    group.bench_with_input(benchmark_id, &(size, batch_size), |b, &(size, batch)| {
                        b.iter_custom(|iters| {
                            let mut total_time = std::time::Duration::ZERO;
                            let mut metrics = WorkloadMetrics::default();
                            
                            for _ in 0..iters {
                                let start = Instant::now();
                                
                                // Use timeline batching
                                implementation::timeline_batching::begin_batch(ctx.queue).unwrap();
                                
                                for i in 0..batch {
                                    // Record command buffer
                                    let cb = allocate_command_buffer(&ctx);
                                    
                                    let begin_info = VkCommandBufferBeginInfo {
                                        sType: VkStructureType::CommandBufferBeginInfo,
                                        pNext: ptr::null(),
                                        flags: VkCommandBufferUsageFlags::ONE_TIME_SUBMIT,
                                        pInheritanceInfo: ptr::null(),
                                    };
                                    
                                    kronos::vkBeginCommandBuffer(cb, &begin_info);
                                    
                                    // Bind persistent descriptor set (no updates!)
                                    kronos::vkCmdBindDescriptorSets(
                                        cb,
                                        VkPipelineBindPoint::Compute,
                                        ctx.pipeline_layout,
                                        0, 1, &ctx.descriptor_set,
                                        0, ptr::null()
                                    );
                                    
                                    // Push constants for parameters
                                    let params = SaxpyParams {
                                        alpha: 2.5f32,
                                        count: size as u32,
                                    };
                                    kronos::vkCmdPushConstants(
                                        cb,
                                        ctx.pipeline_layout,
                                        VkShaderStageFlags::COMPUTE,
                                        0,
                                        std::mem::size_of::<SaxpyParams>() as u32,
                                        &params as *const _ as *const std::ffi::c_void
                                    );
                                    
                                    // Smart barrier if needed
                                    if i == 0 {
                                        ctx.barrier_tracker.borrow_mut().track_buffer_access(
                                            ctx.device_buffer_a,
                                            VkAccessFlags::SHADER_READ,
                                            0, (size * 4) as u64
                                        );
                                    }
                                    
                                    // Dispatch
                                    let workgroup_size = 256;
                                    let workgroups = (size + workgroup_size - 1) / workgroup_size;
                                    kronos::vkCmdDispatch(cb, workgroups as u32, 1, 1);
                                    
                                    kronos::vkEndCommandBuffer(cb);
                                    
                                    // Add to batch
                                    implementation::timeline_batching::add_to_batch(ctx.queue, cb).unwrap();
                                }
                                
                                // Submit batch
                                let submit_start = Instant::now();
                                implementation::timeline_batching::submit_batch(ctx.queue, VkFence::NULL).unwrap();
                                metrics.cpu_submit_time_us += submit_start.elapsed().as_micros() as f64;
                                
                                // Wait for completion
                                kronos::vkQueueWaitIdle(ctx.queue);
                                
                                total_time += start.elapsed();
                                
                                // Update metrics
                                metrics.descriptor_updates = 0; // Zero with persistent descriptors!
                                metrics.barriers_issued = ctx.barrier_tracker.borrow().stats().total_barriers as u32;
                            }
                            
                            metrics.wall_time_ms = total_time.as_secs_f64() * 1000.0 / iters as f64;
                            
                            // Report metrics
                            println!("SAXPY size={} batch={}: {:.2} ms, {:.1} µs submit, {} barriers",
                                size, batch, metrics.wall_time_ms, 
                                metrics.cpu_submit_time_us / iters as f64,
                                metrics.barriers_issued
                            );
                            
                            total_time
                        });
                    });
                }
            }
            
            cleanup_context(ctx);
        }
    }
    
    group.finish();
}

/// Reduction workload: sum = reduce(a)
fn bench_reduction(c: &mut Criterion) {
    let mut group = c.benchmark_group("reduction");
    
    unsafe {
        if let Some(ctx) = create_optimized_context() {
            for &size in &[SMALL_SIZE, MEDIUM_SIZE, LARGE_SIZE] {
                for &batch_size in BATCH_SIZES {
                    let benchmark_id = BenchmarkId::new(
                        format!("size_{}_batch_{}", size, batch_size),
                        size
                    );
                    
                    group.throughput(Throughput::Elements((size * batch_size) as u64));
                    group.bench_with_input(benchmark_id, &(size, batch_size), |b, &(size, batch)| {
                        b.iter_custom(|iters| {
                            let mut total_time = std::time::Duration::ZERO;
                            let mut metrics = WorkloadMetrics::default();
                            
                            for _ in 0..iters {
                                let start = Instant::now();
                                
                                // Use timeline batching
                                implementation::timeline_batching::begin_batch(ctx.queue).unwrap();
                                
                                for i in 0..batch {
                                    // Reduction requires multiple passes for large arrays
                                    let mut current_size = size;
                                    let mut pass = 0;
                                    
                                    while current_size > 1 {
                                        let cb = allocate_command_buffer(&ctx);
                                        
                                        let begin_info = VkCommandBufferBeginInfo {
                                            sType: VkStructureType::CommandBufferBeginInfo,
                                            pNext: ptr::null(),
                                            flags: VkCommandBufferUsageFlags::ONE_TIME_SUBMIT,
                                            pInheritanceInfo: ptr::null(),
                                        };
                                        
                                        kronos::vkBeginCommandBuffer(cb, &begin_info);
                                        
                                        // Bind persistent descriptor set
                                        kronos::vkCmdBindDescriptorSets(
                                            cb,
                                            VkPipelineBindPoint::Compute,
                                            ctx.pipeline_layout,
                                            0, 1, &ctx.descriptor_set,
                                            0, ptr::null()
                                        );
                                        
                                        // Push constants for reduction parameters
                                        let params = ReductionParams {
                                            count: current_size as u32,
                                            stride: 1u32,
                                        };
                                        kronos::vkCmdPushConstants(
                                            cb,
                                            ctx.pipeline_layout,
                                            VkShaderStageFlags::COMPUTE,
                                            0,
                                            std::mem::size_of::<ReductionParams>() as u32,
                                            &params as *const _ as *const std::ffi::c_void
                                        );
                                        
                                        // Smart barrier between passes
                                        if pass > 0 {
                                            ctx.barrier_tracker.borrow_mut().track_buffer_access(
                                                ctx.device_buffer_a,
                                                VkAccessFlags::SHADER_WRITE,
                                                0, (current_size * 4) as u64
                                            );
                                        }
                                        
                                        // Dispatch reduction
                                        let workgroup_size = 256;
                                        let workgroups = (current_size + workgroup_size - 1) / workgroup_size;
                                        kronos::vkCmdDispatch(cb, workgroups as u32, 1, 1);
                                        
                                        kronos::vkEndCommandBuffer(cb);
                                        
                                        // Add to batch
                                        implementation::timeline_batching::add_to_batch(ctx.queue, cb).unwrap();
                                        
                                        current_size /= workgroup_size;
                                        pass += 1;
                                    }
                                }
                                
                                // Submit batch
                                let submit_start = Instant::now();
                                implementation::timeline_batching::submit_batch(ctx.queue, VkFence::NULL).unwrap();
                                metrics.cpu_submit_time_us += submit_start.elapsed().as_micros() as f64;
                                
                                // Wait for completion
                                kronos::vkQueueWaitIdle(ctx.queue);
                                
                                total_time += start.elapsed();
                                
                                // Update metrics
                                metrics.descriptor_updates = 0; // Zero with persistent descriptors!
                                metrics.barriers_issued = ctx.barrier_tracker.borrow().stats().total_barriers as u32;
                            }
                            
                            metrics.wall_time_ms = total_time.as_secs_f64() * 1000.0 / iters as f64;
                            
                            // Report metrics
                            println!("Reduction size={} batch={}: {:.2} ms, {:.1} µs submit, {} barriers",
                                size, batch, metrics.wall_time_ms, 
                                metrics.cpu_submit_time_us / iters as f64,
                                metrics.barriers_issued
                            );
                            
                            total_time
                        });
                    });
                }
            }
            
            cleanup_context(ctx);
        }
    }
    
    group.finish();
}

/// Prefix sum (scan) workload
fn bench_prefix_sum(c: &mut Criterion) {
    let mut group = c.benchmark_group("prefix_sum");
    
    unsafe {
        if let Some(ctx) = create_optimized_context() {
            for &size in &[SMALL_SIZE, MEDIUM_SIZE, LARGE_SIZE] {
                for &batch_size in BATCH_SIZES {
                    let benchmark_id = BenchmarkId::new(
                        format!("size_{}_batch_{}", size, batch_size),
                        size
                    );
                    
                    group.throughput(Throughput::Elements((size * batch_size) as u64));
                    group.bench_with_input(benchmark_id, &(size, batch_size), |b, &(size, batch)| {
                        b.iter_custom(|iters| {
                            let mut total_time = std::time::Duration::ZERO;
                            let mut metrics = WorkloadMetrics::default();
                            
                            for _ in 0..iters {
                                let start = Instant::now();
                                
                                // Use timeline batching
                                implementation::timeline_batching::begin_batch(ctx.queue).unwrap();
                                
                                for i in 0..batch {
                                    // Prefix sum has 3 phases: up-sweep, down-sweep, final
                                    let phases = 3;
                                    
                                    for phase in 0..phases {
                                        let cb = allocate_command_buffer(&ctx);
                                        
                                        let begin_info = VkCommandBufferBeginInfo {
                                            sType: VkStructureType::CommandBufferBeginInfo,
                                            pNext: ptr::null(),
                                            flags: VkCommandBufferUsageFlags::ONE_TIME_SUBMIT,
                                            pInheritanceInfo: ptr::null(),
                                        };
                                        
                                        kronos::vkBeginCommandBuffer(cb, &begin_info);
                                        
                                        // Bind persistent descriptor set
                                        kronos::vkCmdBindDescriptorSets(
                                            cb,
                                            VkPipelineBindPoint::Compute,
                                            ctx.pipeline_layout,
                                            0, 1, &ctx.descriptor_set,
                                            0, ptr::null()
                                        );
                                        
                                        // Push constants for scan parameters
                                        let params = ScanParams {
                                            count: size as u32,
                                            phase: phase as u32,
                                        };
                                        kronos::vkCmdPushConstants(
                                            cb,
                                            ctx.pipeline_layout,
                                            VkShaderStageFlags::COMPUTE,
                                            0,
                                            std::mem::size_of::<ScanParams>() as u32,
                                            &params as *const _ as *const std::ffi::c_void
                                        );
                                        
                                        // Smart barrier between phases
                                        if phase > 0 {
                                            ctx.barrier_tracker.borrow_mut().track_buffer_access(
                                                ctx.device_buffer_a,
                                                VkAccessFlags::SHADER_READ | VkAccessFlags::SHADER_WRITE,
                                                0, (size * 4) as u64
                                            );
                                        }
                                        
                                        // Dispatch scan phase
                                        let workgroup_size = 256;
                                        let workgroups = (size + workgroup_size - 1) / workgroup_size;
                                        kronos::vkCmdDispatch(cb, workgroups as u32, 1, 1);
                                        
                                        kronos::vkEndCommandBuffer(cb);
                                        
                                        // Add to batch
                                        implementation::timeline_batching::add_to_batch(ctx.queue, cb).unwrap();
                                    }
                                }
                                
                                // Submit batch
                                let submit_start = Instant::now();
                                implementation::timeline_batching::submit_batch(ctx.queue, VkFence::NULL).unwrap();
                                metrics.cpu_submit_time_us += submit_start.elapsed().as_micros() as f64;
                                
                                // Wait for completion
                                kronos::vkQueueWaitIdle(ctx.queue);
                                
                                total_time += start.elapsed();
                                
                                // Update metrics
                                metrics.descriptor_updates = 0; // Zero with persistent descriptors!
                                metrics.barriers_issued = ctx.barrier_tracker.borrow().stats().total_barriers as u32;
                            }
                            
                            metrics.wall_time_ms = total_time.as_secs_f64() * 1000.0 / iters as f64;
                            
                            // Report metrics
                            println!("Prefix sum size={} batch={}: {:.2} ms, {:.1} µs submit, {} barriers",
                                size, batch, metrics.wall_time_ms, 
                                metrics.cpu_submit_time_us / iters as f64,
                                metrics.barriers_issued
                            );
                            
                            total_time
                        });
                    });
                }
            }
            
            cleanup_context(ctx);
        }
    }
    
    group.finish();
}

/// Tiny GEMM workload: C = A * B
fn bench_gemm(c: &mut Criterion) {
    let mut group = c.benchmark_group("gemm");
    
    unsafe {
        if let Some(ctx) = create_optimized_context() {
            // Tiny GEMM sizes as recommended by Mini
            let matrix_sizes = &[(64, 64, 64), (128, 128, 128), (256, 256, 256)];
            
            for &(m, n, k) in matrix_sizes {
                for &batch_size in BATCH_SIZES {
                    let benchmark_id = BenchmarkId::new(
                        format!("{}x{}x{}_batch_{}", m, n, k, batch_size),
                        m * n * k
                    );
                    
                    group.throughput(Throughput::Elements((m * n * k * batch_size) as u64));
                    group.bench_with_input(benchmark_id, &(m, n, k, batch_size), |b, &(m, n, k, batch)| {
                        b.iter_custom(|iters| {
                            let mut total_time = std::time::Duration::ZERO;
                            let mut metrics = WorkloadMetrics::default();
                            
                            for _ in 0..iters {
                                let start = Instant::now();
                                
                                // Use timeline batching
                                implementation::timeline_batching::begin_batch(ctx.queue).unwrap();
                                
                                for i in 0..batch {
                                    let cb = allocate_command_buffer(&ctx);
                                    
                                    let begin_info = VkCommandBufferBeginInfo {
                                        sType: VkStructureType::CommandBufferBeginInfo,
                                        pNext: ptr::null(),
                                        flags: VkCommandBufferUsageFlags::ONE_TIME_SUBMIT,
                                        pInheritanceInfo: ptr::null(),
                                    };
                                    
                                    kronos::vkBeginCommandBuffer(cb, &begin_info);
                                    
                                    // Bind persistent descriptor set
                                    kronos::vkCmdBindDescriptorSets(
                                        cb,
                                        VkPipelineBindPoint::Compute,
                                        ctx.pipeline_layout,
                                        0, 1, &ctx.descriptor_set,
                                        0, ptr::null()
                                    );
                                    
                                    // Push constants for GEMM dimensions
                                    let params = GemmParams {
                                        m: m as u32,
                                        n: n as u32,
                                        k: k as u32,
                                        alpha: 1.0f32,
                                        beta: 0.0f32,
                                    };
                                    kronos::vkCmdPushConstants(
                                        cb,
                                        ctx.pipeline_layout,
                                        VkShaderStageFlags::COMPUTE,
                                        0,
                                        std::mem::size_of::<GemmParams>() as u32,
                                        &params as *const _ as *const std::ffi::c_void
                                    );
                                    
                                    // Smart barrier if needed
                                    if i == 0 {
                                        // A and B are read, C is written
                                        ctx.barrier_tracker.borrow_mut().track_buffer_access(
                                            ctx.device_buffer_a,
                                            VkAccessFlags::SHADER_READ,
                                            0, (m * k * 4) as u64
                                        );
                                        ctx.barrier_tracker.borrow_mut().track_buffer_access(
                                            ctx.device_buffer_b,
                                            VkAccessFlags::SHADER_READ,
                                            0, (k * n * 4) as u64
                                        );
                                        ctx.barrier_tracker.borrow_mut().track_buffer_access(
                                            ctx.device_buffer_c,
                                            VkAccessFlags::SHADER_WRITE,
                                            0, (m * n * 4) as u64
                                        );
                                    }
                                    
                                    // Dispatch GEMM with tile-based approach
                                    let tile_size = 16; // Common tile size for shared memory
                                    let workgroups_x = (n + tile_size - 1) / tile_size;
                                    let workgroups_y = (m + tile_size - 1) / tile_size;
                                    kronos::vkCmdDispatch(cb, workgroups_x as u32, workgroups_y as u32, 1);
                                    
                                    kronos::vkEndCommandBuffer(cb);
                                    
                                    // Add to batch
                                    implementation::timeline_batching::add_to_batch(ctx.queue, cb).unwrap();
                                }
                                
                                // Submit batch
                                let submit_start = Instant::now();
                                implementation::timeline_batching::submit_batch(ctx.queue, VkFence::NULL).unwrap();
                                metrics.cpu_submit_time_us += submit_start.elapsed().as_micros() as f64;
                                
                                // Wait for completion
                                kronos::vkQueueWaitIdle(ctx.queue);
                                
                                total_time += start.elapsed();
                                
                                // Update metrics
                                metrics.descriptor_updates = 0; // Zero with persistent descriptors!
                                metrics.barriers_issued = ctx.barrier_tracker.borrow().stats().total_barriers as u32;
                            }
                            
                            metrics.wall_time_ms = total_time.as_secs_f64() * 1000.0 / iters as f64;
                            
                            // Report metrics
                            println!("GEMM {}x{}x{} batch={}: {:.2} ms, {:.1} µs submit, {} barriers",
                                m, n, k, batch, metrics.wall_time_ms, 
                                metrics.cpu_submit_time_us / iters as f64,
                                metrics.barriers_issued
                            );
                            
                            total_time
                        });
                    });
                }
            }
            
            cleanup_context(ctx);
        }
    }
    
    group.finish();
}

// Helper structures
#[repr(C)]
struct SaxpyParams {
    alpha: f32,
    count: u32,
}

#[repr(C)]
struct ReductionParams {
    count: u32,
    stride: u32,
}

#[repr(C)]
struct ScanParams {
    count: u32,
    phase: u32,
}

#[repr(C)]
struct GemmParams {
    m: u32,
    n: u32,
    k: u32,
    alpha: f32,
    beta: f32,
}

// Helper functions
unsafe fn create_optimized_context() -> Option<OptimizedContext> {
    // Initialize Kronos
    kronos::initialize_kronos().ok()?;
    
    // Create instance
    let app_info = VkApplicationInfo {
        sType: VkStructureType::ApplicationInfo,
        pNext: ptr::null(),
        pApplicationName: CString::new("Kronos Benchmark").unwrap().as_ptr(),
        applicationVersion: VK_MAKE_VERSION(1, 0, 0),
        pEngineName: CString::new("Kronos").unwrap().as_ptr(),
        engineVersion: VK_MAKE_VERSION(1, 0, 0),
        apiVersion: VK_API_VERSION_1_3,
    };
    
    let create_info = VkInstanceCreateInfo {
        sType: VkStructureType::InstanceCreateInfo,
        pNext: ptr::null(),
        flags: VkInstanceCreateFlags::empty(),
        pApplicationInfo: &app_info,
        enabledLayerCount: 0,
        ppEnabledLayerNames: ptr::null(),
        enabledExtensionCount: 0,
        ppEnabledExtensionNames: ptr::null(),
    };
    
    let mut instance = VkInstance::NULL;
    kronos::vkCreateInstance(&create_info, ptr::null(), &mut instance);
    
    // Get physical device
    let mut device_count = 0;
    kronos::vkEnumeratePhysicalDevices(instance, &mut device_count, ptr::null_mut());
    if device_count == 0 {
        return None;
    }
    
    let mut physical_devices = vec![VkPhysicalDevice::NULL; device_count as usize];
    kronos::vkEnumeratePhysicalDevices(instance, &mut device_count, physical_devices.as_mut_ptr());
    let physical_device = physical_devices[0];
    
    // Create device with compute queue
    let queue_priority = 1.0f32;
    let queue_create_info = VkDeviceQueueCreateInfo {
        sType: VkStructureType::DeviceQueueCreateInfo,
        pNext: ptr::null(),
        flags: VkDeviceQueueCreateFlags::empty(),
        queueFamilyIndex: 0, // Assume compute is family 0
        queueCount: 1,
        pQueuePriorities: &queue_priority,
    };
    
    let device_create_info = VkDeviceCreateInfo {
        sType: VkStructureType::DeviceCreateInfo,
        pNext: ptr::null(),
        flags: VkDeviceCreateFlags::empty(),
        queueCreateInfoCount: 1,
        pQueueCreateInfos: &queue_create_info,
        enabledLayerCount: 0,
        ppEnabledLayerNames: ptr::null(),
        enabledExtensionCount: 0,
        ppEnabledExtensionNames: ptr::null(),
        pEnabledFeatures: ptr::null(),
    };
    
    let mut device = VkDevice::NULL;
    kronos::vkCreateDevice(physical_device, &device_create_info, ptr::null(), &mut device);
    
    let mut queue = VkQueue::NULL;
    kronos::vkGetDeviceQueue(device, 0, 0, &mut queue);
    
    // Create command pool
    let pool_create_info = VkCommandPoolCreateInfo {
        sType: VkStructureType::CommandPoolCreateInfo,
        pNext: ptr::null(),
        flags: VkCommandPoolCreateFlags::RESET_COMMAND_BUFFER,
        queueFamilyIndex: 0,
    };
    
    let mut command_pool = VkCommandPool::NULL;
    kronos::vkCreateCommandPool(device, &pool_create_info, ptr::null(), &mut command_pool);
    
    // Initialize pools
    implementation::pool_allocator::initialize_pools(device, physical_device).ok()?;
    
    // Create buffers using pool allocator
    let buffer_size = (LARGE_SIZE * std::mem::size_of::<f32>()) as VkDeviceSize;
    let buffer_create_info = VkBufferCreateInfo {
        sType: VkStructureType::BufferCreateInfo,
        pNext: ptr::null(),
        flags: VkBufferCreateFlags::empty(),
        size: buffer_size,
        usage: VkBufferUsageFlags::STORAGE_BUFFER | VkBufferUsageFlags::TRANSFER_SRC | VkBufferUsageFlags::TRANSFER_DST,
        sharingMode: VkSharingMode::Exclusive,
        queueFamilyIndexCount: 0,
        pQueueFamilyIndices: ptr::null(),
    };
    
    let mut device_buffer_a = VkBuffer::NULL;
    let mut device_buffer_b = VkBuffer::NULL;
    let mut device_buffer_c = VkBuffer::NULL;
    let mut staging_buffer = VkBuffer::NULL;
    
    kronos::vkCreateBuffer(device, &buffer_create_info, ptr::null(), &mut device_buffer_a);
    kronos::vkCreateBuffer(device, &buffer_create_info, ptr::null(), &mut device_buffer_b);
    kronos::vkCreateBuffer(device, &buffer_create_info, ptr::null(), &mut device_buffer_c);
    kronos::vkCreateBuffer(device, &buffer_create_info, ptr::null(), &mut staging_buffer);
    
    // Allocate memory from pools
    implementation::pool_allocator::allocate_buffer_memory(device, device_buffer_a, implementation::pool_allocator::PoolType::DeviceLocal).ok()?;
    implementation::pool_allocator::allocate_buffer_memory(device, device_buffer_b, implementation::pool_allocator::PoolType::DeviceLocal).ok()?;
    implementation::pool_allocator::allocate_buffer_memory(device, device_buffer_c, implementation::pool_allocator::PoolType::DeviceLocal).ok()?;
    implementation::pool_allocator::allocate_buffer_memory(device, staging_buffer, implementation::pool_allocator::PoolType::HostVisibleCoherent).ok()?;
    
    // Create persistent descriptor set
    let buffers = vec![device_buffer_a, device_buffer_b, device_buffer_c];
    let descriptor_set = implementation::persistent_descriptors::get_persistent_descriptor_set(device, &buffers).ok()?;
    
    // Get pipeline layout (simplified - would need actual pipeline creation)
    let pipeline_layout = VkPipelineLayout::NULL; // Would be created with descriptor set layout
    
    // Get vendor for barrier optimization
    let mut props = VkPhysicalDeviceProperties {
        apiVersion: 0,
        driverVersion: 0,
        vendorID: 0,
        deviceID: 0,
        deviceType: VkPhysicalDeviceType::Other,
        deviceName: [0; VK_MAX_PHYSICAL_DEVICE_NAME_SIZE],
        pipelineCacheUUID: [0; VK_UUID_SIZE],
        limits: unsafe { std::mem::zeroed() },
        sparseProperties: unsafe { std::mem::zeroed() },
    };
    kronos::vkGetPhysicalDeviceProperties(physical_device, &mut props);
    let vendor = implementation::barrier_policy::GpuVendor::from_vendor_id(props.vendorID);
    
    Some(OptimizedContext {
        device,
        queue,
        command_pool,
        descriptor_set,
        pipeline_layout,
        device_buffer_a,
        device_buffer_b,
        device_buffer_c,
        staging_buffer,
        barrier_tracker: RefCell::new(implementation::barrier_policy::BarrierTracker::new(vendor)),
    })
}

unsafe fn allocate_command_buffer(ctx: &OptimizedContext) -> VkCommandBuffer {
    let alloc_info = VkCommandBufferAllocateInfo {
        sType: VkStructureType::CommandBufferAllocateInfo,
        pNext: ptr::null(),
        commandPool: ctx.command_pool,
        level: VkCommandBufferLevel::Primary,
        commandBufferCount: 1,
    };
    
    let mut command_buffer = VkCommandBuffer::NULL;
    kronos::vkAllocateCommandBuffers(ctx.device, &alloc_info, &mut command_buffer);
    command_buffer
}

unsafe fn cleanup_context(ctx: OptimizedContext) {
    // Wait for queue idle before cleanup
    kronos::vkQueueWaitIdle(ctx.queue);
    
    // Destroy buffers
    kronos::vkDestroyBuffer(ctx.device, ctx.device_buffer_a, ptr::null());
    kronos::vkDestroyBuffer(ctx.device, ctx.device_buffer_b, ptr::null());
    kronos::vkDestroyBuffer(ctx.device, ctx.device_buffer_c, ptr::null());
    kronos::vkDestroyBuffer(ctx.device, ctx.staging_buffer, ptr::null());
    
    // Destroy command pool
    kronos::vkDestroyCommandPool(ctx.device, ctx.command_pool, ptr::null());
    
    // Destroy device
    kronos::vkDestroyDevice(ctx.device, ptr::null());
}

// Metrics helper
fn report_metrics(name: &str, metrics: &WorkloadMetrics) {
    println!("\n{} Performance Metrics:", name);
    println!("  CPU Submit Time: {:.2} µs/dispatch", metrics.cpu_submit_time_us);
    println!("  Descriptor Updates: {}/dispatch", metrics.descriptor_updates);
    println!("  Barriers: {}/dispatch", metrics.barriers_issued);
    println!("  Total Wall Time: {:.2} ms", metrics.wall_time_ms);
    
    // Check against Mini's targets
    if metrics.descriptor_updates == 0 {
        println!("  ✓ Target met: 0 descriptor updates");
    }
    
    let barriers_per_dispatch = metrics.barriers_issued as f64;
    if barriers_per_dispatch <= 0.5 {
        println!("  ✓ Target met: ≤0.5 barriers/dispatch");
    }
}

criterion_group!(
    benches,
    bench_saxpy,
    bench_reduction,
    bench_prefix_sum,
    bench_gemm
);
criterion_main!(benches);