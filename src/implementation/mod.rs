//! Actual implementation of Kronos compute APIs

use std::sync::{Arc, Mutex};
use std::collections::HashMap;
use crate::sys::*;
use crate::core::*;
use crate::ffi::*;

pub mod instance;
pub mod device;
pub mod memory;
pub mod buffer;
pub mod compute;
pub mod descriptor;
pub mod sync;
pub mod icd_loader;
pub mod forward;

// Re-export all implementation functions
pub use instance::*;
pub use device::*;
pub use memory::*;
pub use buffer::*;
pub use compute::*;
pub use descriptor::*;
pub use sync::*;

// Backend mode
#[derive(Debug, Clone, Copy)]
pub enum BackendMode {
    Mock,      // Use our mock implementation
    RealICD,   // Use real Vulkan driver
}

lazy_static::lazy_static! {
    pub static ref BACKEND_MODE: Mutex<BackendMode> = Mutex::new(BackendMode::Mock);
}

/// Set backend mode
pub fn set_backend_mode(mode: BackendMode) {
    *BACKEND_MODE.lock().unwrap() = mode;
}

/// Get current backend mode
pub fn get_backend_mode() -> BackendMode {
    let mode = BACKEND_MODE.lock().unwrap();
    match *mode {
        BackendMode::Mock => BackendMode::Mock,
        BackendMode::RealICD => BackendMode::RealICD,
    }
}

/// Initialize Kronos with specified backend
pub fn initialize_kronos(mode: BackendMode) -> Result<(), String> {
    match mode {
        BackendMode::Mock => {
            *BACKEND_MODE.lock().unwrap() = BackendMode::Mock;
            Ok(())
        }
        BackendMode::RealICD => {
            icd_loader::initialize_icd_loader()?;
            *BACKEND_MODE.lock().unwrap() = BackendMode::RealICD;
            Ok(())
        }
    }
}

// Global loader state
lazy_static::lazy_static! {
    static ref LOADER: Mutex<KronosLoader> = Mutex::new(KronosLoader::new());
}

/// The Kronos loader - manages instances and devices
pub struct KronosLoader {
    instances: HashMap<u64, Arc<Instance>>,
    next_handle: u64,
}

impl KronosLoader {
    fn new() -> Self {
        Self {
            instances: HashMap::new(),
            next_handle: 1,
        }
    }
    
    fn allocate_handle(&mut self) -> u64 {
        let handle = self.next_handle;
        self.next_handle += 1;
        handle
    }
}

/// Instance implementation - thread-safe version
pub struct Instance {
    handle: VkInstance,
    app_name: Option<String>,
    engine_name: Option<String>,
    physical_devices: Vec<Arc<PhysicalDevice>>,
}

// Mark Instance as Send + Sync explicitly
unsafe impl Send for Instance {}
unsafe impl Sync for Instance {}

/// Physical device (GPU)
pub struct PhysicalDevice {
    handle: VkPhysicalDevice,
    properties: PhysicalDeviceProperties,
    memory_properties: VkPhysicalDeviceMemoryProperties,
    queue_families: Vec<VkQueueFamilyProperties>,
}

// Thread-safe version of device properties
pub struct PhysicalDeviceProperties {
    pub api_version: u32,
    pub driver_version: u32,
    pub vendor_id: u32,
    pub device_id: u32,
    pub device_type: VkPhysicalDeviceType,
    pub device_name: String,
    pub pipeline_cache_uuid: [u8; 16],
    pub limits: VkPhysicalDeviceLimits,
    pub sparse_properties: VkPhysicalDeviceSparseProperties,
}

/// Logical device
pub struct Device {
    handle: VkDevice,
    physical_device: Arc<PhysicalDevice>,
    queues: HashMap<(u32, u32), VkQueue>, // (family, index) -> queue
    memory_allocations: HashMap<u64, MemoryAllocation>,
    buffers: HashMap<u64, Buffer>,
}

/// Memory allocation
pub struct MemoryAllocation {
    handle: VkDeviceMemory,
    size: VkDeviceSize,
    memory_type: u32,
    mapped_ptr: Option<*mut u8>,
}

// Memory allocations can be sent between threads
unsafe impl Send for MemoryAllocation {}
unsafe impl Sync for MemoryAllocation {}

/// Buffer
pub struct Buffer {
    handle: VkBuffer,
    size: VkDeviceSize,
    usage: VkBufferUsageFlags,
    memory: Option<VkDeviceMemory>,
    offset: VkDeviceSize,
}