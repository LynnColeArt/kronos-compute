//! Main entry point for Kronos Compute

use super::*;
use crate::*; // Need all the type definitions
use crate::implementation::initialize_kronos;

// Explicitly import Vulkan functions from implementation when available
#[cfg(feature = "implementation")]
use crate::implementation::{
    vkCreateInstance, vkDestroyInstance, vkEnumeratePhysicalDevices,
    vkGetPhysicalDeviceProperties, vkGetPhysicalDeviceMemoryProperties,
    vkGetPhysicalDeviceQueueFamilyProperties,
    vkCreateDevice, vkDestroyDevice, vkGetDeviceQueue,
};
use std::ffi::CString;
use std::ptr;
use std::sync::{Arc, Mutex};

/// Internal state for ComputeContext
pub(super) struct ContextInner {
    pub(super) instance: VkInstance,
    pub(super) physical_device: VkPhysicalDevice,
    pub(super) device: VkDevice,
    pub(super) queue: VkQueue,
    pub(super) queue_family_index: u32,
    
    // Optimization managers
    pub(super) descriptor_pool: VkDescriptorPool,
    pub(super) command_pool: VkCommandPool,
    
    // Device properties
    pub(super) device_properties: VkPhysicalDeviceProperties,
    pub(super) memory_properties: VkPhysicalDeviceMemoryProperties,
}

/// Main context for compute operations
/// 
/// This is the primary entry point for the Kronos Compute API.
/// It manages the Vulkan instance, device, and queue, and provides
/// methods to create buffers, pipelines, and execute commands.
#[derive(Clone)]
pub struct ComputeContext {
    pub(super) inner: Arc<Mutex<ContextInner>>,
}

// Send + Sync for thread safety
unsafe impl Send for ComputeContext {}
unsafe impl Sync for ComputeContext {}

impl ComputeContext {
    pub(super) fn new_with_config(config: ContextConfig) -> Result<Self> {
        log::info!("[SAFE API] ComputeContext::new_with_config() called");
        unsafe {
            // Apply preferred ICD selection (process-wide for now)
            log::info!("[SAFE API] Applying ICD preferences");
            if let Some(ref p) = config.preferred_icd_path {
                log::info!("[SAFE API] Setting preferred ICD path: {:?}", p);
                crate::implementation::icd_loader::set_preferred_icd_path(p.clone());
            } else if let Some(i) = config.preferred_icd_index {
                log::info!("[SAFE API] Setting preferred ICD index: {}", i);
                crate::implementation::icd_loader::set_preferred_icd_index(i);
            }

            // Initialize Kronos ICD loader
            log::info!("[SAFE API] Initializing Kronos ICD loader");
            log::info!("[SAFE API] KRONOS_AGGREGATE_ICD = {:?}", std::env::var("KRONOS_AGGREGATE_ICD").ok());
            initialize_kronos()
                .map_err(|e| {
                    log::error!("[SAFE API] Failed to initialize Kronos: {:?}", e);
                    KronosError::InitializationFailed(e.to_string())
                })?;
            log::info!("[SAFE API] Kronos initialized successfully");
            
            // Create instance
            log::info!("[SAFE API] Creating Vulkan instance");
            let instance = Self::create_instance(&config)?;
            log::info!("[SAFE API] Instance created: {:?}", instance);
            
            // Find compute-capable device
            log::info!("[SAFE API] Finding compute-capable device");
            let (physical_device, queue_family_index) = Self::find_compute_device(instance)?;
            log::info!("[SAFE API] Found device: {:?}, queue family: {}", physical_device, queue_family_index);
            
            // Get device properties
            log::info!("[SAFE API] Getting device properties");
            let mut device_properties = VkPhysicalDeviceProperties::default();
            vkGetPhysicalDeviceProperties(physical_device, &mut device_properties);
            log::info!("[SAFE API] Got device properties successfully");
            
            let mut memory_properties = VkPhysicalDeviceMemoryProperties::default();
            log::info!("[SAFE API] Getting memory properties");
            vkGetPhysicalDeviceMemoryProperties(physical_device, &mut memory_properties);
            log::info!("[SAFE API] Got memory properties successfully");
            
            // Log selected device info
            // deviceName is a fixed-size array, ensure it's null-terminated
            let device_name_bytes = &device_properties.deviceName;
            let null_pos = device_name_bytes.iter().position(|&c| c == 0).unwrap_or(device_name_bytes.len());
            // Convert from &[i8] to &[u8] for from_utf8
            let device_name_u8: Vec<u8> = device_name_bytes[..null_pos]
                .iter()
                .map(|&c| c as u8)
                .collect();
            let device_name = std::str::from_utf8(&device_name_u8)
                .unwrap_or("Unknown Device");
            let device_type_str = match device_properties.deviceType {
                VkPhysicalDeviceType::DiscreteGpu => "Discrete GPU",
                VkPhysicalDeviceType::IntegratedGpu => "Integrated GPU",
                VkPhysicalDeviceType::VirtualGpu => "Virtual GPU",
                VkPhysicalDeviceType::Cpu => "CPU (Software Renderer)",
                _ => "Unknown",
            };
            log::info!("Selected Vulkan device: {} ({})", device_name, device_type_str);
            
            // Create logical device
            log::info!("[SAFE API] Creating logical device");
            let (device, queue) = Self::create_device(physical_device, queue_family_index)?;
            log::info!("[SAFE API] Device created: {:?}, queue: {:?}", device, queue);
            
            // Create descriptor pool for persistent descriptors
            log::info!("[SAFE API] Creating descriptor pool");
            let descriptor_pool = Self::create_descriptor_pool(device)?;
            log::info!("[SAFE API] Descriptor pool created: {:?}", descriptor_pool);
            
            // Create command pool
            log::info!("[SAFE API] Creating command pool");
            let command_pool = Self::create_command_pool(device, queue_family_index)?;
            log::info!("[SAFE API] Command pool created: {:?}", command_pool);
            
            let inner = ContextInner {
                instance,
                physical_device,
                device,
                queue,
                queue_family_index,
                descriptor_pool,
                command_pool,
                device_properties,
                memory_properties,
            };
            
            // Log selected ICD info
            if let Some(info) = crate::implementation::icd_loader::selected_icd_info() {
                log::info!(
                    "ComputeContext bound to ICD: {} ({}), api=0x{:x}",
                    info.library_path.display(),
                    if info.is_software { "software" } else { "hardware" },
                    info.api_version
                );
            }

            let result = Self {
                inner: Arc::new(Mutex::new(inner)),
            };
            log::info!("[SAFE API] ComputeContext created successfully");
            Ok(result)
        }
    }
    
    /// Create a Vulkan instance
    ///
    /// # Safety
    ///
    /// This function is unsafe because:
    /// - It calls vkCreateInstance which requires the Vulkan loader to be initialized
    /// - The returned instance must be destroyed with vkDestroyInstance to avoid leaks
    /// - The config strings must remain valid for the lifetime of the instance creation
    /// - Null or invalid pointers in the create info will cause undefined behavior
    unsafe fn create_instance(config: &ContextConfig) -> Result<VkInstance> {
        log::info!("[SAFE API] create_instance called with app_name: {}", config.app_name);
        let app_name = CString::new(config.app_name.clone())
            .unwrap_or_else(|_| CString::new("Kronos App").unwrap());
        let engine_name = CString::new("Kronos Compute").unwrap();
        log::info!("[SAFE API] CStrings created successfully");
        
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
        // IMPORTANT: CStrings must remain alive during vkCreateInstance call
        // They are dropped at the end of this function, which is safe
        log::info!("[SAFE API] Calling vkCreateInstance");
        let result = vkCreateInstance(&create_info, ptr::null(), &mut instance);
        log::info!("[SAFE API] vkCreateInstance returned: {:?}", result);
        
        if result != VkResult::Success {
            log::error!("[SAFE API] vkCreateInstance failed with: {:?}", result);
            return Err(KronosError::from(result));
        }
        
        log::info!("[SAFE API] Instance created successfully: {:?}", instance);
        Ok(instance)
    }
    
    /// Find a physical device with compute capabilities
    ///
    /// # Safety
    ///
    /// This function is unsafe because:
    /// - The instance must be a valid VkInstance handle
    /// - Calls vkEnumeratePhysicalDevices which may fail with invalid instance
    /// - The returned physical device is tied to the instance lifetime
    /// - Accessing the device after instance destruction is undefined behavior
    unsafe fn find_compute_device(instance: VkInstance) -> Result<(VkPhysicalDevice, u32)> {
        let mut device_count = 0;
        log::info!("[SAFE API] Enumerating physical devices...");
        vkEnumeratePhysicalDevices(instance, &mut device_count, ptr::null_mut());
        log::info!("[SAFE API] Found {} physical devices", device_count);
        
        if device_count == 0 {
            return Err(KronosError::DeviceNotFound);
        }
        
        let mut devices = vec![VkPhysicalDevice::NULL; device_count as usize];
        vkEnumeratePhysicalDevices(instance, &mut device_count, devices.as_mut_ptr());
        
        // Collect all devices with compute support and their properties
        let mut candidates = Vec::new();
        
        for device in devices {
            let queue_family = Self::find_compute_queue_family(device)?;
            if let Some(index) = queue_family {
                // Get device properties to determine device type
                let mut properties = VkPhysicalDeviceProperties::default();
                vkGetPhysicalDeviceProperties(device, &mut properties);
                
                candidates.push((device, index, properties.deviceType));
            }
        }
        
        if candidates.is_empty() {
            return Err(KronosError::DeviceNotFound);
        }
        
        // Sort by device type preference: DiscreteGpu > IntegratedGpu > VirtualGpu > Cpu
        candidates.sort_by_key(|(_, _, device_type)| {
            match *device_type {
                VkPhysicalDeviceType::DiscreteGpu => 0,
                VkPhysicalDeviceType::IntegratedGpu => 1,
                VkPhysicalDeviceType::VirtualGpu => 2,
                VkPhysicalDeviceType::Cpu => 3,
                VkPhysicalDeviceType::Other => 4,
                _ => 5,
            }
        });
        
        // Return the best device
        let (device, queue_index, _) = candidates[0];
        Ok((device, queue_index))
    }
    
    /// Find a queue family with compute support
    ///
    /// # Safety
    ///
    /// This function is unsafe because:
    /// - The device must be a valid VkPhysicalDevice handle
    /// - Calls vkGetPhysicalDeviceQueueFamilyProperties with the device
    /// - Invalid device handle will cause undefined behavior
    /// - The device must remain valid during the function execution
    unsafe fn find_compute_queue_family(device: VkPhysicalDevice) -> Result<Option<u32>> {
        let mut queue_family_count = 0;
        vkGetPhysicalDeviceQueueFamilyProperties(device, &mut queue_family_count, ptr::null_mut());
        
        let mut queue_families = vec![VkQueueFamilyProperties {
            queueFlags: VkQueueFlags::empty(),
            queueCount: 0,
            timestampValidBits: 0,
            minImageTransferGranularity: VkExtent3D { width: 0, height: 0, depth: 0 },
        }; queue_family_count as usize];
        vkGetPhysicalDeviceQueueFamilyProperties(device, &mut queue_family_count, queue_families.as_mut_ptr());
        
        for (index, family) in queue_families.iter().enumerate() {
            if family.queueFlags.contains(VkQueueFlags::COMPUTE) {
                return Ok(Some(index as u32));
            }
        }
        
        Ok(None)
    }
    
    /// Create a logical device and get its compute queue
    ///
    /// # Safety
    ///
    /// This function is unsafe because:
    /// - The physical_device must be a valid VkPhysicalDevice handle
    /// - The queue_family_index must be valid for the physical device
    /// - Calls vkCreateDevice and vkGetDeviceQueue which require valid handles
    /// - The returned device and queue must be properly destroyed
    /// - Queue family index out of bounds will cause undefined behavior
    unsafe fn create_device(physical_device: VkPhysicalDevice, queue_family_index: u32) -> Result<(VkDevice, VkQueue)> {
        let queue_priority = 1.0f32;
        
        let queue_create_info = VkDeviceQueueCreateInfo {
            sType: VkStructureType::DeviceQueueCreateInfo,
            pNext: ptr::null(),
            flags: 0,
            queueFamilyIndex: queue_family_index,
            queueCount: 1,
            pQueuePriorities: &queue_priority,
        };
        
        // Don't request any features - use default (all disabled)
        let features = VkPhysicalDeviceFeatures::default();
        log::info!("[SAFE API] Creating device with default features (all disabled)");
        
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
            pEnabledFeatures: &features,
        };
        
        let mut device = VkDevice::NULL;
        log::info!("[SAFE API] Calling vkCreateDevice with queue family index {}", queue_family_index);
        let result = vkCreateDevice(physical_device, &device_create_info, ptr::null(), &mut device);
        log::info!("[SAFE API] vkCreateDevice returned: {:?}", result);
        
        if result != VkResult::Success {
            log::error!("[SAFE API] Failed to create device: {:?}", result);
            return Err(KronosError::from(result));
        }
        
        let mut queue = VkQueue::NULL;
        vkGetDeviceQueue(device, queue_family_index, 0, &mut queue);
        
        Ok((device, queue))
    }
    
    /// Create a descriptor pool for persistent descriptors
    ///
    /// # Safety
    ///
    /// This function is unsafe because:
    /// - The device must be a valid VkDevice handle
    /// - Calls vkCreateDescriptorPool which requires valid device
    /// - The returned pool must be destroyed with vkDestroyDescriptorPool
    /// - Invalid device handle will cause undefined behavior
    /// - Pool creation may fail if device limits are exceeded
    unsafe fn create_descriptor_pool(device: VkDevice) -> Result<VkDescriptorPool> {
        // Create a large pool for persistent descriptors
        let pool_size = VkDescriptorPoolSize {
            type_: VkDescriptorType::StorageBuffer,
            descriptorCount: 10000, // Should be enough for most use cases
        };
        
        let pool_info = VkDescriptorPoolCreateInfo {
            sType: VkStructureType::DescriptorPoolCreateInfo,
            pNext: ptr::null(),
            flags: VkDescriptorPoolCreateFlags::FREE_DESCRIPTOR_SET,
            maxSets: 1000,
            poolSizeCount: 1,
            pPoolSizes: &pool_size,
        };
        
        let mut pool = VkDescriptorPool::NULL;
        let result = vkCreateDescriptorPool(device, &pool_info, ptr::null(), &mut pool);
        
        if result != VkResult::Success {
            return Err(KronosError::from(result));
        }
        
        Ok(pool)
    }
    
    /// Create a command pool for allocating command buffers
    ///
    /// # Safety
    ///
    /// This function is unsafe because:
    /// - The device must be a valid VkDevice handle
    /// - The queue_family_index must be valid for the device
    /// - Calls vkCreateCommandPool which requires valid parameters
    /// - The returned pool must be destroyed with vkDestroyCommandPool
    /// - Invalid queue family index will cause undefined behavior
    unsafe fn create_command_pool(device: VkDevice, queue_family_index: u32) -> Result<VkCommandPool> {
        let pool_info = VkCommandPoolCreateInfo {
            sType: VkStructureType::CommandPoolCreateInfo,
            pNext: ptr::null(),
            flags: VkCommandPoolCreateFlags::RESET_COMMAND_BUFFER,
            queueFamilyIndex: queue_family_index,
        };
        
        let mut pool = VkCommandPool::NULL;
        let result = vkCreateCommandPool(device, &pool_info, ptr::null(), &mut pool);
        
        if result != VkResult::Success {
            return Err(KronosError::from(result));
        }
        
        Ok(pool)
    }
    
    /// Get the underlying Vulkan device (for advanced usage)
    pub fn device(&self) -> VkDevice {
        self.inner.lock().unwrap().device
    }
    
    /// Get the compute queue
    pub fn queue(&self) -> VkQueue {
        self.inner.lock().unwrap().queue
    }
    
    /// Get device properties
    pub fn device_properties(&self) -> VkPhysicalDeviceProperties {
        self.inner.lock().unwrap().device_properties
    }
    
    /// Get information about the ICD bound to this context (process-wide)
    pub fn icd_info(&self) -> Option<crate::implementation::icd_loader::IcdInfo> {
        crate::implementation::icd_loader::selected_icd_info()
    }

    // Internal helper for other modules
    pub(super) fn with_inner<F, R>(&self, f: F) -> R
    where
        F: FnOnce(&ContextInner) -> R,
    {
        let inner = self.inner.lock().unwrap();
        f(&*inner)
    }
}

impl Drop for ComputeContext {
    fn drop(&mut self) {
        // Only the last Clone should perform destruction to avoid double-free.
        if std::sync::Arc::strong_count(&self.inner) != 1 {
            return;
        }
        let inner = self.inner.lock().unwrap();
        unsafe {
            if inner.command_pool != VkCommandPool::NULL {
                vkDestroyCommandPool(inner.device, inner.command_pool, ptr::null());
            }
            if inner.descriptor_pool != VkDescriptorPool::NULL {
                vkDestroyDescriptorPool(inner.device, inner.descriptor_pool, ptr::null());
            }
            if inner.device != VkDevice::NULL {
                vkDestroyDevice(inner.device, ptr::null());
            }
            if inner.instance != VkInstance::NULL {
                vkDestroyInstance(inner.instance, ptr::null());
            }
        }
    }
}
