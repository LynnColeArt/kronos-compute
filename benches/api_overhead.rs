//! Benchmarks comparing Rust Kronos API overhead vs C implementations

use criterion::{black_box, criterion_group, criterion_main, Criterion, BenchmarkId};
use kronos::*;
use std::mem;
use std::ptr;

fn benchmark_structure_creation(c: &mut Criterion) {
    let mut group = c.benchmark_group("structure_creation");
    
    // Benchmark VkApplicationInfo creation
    group.bench_function("VkApplicationInfo::default", |b| {
        b.iter(|| {
            let app_info = black_box(VkApplicationInfo::default());
            app_info
        });
    });
    
    group.bench_function("VkApplicationInfo::manual", |b| {
        b.iter(|| {
            let app_info = black_box(VkApplicationInfo {
                sType: VkStructureType::ApplicationInfo,
                pNext: ptr::null(),
                pApplicationName: ptr::null(),
                applicationVersion: 0,
                pEngineName: ptr::null(),
                engineVersion: 0,
                apiVersion: VK_API_VERSION_1_0,
            });
            app_info
        });
    });
    
    // Benchmark VkBufferCreateInfo creation
    group.bench_function("VkBufferCreateInfo::default", |b| {
        b.iter(|| {
            let buffer_info = black_box(VkBufferCreateInfo::default());
            buffer_info
        });
    });
    
    group.bench_function("VkBufferCreateInfo::manual", |b| {
        b.iter(|| {
            let buffer_info = black_box(VkBufferCreateInfo {
                sType: VkStructureType::BufferCreateInfo,
                pNext: ptr::null(),
                size: 1024 * 1024,
                usage: VkBufferUsageFlags::STORAGE_BUFFER,
                sharingMode: VkSharingMode::Exclusive,
                queueFamilyIndexCount: 0,
                pQueueFamilyIndices: ptr::null(),
                flags: VkBufferCreateFlags::empty(),
            });
            buffer_info
        });
    });
    
    group.finish();
}

fn benchmark_memory_type_lookup(c: &mut Criterion) {
    let mut group = c.benchmark_group("memory_type_lookup");
    
    // Setup mock memory properties
    let mut mem_props = VkPhysicalDeviceMemoryProperties {
        memoryTypeCount: 11,
        memoryTypes: [VkMemoryType { propertyFlags: VkMemoryPropertyFlags::empty(), heapIndex: 0 }; 32],
        memoryHeapCount: 4,
        memoryHeaps: [VkMemoryHeap { size: 0, flags: 0 }; 16],
    };
    
    // Set up realistic memory types
    mem_props.memoryTypes[0].propertyFlags = VkMemoryPropertyFlags::DEVICE_LOCAL;
    mem_props.memoryTypes[2].propertyFlags = VkMemoryPropertyFlags::HOST_VISIBLE | VkMemoryPropertyFlags::HOST_COHERENT;
    mem_props.memoryTypes[3].propertyFlags = VkMemoryPropertyFlags::HOST_VISIBLE | VkMemoryPropertyFlags::HOST_CACHED;
    mem_props.memoryTypes[7].propertyFlags = VkMemoryPropertyFlags::DEVICE_LOCAL | VkMemoryPropertyFlags::HOST_VISIBLE;
    
    let memory_type_bits: u32 = 0x7FF; // First 11 types available
    
    // Traditional O(n) lookup
    group.bench_function("traditional_O(n)_lookup", |b| {
        b.iter(|| {
            let mut memory_type = 0;
            for i in 0..mem_props.memoryTypeCount {
                if (memory_type_bits & (1 << i)) != 0 &&
                   mem_props.memoryTypes[i as usize].propertyFlags.contains(
                       VkMemoryPropertyFlags::HOST_VISIBLE | VkMemoryPropertyFlags::HOST_COHERENT
                   ) {
                    memory_type = i;
                    break;
                }
            }
            black_box(memory_type)
        });
    });
    
    // Pre-cached O(1) lookup
    let mut cache = VkMemoryTypeCache::default();
    for i in 0..mem_props.memoryTypeCount {
        let flags = mem_props.memoryTypes[i as usize].propertyFlags;
        if flags.contains(VkMemoryPropertyFlags::HOST_VISIBLE | VkMemoryPropertyFlags::HOST_COHERENT) {
            cache.hostVisibleCoherent = i;
        }
        if flags.contains(VkMemoryPropertyFlags::DEVICE_LOCAL) && cache.deviceLocal == 0 {
            cache.deviceLocal = i;
        }
    }
    
    group.bench_function("cached_O(1)_lookup", |b| {
        b.iter(|| {
            black_box(cache.hostVisibleCoherent)
        });
    });
    
    group.finish();
}

fn benchmark_flag_operations(c: &mut Criterion) {
    let mut group = c.benchmark_group("flag_operations");
    
    // Benchmark bitflag operations
    group.bench_function("VkQueueFlags::contains", |b| {
        let flags = VkQueueFlags::COMPUTE | VkQueueFlags::TRANSFER;
        b.iter(|| {
            black_box(flags.contains(VkQueueFlags::COMPUTE))
        });
    });
    
    group.bench_function("VkQueueFlags::intersects", |b| {
        let flags1 = VkQueueFlags::COMPUTE | VkQueueFlags::TRANSFER;
        let flags2 = VkQueueFlags::COMPUTE | VkQueueFlags::SPARSE_BINDING;
        b.iter(|| {
            black_box(flags1.intersects(flags2))
        });
    });
    
    group.bench_function("VkBufferUsageFlags::union", |b| {
        let flags1 = VkBufferUsageFlags::STORAGE_BUFFER;
        let flags2 = VkBufferUsageFlags::TRANSFER_DST;
        b.iter(|| {
            black_box(flags1 | flags2)
        });
    });
    
    group.finish();
}

fn benchmark_handle_operations(c: &mut Criterion) {
    let mut group = c.benchmark_group("handle_operations");
    
    // Benchmark handle creation and comparison
    group.bench_function("Handle::from_raw", |b| {
        b.iter(|| {
            let handle = black_box(VkBuffer::from_raw(0x123456789ABCDEF0));
            handle
        });
    });
    
    group.bench_function("Handle::is_null", |b| {
        let handle = VkBuffer::from_raw(0x123456789ABCDEF0);
        b.iter(|| {
            black_box(handle.is_null())
        });
    });
    
    group.bench_function("Handle::equality", |b| {
        let handle1 = VkBuffer::from_raw(0x123456789ABCDEF0);
        let handle2 = VkBuffer::from_raw(0x123456789ABCDEF0);
        b.iter(|| {
            black_box(handle1 == handle2)
        });
    });
    
    group.finish();
}

fn benchmark_structure_sizes(c: &mut Criterion) {
    let mut group = c.benchmark_group("structure_sizes");
    
    // Compare structure sizes (these are compile-time constants, but useful for documentation)
    let sizes = vec![
        ("VkPhysicalDeviceFeatures", mem::size_of::<VkPhysicalDeviceFeatures>()),
        ("VkBufferCreateInfo", mem::size_of::<VkBufferCreateInfo>()),
        ("VkComputePipelineCreateInfo", mem::size_of::<VkComputePipelineCreateInfo>()),
        ("VkMemoryTypeCache", mem::size_of::<VkMemoryTypeCache>()),
    ];
    
    for (name, size) in sizes {
        group.bench_with_input(BenchmarkId::new("size", name), &size, |b, &size| {
            b.iter(|| black_box(size));
        });
    }
    
    group.finish();
}

fn benchmark_enum_dispatch(c: &mut Criterion) {
    let mut group = c.benchmark_group("enum_dispatch");
    
    // Benchmark enum matching (simulating dispatch tables)
    group.bench_function("VkStructureType::match", |b| {
        let stype = VkStructureType::ComputePipelineCreateInfo;
        b.iter(|| {
            match black_box(stype) {
                VkStructureType::ApplicationInfo => 1,
                VkStructureType::InstanceCreateInfo => 2,
                VkStructureType::DeviceCreateInfo => 3,
                VkStructureType::BufferCreateInfo => 4,
                VkStructureType::ComputePipelineCreateInfo => 5,
                _ => 0,
            }
        });
    });
    
    group.bench_function("VkResult::is_success", |b| {
        let result = VkResult::Success;
        b.iter(|| {
            black_box(result == VkResult::Success)
        });
    });
    
    group.finish();
}

fn benchmark_array_operations(c: &mut Criterion) {
    let mut group = c.benchmark_group("array_operations");
    
    // Benchmark operations on fixed-size arrays (like memory types)
    let mem_types = [VkMemoryType { 
        propertyFlags: VkMemoryPropertyFlags::DEVICE_LOCAL, 
        heapIndex: 0 
    }; 32];
    
    group.bench_function("iterate_memory_types", |b| {
        b.iter(|| {
            let mut found = None;
            for (i, mem_type) in mem_types.iter().enumerate().take(11) {
                if mem_type.propertyFlags.contains(VkMemoryPropertyFlags::DEVICE_LOCAL) {
                    found = Some(i);
                    break;
                }
            }
            black_box(found)
        });
    });
    
    group.finish();
}

criterion_group!(
    benches,
    benchmark_structure_creation,
    benchmark_memory_type_lookup,
    benchmark_flag_operations,
    benchmark_handle_operations,
    benchmark_structure_sizes,
    benchmark_enum_dispatch,
    benchmark_array_operations
);
criterion_main!(benches);