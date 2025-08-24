//! Low-level system types and constants
//! 
//! This module contains the raw FFI-compatible types that match the C API

use std::ffi::{c_char, c_void};
use std::fmt;

/// Vulkan-compatible handle type
pub type VkHandle = u64;

/// Opaque handle types
#[repr(transparent)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Handle<T> {
    raw: VkHandle,
    _marker: std::marker::PhantomData<T>,
}

impl<T> Handle<T> {
    pub const NULL: Self = Self {
        raw: 0,
        _marker: std::marker::PhantomData,
    };
    
    #[inline]
    pub fn from_raw(raw: VkHandle) -> Self {
        Self {
            raw,
            _marker: std::marker::PhantomData,
        }
    }
    
    #[inline]
    pub fn as_raw(&self) -> VkHandle {
        self.raw
    }
    
    #[inline]
    pub fn is_null(&self) -> bool {
        self.raw == 0
    }
}

// Define handle types - these are phantom types for type safety
#[derive(Debug, Clone, Copy)]
pub enum InstanceT {}
#[derive(Debug, Clone, Copy)]
pub enum PhysicalDeviceT {}
#[derive(Debug, Clone, Copy)]
pub enum DeviceT {}
#[derive(Debug, Clone, Copy)]
pub enum QueueT {}
#[derive(Debug, Clone, Copy)]
pub enum CommandBufferT {}
#[derive(Debug, Clone, Copy)]
pub enum BufferT {}
#[derive(Debug, Clone, Copy)]
pub enum DeviceMemoryT {}
#[derive(Debug, Clone, Copy)]
pub enum CommandPoolT {}
#[derive(Debug, Clone, Copy)]
pub enum PipelineT {}
#[derive(Debug, Clone, Copy)]
pub enum PipelineLayoutT {}
#[derive(Debug, Clone, Copy)]
pub enum DescriptorSetLayoutT {}
#[derive(Debug, Clone, Copy)]
pub enum DescriptorSetT {}
#[derive(Debug, Clone, Copy)]
pub enum DescriptorPoolT {}
#[derive(Debug, Clone, Copy)]
pub enum ShaderModuleT {}
#[derive(Debug, Clone, Copy)]
pub enum FenceT {}
#[derive(Debug, Clone, Copy)]
pub enum SemaphoreT {}
#[derive(Debug, Clone, Copy)]
pub enum EventT {}
#[derive(Debug, Clone, Copy)]
pub enum PipelineCacheT {}
#[derive(Debug, Clone, Copy)]
pub enum QueryPoolT {}

pub type VkInstance = Handle<InstanceT>;
pub type VkPhysicalDevice = Handle<PhysicalDeviceT>;
pub type VkDevice = Handle<DeviceT>;
pub type VkQueue = Handle<QueueT>;
pub type VkCommandBuffer = Handle<CommandBufferT>;
pub type VkBuffer = Handle<BufferT>;
pub type VkDeviceMemory = Handle<DeviceMemoryT>;
pub type VkCommandPool = Handle<CommandPoolT>;
pub type VkPipeline = Handle<PipelineT>;
pub type VkPipelineLayout = Handle<PipelineLayoutT>;
pub type VkDescriptorSetLayout = Handle<DescriptorSetLayoutT>;
pub type VkDescriptorSet = Handle<DescriptorSetT>;
pub type VkDescriptorPool = Handle<DescriptorPoolT>;
pub type VkShaderModule = Handle<ShaderModuleT>;
pub type VkFence = Handle<FenceT>;
pub type VkSemaphore = Handle<SemaphoreT>;
pub type VkEvent = Handle<EventT>;
pub type VkPipelineCache = Handle<PipelineCacheT>;
pub type VkQueryPool = Handle<QueryPoolT>;

/// Basic types
pub type VkFlags = u32;
pub type VkBool32 = u32;
pub type VkDeviceSize = u64;

/// Constants
pub const VK_TRUE: VkBool32 = 1;
pub const VK_FALSE: VkBool32 = 0;
pub const VK_WHOLE_SIZE: VkDeviceSize = !0;
pub const VK_ATTACHMENT_UNUSED: u32 = !0;
pub const VK_QUEUE_FAMILY_IGNORED: u32 = !0;
pub const VK_SUBPASS_EXTERNAL: u32 = !0;

/// API Version
pub const VK_API_VERSION_1_0: u32 = crate::make_version(1, 0, 0);

/// Result codes
#[repr(i32)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum VkResult {
    Success = 0,
    NotReady = 1,
    Timeout = 2,
    EventSet = 3,
    EventReset = 4,
    Incomplete = 5,
    ErrorOutOfHostMemory = -1,
    ErrorOutOfDeviceMemory = -2,
    ErrorInitializationFailed = -3,
    ErrorDeviceLost = -4,
    ErrorMemoryMapFailed = -5,
    ErrorLayerNotPresent = -6,
    ErrorExtensionNotPresent = -7,
    ErrorFeatureNotPresent = -8,
    ErrorIncompatibleDriver = -9,
    ErrorTooManyObjects = -10,
    ErrorFormatNotSupported = -11,
    ErrorFragmentedPool = -12,
    ErrorUnknown = -13,
    ErrorOutOfPoolMemory = -1000069000,
}

impl VkResult {
    #[inline]
    pub fn is_success(self) -> bool {
        self as i32 >= 0
    }
    
    #[inline]
    pub fn is_error(self) -> bool {
        (self as i32) < 0
    }
}

impl fmt::Display for VkResult {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            VkResult::Success => write!(f, "Success"),
            VkResult::NotReady => write!(f, "Not ready"),
            VkResult::Timeout => write!(f, "Timeout"),
            VkResult::EventSet => write!(f, "Event set"),
            VkResult::EventReset => write!(f, "Event reset"),
            VkResult::Incomplete => write!(f, "Incomplete"),
            VkResult::ErrorOutOfHostMemory => write!(f, "Out of host memory"),
            VkResult::ErrorOutOfDeviceMemory => write!(f, "Out of device memory"),
            VkResult::ErrorInitializationFailed => write!(f, "Initialization failed"),
            VkResult::ErrorDeviceLost => write!(f, "Device lost"),
            VkResult::ErrorMemoryMapFailed => write!(f, "Memory map failed"),
            VkResult::ErrorLayerNotPresent => write!(f, "Layer not present"),
            VkResult::ErrorExtensionNotPresent => write!(f, "Extension not present"),
            VkResult::ErrorFeatureNotPresent => write!(f, "Feature not present"),
            VkResult::ErrorIncompatibleDriver => write!(f, "Incompatible driver"),
            VkResult::ErrorTooManyObjects => write!(f, "Too many objects"),
            VkResult::ErrorFormatNotSupported => write!(f, "Format not supported"),
            VkResult::ErrorFragmentedPool => write!(f, "Fragmented pool"),
            VkResult::ErrorUnknown => write!(f, "Unknown error"),
            VkResult::ErrorOutOfPoolMemory => write!(f, "Out of pool memory"),
        }
    }
}

impl std::error::Error for VkResult {}