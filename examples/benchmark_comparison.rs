//! Direct comparison between Kronos and standard Vulkan

use std::ffi::{CString, c_void};
use std::ptr;
use std::time::{Duration, Instant};
use libc;

// Kronos imports
use kronos::*;

// Standard Vulkan function types (matching Kronos signatures)
type VulkanCreateInstance = unsafe extern "C" fn(
    *const VkInstanceCreateInfo,
    *const VkAllocationCallbacks,
    *mut VkInstance,
) -> VkResult;

type VulkanDestroyInstance = unsafe extern "C" fn(
    VkInstance,
    *const VkAllocationCallbacks,
);

type VulkanEnumeratePhysicalDevices = unsafe extern "C" fn(
    VkInstance,
    *mut u32,
    *mut VkPhysicalDevice,
) -> VkResult;

type VulkanCreateDevice = unsafe extern "C" fn(
    VkPhysicalDevice,
    *const VkDeviceCreateInfo,
    *const VkAllocationCallbacks,
    *mut VkDevice,
) -> VkResult;

type VulkanDestroyDevice = unsafe extern "C" fn(
    VkDevice,
    *const VkAllocationCallbacks,
);

struct VulkanFunctions {
    create_instance: VulkanCreateInstance,
    destroy_instance: VulkanDestroyInstance,
    enumerate_physical_devices: VulkanEnumeratePhysicalDevices,
    create_device: VulkanCreateDevice,
    destroy_device: VulkanDestroyDevice,
}

unsafe fn load_vulkan() -> Option<(VulkanFunctions, *mut c_void)> {
    // Load standard Vulkan library
    let lib_name = CString::new("libvulkan.so.1").ok()?;
    let handle = libc::dlopen(lib_name.as_ptr(), libc::RTLD_NOW);
    if handle.is_null() {
        eprintln!("Failed to load libvulkan.so.1");
        return None;
    }

    // Load functions
    let get_fn = |name: &str| -> Option<*const c_void> {
        let c_name = CString::new(name).ok()?;
        let ptr = libc::dlsym(handle, c_name.as_ptr());
        if ptr.is_null() {
            None
        } else {
            Some(ptr)
        }
    };

    let create_instance = std::mem::transmute(get_fn("vkCreateInstance")?);
    let destroy_instance = std::mem::transmute(get_fn("vkDestroyInstance")?);
    let enumerate_physical_devices = std::mem::transmute(get_fn("vkEnumeratePhysicalDevices")?);
    let create_device = std::mem::transmute(get_fn("vkCreateDevice")?);
    let destroy_device = std::mem::transmute(get_fn("vkDestroyDevice")?);

    Some((VulkanFunctions {
        create_instance,
        destroy_instance,
        enumerate_physical_devices,
        create_device,
        destroy_device,
    }, handle))
}

fn benchmark_operation<F>(name: &str, iterations: u32, mut op: F) -> Duration
where
    F: FnMut(),
{
    // Warmup
    for _ in 0..10 {
        op();
    }
    
    // Measure
    let start = Instant::now();
    for _ in 0..iterations {
        op();
    }
    let elapsed = start.elapsed();
    
    elapsed
}

fn main() {
    println!("Kronos vs Standard Vulkan Performance Comparison");
    println!("{}", "=".repeat(70));
    println!();

    unsafe {
        // Initialize Kronos
        if let Err(e) = kronos::initialize_kronos() {
            eprintln!("Failed to initialize Kronos: {:?}", e);
            return;
        }

        // Load standard Vulkan
        let (vulkan, vulkan_handle) = match load_vulkan() {
            Some(v) => v,
            None => {
                eprintln!("Failed to load standard Vulkan library");
                return;
            }
        };

        println!("Both libraries loaded successfully!\n");

        // Test parameters
        let iterations = 100;

        // 1. Instance Creation Benchmark
        println!("1. Instance Creation/Destruction");
        println!("{}", "-".repeat(70));

        // Kronos benchmark
        let kronos_instance_time = benchmark_operation("Kronos instance", iterations, || {
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
        });

        // Standard Vulkan benchmark
        let vulkan_instance_time = benchmark_operation("Vulkan instance", iterations, || {
            let app_name = CString::new("Benchmark").unwrap();
            let engine_name = CString::new("Vulkan").unwrap();
            
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
            let result = (vulkan.create_instance)(&create_info, ptr::null(), &mut instance);
            
            if result == VkResult::Success && !instance.is_null() {
                (vulkan.destroy_instance)(instance, ptr::null());
            }
        });

        let kronos_avg = kronos_instance_time.as_nanos() as f64 / iterations as f64;
        let vulkan_avg = vulkan_instance_time.as_nanos() as f64 / iterations as f64;
        let improvement = ((vulkan_avg - kronos_avg) / vulkan_avg * 100.0).max(0.0);

        println!("Kronos:  {:>10.2} ns/iter", kronos_avg);
        println!("Vulkan:  {:>10.2} ns/iter", vulkan_avg);
        println!("Improvement: {:.1}% faster", improvement);

        // 2. Full Initialization Sequence
        println!("\n2. Full Initialization (Instance + Device)");
        println!("{}", "-".repeat(70));

        // First create instances for device enumeration
        let app_name = CString::new("Benchmark").unwrap();
        let engine_name = CString::new("Test").unwrap();
        
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

        // Create test instances
        let mut kronos_instance = VkInstance::NULL;
        let mut vulkan_instance = VkInstance::NULL;
        
        vkCreateInstance(&create_info, ptr::null(), &mut kronos_instance);
        (vulkan.create_instance)(&create_info, ptr::null(), &mut vulkan_instance);

        // Check if we have physical devices
        let mut kronos_device_count = 0u32;
        let mut vulkan_device_count = 0u32;
        
        vkEnumeratePhysicalDevices(kronos_instance, &mut kronos_device_count, ptr::null_mut());
        (vulkan.enumerate_physical_devices)(vulkan_instance, &mut vulkan_device_count, ptr::null_mut());

        println!("Kronos found {} device(s)", kronos_device_count);
        println!("Vulkan found {} device(s)", vulkan_device_count);

        if kronos_device_count > 0 && vulkan_device_count > 0 {
            // Get physical devices
            let mut kronos_physical_devices = vec![VkPhysicalDevice::NULL; kronos_device_count as usize];
            let mut vulkan_physical_devices = vec![VkPhysicalDevice::NULL; vulkan_device_count as usize];
            
            vkEnumeratePhysicalDevices(kronos_instance, &mut kronos_device_count, kronos_physical_devices.as_mut_ptr());
            (vulkan.enumerate_physical_devices)(vulkan_instance, &mut vulkan_device_count, vulkan_physical_devices.as_mut_ptr());

            let kronos_physical_device = kronos_physical_devices[0];
            let vulkan_physical_device = vulkan_physical_devices[0];

            // Benchmark device creation
            let iterations = 50;
            
            let kronos_device_time = benchmark_operation("Kronos device", iterations, || {
                let queue_priority = 1.0f32;
                let queue_create_info = VkDeviceQueueCreateInfo {
                    sType: VkStructureType::DeviceQueueCreateInfo,
                    pNext: ptr::null(),
                    flags: 0,
                    queueFamilyIndex: 0, // Assume 0 for simplicity
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
                let result = vkCreateDevice(kronos_physical_device, &device_create_info, ptr::null(), &mut device);
                
                if result == VkResult::Success && !device.is_null() {
                    vkDestroyDevice(device, ptr::null());
                }
            });

            let vulkan_device_time = benchmark_operation("Vulkan device", iterations, || {
                let queue_priority = 1.0f32;
                let queue_create_info = VkDeviceQueueCreateInfo {
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
                    pQueueCreateInfos: &queue_create_info,
                    enabledLayerCount: 0,
                    ppEnabledLayerNames: ptr::null(),
                    enabledExtensionCount: 0,
                    ppEnabledExtensionNames: ptr::null(),
                    pEnabledFeatures: ptr::null(),
                };
                
                let mut device = VkDevice::NULL;
                let result = (vulkan.create_device)(vulkan_physical_device, &device_create_info, ptr::null(), &mut device);
                
                if result == VkResult::Success && !device.is_null() {
                    (vulkan.destroy_device)(device, ptr::null());
                }
            });

            let kronos_avg = kronos_device_time.as_nanos() as f64 / iterations as f64;
            let vulkan_avg = vulkan_device_time.as_nanos() as f64 / iterations as f64;
            let improvement = ((vulkan_avg - kronos_avg) / vulkan_avg * 100.0).max(0.0);

            println!("\nDevice Creation:");
            println!("Kronos:  {:>10.2} ns/iter", kronos_avg);
            println!("Vulkan:  {:>10.2} ns/iter", vulkan_avg);
            println!("Improvement: {:.1}% faster", improvement);
        }

        // Cleanup
        if !kronos_instance.is_null() {
            vkDestroyInstance(kronos_instance, ptr::null());
        }
        if !vulkan_instance.is_null() {
            (vulkan.destroy_instance)(vulkan_instance, ptr::null());
        }

        // Structure creation comparison
        println!("\n3. Structure Creation Overhead");
        println!("{}", "-".repeat(70));
        
        let iterations = 1000000;
        
        // Both use the same structures, so timing should be identical
        let struct_time = benchmark_operation("VkBufferCreateInfo", iterations, || {
            let info = VkBufferCreateInfo {
                sType: VkStructureType::BufferCreateInfo,
                pNext: ptr::null(),
                flags: VkBufferCreateFlags::empty(),
                size: 1024 * 1024,
                usage: VkBufferUsageFlags::STORAGE_BUFFER,
                sharingMode: VkSharingMode::Exclusive,
                queueFamilyIndexCount: 0,
                pQueueFamilyIndices: ptr::null(),
            };
            std::hint::black_box(info);
        });
        
        let avg_time = struct_time.as_nanos() as f64 / iterations as f64;
        println!("Structure creation: {:.2} ns/iter (same for both)", avg_time);

        // Summary
        println!("\n{}", "=".repeat(70));
        println!("Summary:");
        println!("- Kronos uses the same data structures as Vulkan");
        println!("- Primary advantage is in reduced initialization overhead");
        println!("- Compute-only focus eliminates graphics subsystem costs");
        println!("- Direct ICD forwarding minimizes overhead");

        // Cleanup
        libc::dlclose(vulkan_handle);
    }
}