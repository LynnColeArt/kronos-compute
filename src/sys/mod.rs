//! System types and handles for Kronos
//! 
//! Type-safe handle system with zero overhead

use std::marker::PhantomData;
use std::fmt;

/// Opaque handle type with phantom data for type safety
#[repr(transparent)]
#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub struct Handle<T> {
    pub(crate) raw: u64,
    pub(crate) _marker: PhantomData<*const T>,
}

impl<T> Handle<T> {
    pub const NULL: Self = Self {
        raw: 0,
        _marker: PhantomData,
    };
    
    #[inline]
    pub const fn from_raw(raw: u64) -> Self {
        Self {
            raw,
            _marker: PhantomData,
        }
    }
    
    #[inline]
    pub const fn as_raw(&self) -> u64 {
        self.raw
    }
    
    #[inline]
    pub const fn is_null(&self) -> bool {
        self.raw == 0
    }
}

impl<T> fmt::Debug for Handle<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Handle")
            .field("raw", &self.raw)
            .field("_marker", &self._marker)
            .finish()
    }
}

// SAFETY: Handle<T> is safe to send between threads because:
// 1. It contains only a u64 value (no references or pointers)
// 2. The phantom data doesn't affect thread safety
// 3. Vulkan handles are designed to be thread-safe at the API level
unsafe impl<T> Send for Handle<T> {}
unsafe impl<T> Sync for Handle<T> {}

// Define opaque types
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum InstanceT {}
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum PhysicalDeviceT {}
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum DeviceT {}
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum QueueT {}
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum CommandBufferT {}
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum BufferT {}
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum DeviceMemoryT {}
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum PipelineT {}
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum PipelineLayoutT {}
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ShaderModuleT {}
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum DescriptorSetT {}
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum DescriptorSetLayoutT {}
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum DescriptorPoolT {}
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum CommandPoolT {}
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum FenceT {}
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum SemaphoreT {}
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum EventT {}
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum PipelineCacheT {}
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum SamplerT {}
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ImageViewT {}

// Type aliases for handles
pub type VkInstance = Handle<InstanceT>;
pub type VkPhysicalDevice = Handle<PhysicalDeviceT>;
pub type VkDevice = Handle<DeviceT>;
pub type VkQueue = Handle<QueueT>;
pub type VkCommandBuffer = Handle<CommandBufferT>;
pub type VkBuffer = Handle<BufferT>;
pub type VkDeviceMemory = Handle<DeviceMemoryT>;
pub type VkPipeline = Handle<PipelineT>;
pub type VkPipelineLayout = Handle<PipelineLayoutT>;
pub type VkShaderModule = Handle<ShaderModuleT>;
pub type VkDescriptorSet = Handle<DescriptorSetT>;
pub type VkDescriptorSetLayout = Handle<DescriptorSetLayoutT>;
pub type VkDescriptorPool = Handle<DescriptorPoolT>;
pub type VkCommandPool = Handle<CommandPoolT>;
pub type VkFence = Handle<FenceT>;
pub type VkSemaphore = Handle<SemaphoreT>;
pub type VkEvent = Handle<EventT>;
pub type VkPipelineCache = Handle<PipelineCacheT>;
pub type VkSampler = Handle<SamplerT>;
pub type VkImageView = Handle<ImageViewT>;

// Basic types
pub type VkBool32 = u32;
pub type VkFlags = u32;
pub type VkDeviceSize = u64;

// Constants
pub const VK_TRUE: VkBool32 = 1;
pub const VK_FALSE: VkBool32 = 0;
pub const VK_WHOLE_SIZE: VkDeviceSize = !0;
pub const VK_QUEUE_FAMILY_IGNORED: u32 = !0;

// API version
pub const VK_API_VERSION_1_0: u32 = (1 << 22) | (0 << 12) | 0;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_handle_null() {
        let handle: VkDevice = Handle::NULL;
        assert!(handle.is_null());
        assert_eq!(handle.as_raw(), 0);
    }

    #[test]
    fn test_handle_creation() {
        let handle: VkBuffer = Handle::from_raw(42);
        assert!(!handle.is_null());
        assert_eq!(handle.as_raw(), 42);
    }

    #[test]
    fn test_handle_equality() {
        let h1: VkPipeline = Handle::from_raw(123);
        let h2: VkPipeline = Handle::from_raw(123);
        let h3: VkPipeline = Handle::from_raw(456);
        
        assert_eq!(h1, h2);
        assert_ne!(h1, h3);
    }

    #[test]
    fn test_handle_copy() {
        let h1: VkQueue = Handle::from_raw(789);
        let h2 = h1; // Copy
        assert_eq!(h1, h2);
        assert_eq!(h1.as_raw(), h2.as_raw());
    }

    #[test]
    fn test_constants() {
        assert_eq!(VK_TRUE, 1);
        assert_eq!(VK_FALSE, 0);
        assert_eq!(VK_WHOLE_SIZE, u64::MAX);
        assert_eq!(VK_QUEUE_FAMILY_IGNORED, u32::MAX);
    }

    #[test]
    fn test_handle_debug() {
        let handle: VkDevice = Handle::from_raw(999);
        let debug_str = format!("{:?}", handle);
        assert!(debug_str.contains("raw: 999"));
    }
}