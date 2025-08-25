//! Test compute functionality with thread-safe implementation

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
    
    fn vkGetPhysicalDeviceProperties(
        physicalDevice: VkPhysicalDevice,
        pProperties: *mut VkPhysicalDeviceProperties,
    );
    
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
    
    fn vkGetDeviceQueue(
        device: VkDevice,
        queueFamilyIndex: u32,
        queueIndex: u32,
        pQueue: *mut VkQueue,
    );
    
    fn vkCreateCommandPool(
        device: VkDevice,
        pCreateInfo: *const VkCommandPoolCreateInfo,
        pAllocator: *const VkAllocationCallbacks,
        pCommandPool: *mut VkCommandPool,
    ) -> VkResult;
    
    fn vkDestroyCommandPool(
        device: VkDevice,
        commandPool: VkCommandPool,
        pAllocator: *const VkAllocationCallbacks,
    );
}

fn main() {
    println!("Kronos Compute Test");
    println!("===================\n");
    
    unsafe {
        // 1. Create instance
        println!("Creating instance...");
        let app_name = CString::new("Kronos Compute Test").unwrap();
        let engine_name = CString::new("Kronos Engine").unwrap();
        
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
        let result = vkCreateInstance(&create_info, ptr::null(), &mut instance);
        
        if result == VkResult::Success {
            println!("✓ Instance created: {:?}", instance);
        } else {
            println!("✗ Failed to create instance: {:?}", result);
            return;
        }
        
        // 2. Enumerate physical devices
        println!("\nEnumerating physical devices...");
        let mut device_count = 0;
        vkEnumeratePhysicalDevices(instance, &mut device_count, ptr::null_mut());
        println!("Found {} physical device(s)", device_count);
        
        let mut physical_devices = vec![VkPhysicalDevice::NULL; device_count as usize];
        vkEnumeratePhysicalDevices(instance, &mut device_count, physical_devices.as_mut_ptr());
        
        let physical_device = physical_devices[0];
        println!("✓ Using physical device: {:?}", physical_device);
        
        // 3. Get device properties
        println!("\nGetting device properties...");
        let mut properties = std::mem::zeroed::<VkPhysicalDeviceProperties>();
        vkGetPhysicalDeviceProperties(physical_device, &mut properties);
        
        // Convert device name from C array to string
        let device_name = std::ffi::CStr::from_ptr(properties.deviceName.as_ptr())
            .to_string_lossy();
        
        println!("Device: {}", device_name);
        println!("API Version: {}.{}.{}", 
                 VK_VERSION_MAJOR(properties.apiVersion),
                 VK_VERSION_MINOR(properties.apiVersion),
                 VK_VERSION_PATCH(properties.apiVersion));
        println!("Max compute work group size: {:?}", 
                 properties.limits.maxComputeWorkGroupSize);
        
        // 4. Create logical device
        println!("\nCreating logical device...");
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
        
        if result == VkResult::Success {
            println!("✓ Device created: {:?}", device);
        } else {
            println!("✗ Failed to create device: {:?}", result);
            return;
        }
        
        // 5. Get compute queue
        println!("\nGetting compute queue...");
        let mut queue = VkQueue::NULL;
        vkGetDeviceQueue(device, 0, 0, &mut queue);
        println!("✓ Got compute queue: {:?}", queue);
        
        // 6. Create command pool
        println!("\nCreating command pool...");
        let pool_create_info = VkCommandPoolCreateInfo {
            sType: VkStructureType::CommandPoolCreateInfo,
            pNext: ptr::null(),
            flags: VkCommandPoolCreateFlags::empty(),
            queueFamilyIndex: 0,
        };
        
        let mut command_pool = VkCommandPool::NULL;
        let result = vkCreateCommandPool(device, &pool_create_info, ptr::null(), &mut command_pool);
        
        if result == VkResult::Success {
            println!("✓ Command pool created: {:?}", command_pool);
        } else {
            println!("✗ Failed to create command pool: {:?}", result);
            return;
        }
        
        // Cleanup
        println!("\nCleaning up...");
        vkDestroyCommandPool(device, command_pool, ptr::null());
        vkDestroyDevice(device, ptr::null());
        vkDestroyInstance(instance, ptr::null());
        
        println!("✓ Cleanup complete");
        println!("\n✓ All tests passed!");
    }
}

// Version macros
const fn VK_MAKE_VERSION(major: u32, minor: u32, patch: u32) -> u32 {
    (major << 22) | (minor << 12) | patch
}

const fn VK_VERSION_MAJOR(version: u32) -> u32 {
    version >> 22
}

const fn VK_VERSION_MINOR(version: u32) -> u32 {
    (version >> 12) & 0x3ff
}

const fn VK_VERSION_PATCH(version: u32) -> u32 {
    version & 0xfff
}