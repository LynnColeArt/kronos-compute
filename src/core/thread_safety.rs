//! Thread safety implementations for Vulkan structures
//! 
//! Vulkan structures contain pNext pointers for extension chains.
//! These are *const c_void pointers that are typically null or point to
//! other Vulkan structures. They are safe to send between threads because:
//! 1. They are read-only (const pointers)
//! 2. They point to immutable data during structure lifetime
//! 3. Vulkan API guarantees thread safety for these structures

use super::structs::*;
use super::compute::*;

// Application info structures
unsafe impl Send for VkApplicationInfo {}
unsafe impl Sync for VkApplicationInfo {}

unsafe impl Send for VkInstanceCreateInfo {}
unsafe impl Sync for VkInstanceCreateInfo {}

// Device structures
unsafe impl Send for VkDeviceQueueCreateInfo {}
unsafe impl Sync for VkDeviceQueueCreateInfo {}

unsafe impl Send for VkDeviceCreateInfo {}
unsafe impl Sync for VkDeviceCreateInfo {}

// Memory structures
unsafe impl Send for VkMemoryAllocateInfo {}
unsafe impl Sync for VkMemoryAllocateInfo {}

// Buffer structures
unsafe impl Send for VkBufferCreateInfo {}
unsafe impl Sync for VkBufferCreateInfo {}

// Shader structures
unsafe impl Send for VkShaderModuleCreateInfo {}
unsafe impl Sync for VkShaderModuleCreateInfo {}

unsafe impl Send for VkPipelineShaderStageCreateInfo {}
unsafe impl Sync for VkPipelineShaderStageCreateInfo {}

// Pipeline structures
unsafe impl Send for VkComputePipelineCreateInfo {}
unsafe impl Sync for VkComputePipelineCreateInfo {}

unsafe impl Send for VkPipelineLayoutCreateInfo {}
unsafe impl Sync for VkPipelineLayoutCreateInfo {}

// Command structures
unsafe impl Send for VkCommandPoolCreateInfo {}
unsafe impl Sync for VkCommandPoolCreateInfo {}

unsafe impl Send for VkCommandBufferAllocateInfo {}
unsafe impl Sync for VkCommandBufferAllocateInfo {}

unsafe impl Send for VkCommandBufferBeginInfo {}
unsafe impl Sync for VkCommandBufferBeginInfo {}

unsafe impl Send for VkSubmitInfo {}
unsafe impl Sync for VkSubmitInfo {}

unsafe impl Send for VkBufferCopy {}
unsafe impl Sync for VkBufferCopy {}

// Barrier structures
unsafe impl Send for VkMemoryBarrier {}
unsafe impl Sync for VkMemoryBarrier {}

unsafe impl Send for VkBufferMemoryBarrier {}
unsafe impl Sync for VkBufferMemoryBarrier {}

// Descriptor structures
unsafe impl Send for VkDescriptorSetLayoutBinding {}
unsafe impl Sync for VkDescriptorSetLayoutBinding {}

unsafe impl Send for VkDescriptorSetLayoutCreateInfo {}
unsafe impl Sync for VkDescriptorSetLayoutCreateInfo {}

// Query results structures
unsafe impl Send for VkQueueFamilyProperties {}
unsafe impl Sync for VkQueueFamilyProperties {}

unsafe impl Send for VkMemoryRequirements {}
unsafe impl Sync for VkMemoryRequirements {}

unsafe impl Send for VkPhysicalDeviceMemoryProperties {}
unsafe impl Sync for VkPhysicalDeviceMemoryProperties {}

unsafe impl Send for VkMemoryType {}
unsafe impl Sync for VkMemoryType {}

unsafe impl Send for VkMemoryHeap {}
unsafe impl Sync for VkMemoryHeap {}

unsafe impl Send for VkPhysicalDeviceFeatures {}
unsafe impl Sync for VkPhysicalDeviceFeatures {}

// Push constant range
unsafe impl Send for VkPushConstantRange {}
unsafe impl Sync for VkPushConstantRange {}

// Synchronization structures
unsafe impl Send for VkFenceCreateInfo {}
unsafe impl Sync for VkFenceCreateInfo {}

unsafe impl Send for VkSemaphoreCreateInfo {}
unsafe impl Sync for VkSemaphoreCreateInfo {}

unsafe impl Send for VkEventCreateInfo {}
unsafe impl Sync for VkEventCreateInfo {}

// Specialization structures
unsafe impl Send for VkSpecializationMapEntry {}
unsafe impl Sync for VkSpecializationMapEntry {}

unsafe impl Send for VkSpecializationInfo {}
unsafe impl Sync for VkSpecializationInfo {}

// Descriptor structures
unsafe impl Send for VkDescriptorPoolSize {}
unsafe impl Sync for VkDescriptorPoolSize {}

unsafe impl Send for VkDescriptorPoolCreateInfo {}
unsafe impl Sync for VkDescriptorPoolCreateInfo {}

unsafe impl Send for VkDescriptorSetAllocateInfo {}
unsafe impl Sync for VkDescriptorSetAllocateInfo {}

unsafe impl Send for VkDescriptorBufferInfo {}
unsafe impl Sync for VkDescriptorBufferInfo {}

unsafe impl Send for VkDescriptorImageInfo {}
unsafe impl Sync for VkDescriptorImageInfo {}

unsafe impl Send for VkWriteDescriptorSet {}
unsafe impl Sync for VkWriteDescriptorSet {}

unsafe impl Send for VkCopyDescriptorSet {}
unsafe impl Sync for VkCopyDescriptorSet {}

