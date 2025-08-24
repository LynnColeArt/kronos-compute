//! Benchmark initialization performance of Kronos vs standard Vulkan

use criterion::{black_box, criterion_group, criterion_main, Criterion, BenchmarkId};
use kronos::*;
use std::ffi::CString;
use std::ptr;
use std::time::Instant;

/// Measure Kronos instance creation time
fn bench_kronos_instance_creation(c: &mut Criterion) {
    c.bench_function("kronos_instance_creation", |b| {
        b.iter(|| {
            unsafe {
                // Initialize Kronos
                let _ = kronos::initialize_kronos();
                
                // Create instance
                let app_name = CString::new("Benchmark").unwrap();
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
                
                if result == VkResult::Success && !instance.is_null() {
                    vkDestroyInstance(instance, ptr::null());
                }
                
                black_box(result);
            }
        });
    });
}

/// Measure time to enumerate physical devices
fn bench_physical_device_enumeration(c: &mut Criterion) {
    unsafe {
        // Setup instance once
        let _ = kronos::initialize_kronos();
        
        let app_name = CString::new("Benchmark").unwrap();
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
        
        if result == VkResult::Success {
            c.bench_function("physical_device_enumeration", |b| {
                b.iter(|| {
                    let mut device_count = 0u32;
                    vkEnumeratePhysicalDevices(instance, &mut device_count, ptr::null_mut());
                    
                    if device_count > 0 {
                        let mut devices = vec![VkPhysicalDevice::NULL; device_count as usize];
                        vkEnumeratePhysicalDevices(instance, &mut device_count, devices.as_mut_ptr());
                    }
                    
                    black_box(device_count);
                });
            });
            
            vkDestroyInstance(instance, ptr::null());
        }
    }
}

/// Measure full initialization sequence
fn bench_full_initialization(c: &mut Criterion) {
    c.bench_function("full_initialization_sequence", |b| {
        b.iter(|| {
            unsafe {
                let start = Instant::now();
                
                // 1. Initialize Kronos
                let _ = kronos::initialize_kronos();
                
                // 2. Create instance
                let app_name = CString::new("Benchmark").unwrap();
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
                    return;
                }
                
                // 3. Enumerate physical devices
                let mut device_count = 0u32;
                vkEnumeratePhysicalDevices(instance, &mut device_count, ptr::null_mut());
                
                if device_count == 0 {
                    vkDestroyInstance(instance, ptr::null());
                    return;
                }
                
                let mut physical_devices = vec![VkPhysicalDevice::NULL; device_count as usize];
                vkEnumeratePhysicalDevices(instance, &mut device_count, physical_devices.as_mut_ptr());
                
                let physical_device = physical_devices[0];
                
                // 4. Get queue family properties
                let mut queue_family_count = 0u32;
                vkGetPhysicalDeviceQueueFamilyProperties(physical_device, &mut queue_family_count, ptr::null_mut());
                
                let mut queue_families = vec![VkQueueFamilyProperties::default(); queue_family_count as usize];
                vkGetPhysicalDeviceQueueFamilyProperties(physical_device, &mut queue_family_count, queue_families.as_mut_ptr());
                
                // Find compute queue
                let compute_queue_family = queue_families.iter()
                    .position(|qf| qf.queueFlags.contains(VkQueueFlags::COMPUTE))
                    .unwrap_or(0) as u32;
                
                // 5. Create device
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
                if vkCreateDevice(physical_device, &device_create_info, ptr::null(), &mut device) == VkResult::Success {
                    vkDestroyDevice(device, ptr::null());
                }
                
                vkDestroyInstance(instance, ptr::null());
                
                let elapsed = start.elapsed();
                black_box(elapsed);
            }
        });
    });
}

/// Compare initialization with different numbers of extensions
fn bench_initialization_scaling(c: &mut Criterion) {
    let mut group = c.benchmark_group("initialization_scaling");
    
    for extension_count in [0, 1, 5, 10].iter() {
        group.bench_with_input(
            BenchmarkId::new("extensions", extension_count),
            extension_count,
            |b, &ext_count| {
                b.iter(|| {
                    unsafe {
                        let _ = kronos::initialize_kronos();
                        
                        let app_name = CString::new("Benchmark").unwrap();
                        let engine_name = CString::new("Kronos").unwrap();
                        
                        // Create dummy extension names
                        let extensions: Vec<CString> = (0..ext_count)
                            .map(|i| CString::new(format!("VK_EXT_dummy_{}", i)).unwrap())
                            .collect();
                        let extension_ptrs: Vec<*const i8> = extensions.iter()
                            .map(|e| e.as_ptr())
                            .collect();
                        
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
                            enabledExtensionCount: ext_count as u32,
                            ppEnabledExtensionNames: if ext_count > 0 { extension_ptrs.as_ptr() } else { ptr::null() },
                        };
                        
                        let mut instance = VkInstance::NULL;
                        let result = vkCreateInstance(&create_info, ptr::null(), &mut instance);
                        
                        if result == VkResult::Success && !instance.is_null() {
                            vkDestroyInstance(instance, ptr::null());
                        }
                        
                        black_box(result);
                    }
                });
            },
        );
    }
    group.finish();
}

criterion_group!(
    benches,
    bench_kronos_instance_creation,
    bench_physical_device_enumeration,
    bench_full_initialization,
    bench_initialization_scaling
);
criterion_main!(benches);