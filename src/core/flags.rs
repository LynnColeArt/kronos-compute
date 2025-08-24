//! Bitflag types for Kronos API

use bitflags::bitflags;
use crate::sys::VkFlags;

bitflags! {
    #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
    pub struct VkQueueFlags: VkFlags {
        const COMPUTE = 0x00000002;
        const TRANSFER = 0x00000004;
        const SPARSE_BINDING = 0x00000008;
    }
}

bitflags! {
    #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
    pub struct VkMemoryPropertyFlags: VkFlags {
        const DEVICE_LOCAL = 0x00000001;
        const HOST_VISIBLE = 0x00000002;
        const HOST_COHERENT = 0x00000004;
        const HOST_CACHED = 0x00000008;
        const LAZILY_ALLOCATED = 0x00000010;
    }
}

bitflags! {
    #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
    pub struct VkBufferUsageFlags: VkFlags {
        const TRANSFER_SRC = 0x00000001;
        const TRANSFER_DST = 0x00000002;
        const UNIFORM_BUFFER = 0x00000010;
        const STORAGE_BUFFER = 0x00000020;
        const INDIRECT_BUFFER = 0x00000100;
    }
}

bitflags! {
    #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
    pub struct VkBufferCreateFlags: VkFlags {
        const SPARSE_BINDING = 0x00000001;
        const SPARSE_RESIDENCY = 0x00000002;
        const SPARSE_ALIASED = 0x00000004;
    }
}

bitflags! {
    #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
    pub struct VkCommandBufferUsageFlags: VkFlags {
        const ONE_TIME_SUBMIT = 0x00000001;
        const RENDER_PASS_CONTINUE = 0x00000002;
        const SIMULTANEOUS_USE = 0x00000004;
    }
}

bitflags! {
    #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
    pub struct VkCommandPoolCreateFlags: VkFlags {
        const TRANSIENT = 0x00000001;
        const RESET_COMMAND_BUFFER = 0x00000002;
    }
}

bitflags! {
    #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
    pub struct VkShaderStageFlags: VkFlags {
        const COMPUTE = 0x00000020;
        const ALL = 0x7FFFFFFF;
    }
}

bitflags! {
    #[repr(transparent)]
    #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
    pub struct VkPipelineStageFlags: VkFlags {
        const TOP_OF_PIPE = 0x00000001;
        const COMPUTE_SHADER = 0x00000800;
        const BOTTOM_OF_PIPE = 0x00002000;
        const HOST = 0x00004000;
        const ALL_COMMANDS = 0x00010000;
    }
}

bitflags! {
    #[repr(transparent)]
    #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
    pub struct VkAccessFlags: VkFlags {
        const INDIRECT_COMMAND_READ = 0x00000001;
        const UNIFORM_READ = 0x00000008;
        const SHADER_READ = 0x00000020;
        const SHADER_WRITE = 0x00000040;
        const TRANSFER_READ = 0x00000800;
        const TRANSFER_WRITE = 0x00001000;
        const HOST_READ = 0x00002000;
        const HOST_WRITE = 0x00004000;
        const MEMORY_READ = 0x00008000;
        const MEMORY_WRITE = 0x00010000;
    }
}

bitflags! {
    #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
    pub struct VkPipelineCreateFlags: VkFlags {
        const DISABLE_OPTIMIZATION = 0x00000001;
        const ALLOW_DERIVATIVES = 0x00000002;
        const DERIVATIVE = 0x00000004;
    }
}

bitflags! {
    #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
    pub struct VkDescriptorPoolCreateFlags: VkFlags {
        const FREE_DESCRIPTOR_SET = 0x00000001;
        const UPDATE_AFTER_BIND = 0x00000002;
    }
}

bitflags! {
    #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
    pub struct VkDescriptorPoolResetFlags: VkFlags {
        // Reserved for future use
        const RESERVED = 0;
    }
}

bitflags! {
    #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
    pub struct VkFenceCreateFlags: VkFlags {
        const SIGNALED = 0x00000001;
    }
}

bitflags! {
    #[repr(transparent)]
    #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
    pub struct VkDependencyFlags: VkFlags {
        const BY_REGION = 0x00000001;
    }
}

bitflags! {
    #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
    pub struct VkSemaphoreWaitFlags: VkFlags {
        const ANY = 0x00000001;
    }
}

bitflags! {
    #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
    pub struct VkPipelineShaderStageCreateFlags: VkFlags {
        // Reserved for future use
        const RESERVED = 0;
    }
}

// Type aliases for flags that don't have specific bits
pub type VkInstanceCreateFlags = VkFlags;
pub type VkDeviceCreateFlags = VkFlags;
pub type VkDeviceQueueCreateFlags = VkFlags;
pub type VkMemoryMapFlags = VkFlags;
pub type VkSemaphoreCreateFlags = VkFlags;
pub type VkEventCreateFlags = VkFlags;
pub type VkQueryPoolCreateFlags = VkFlags;
pub type VkPipelineLayoutCreateFlags = VkFlags;
pub type VkDescriptorSetLayoutCreateFlags = VkFlags;

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_queue_flags() {
        let compute_transfer = VkQueueFlags::COMPUTE | VkQueueFlags::TRANSFER;
        assert!(compute_transfer.contains(VkQueueFlags::COMPUTE));
        assert!(compute_transfer.contains(VkQueueFlags::TRANSFER));
        assert!(!compute_transfer.contains(VkQueueFlags::SPARSE_BINDING));
        
        let all = VkQueueFlags::all();
        assert!(all.contains(VkQueueFlags::COMPUTE));
        assert!(all.contains(VkQueueFlags::TRANSFER));
        assert!(all.contains(VkQueueFlags::SPARSE_BINDING));
    }
    
    #[test]
    fn test_memory_property_flags() {
        let host_visible = VkMemoryPropertyFlags::HOST_VISIBLE | VkMemoryPropertyFlags::HOST_COHERENT;
        assert!(host_visible.contains(VkMemoryPropertyFlags::HOST_VISIBLE));
        assert!(host_visible.contains(VkMemoryPropertyFlags::HOST_COHERENT));
        assert!(!host_visible.contains(VkMemoryPropertyFlags::DEVICE_LOCAL));
        
        assert!(VkMemoryPropertyFlags::empty().is_empty());
    }
    
    #[test]
    fn test_buffer_usage_flags() {
        let storage_transfer = VkBufferUsageFlags::STORAGE_BUFFER | VkBufferUsageFlags::TRANSFER_DST;
        assert!(storage_transfer.contains(VkBufferUsageFlags::STORAGE_BUFFER));
        assert!(storage_transfer.contains(VkBufferUsageFlags::TRANSFER_DST));
        assert!(!storage_transfer.contains(VkBufferUsageFlags::UNIFORM_BUFFER));
        
        // Test intersection
        let transfer_only = VkBufferUsageFlags::TRANSFER_SRC | VkBufferUsageFlags::TRANSFER_DST;
        let intersection = storage_transfer & transfer_only;
        assert_eq!(intersection, VkBufferUsageFlags::TRANSFER_DST);
    }
    
    #[test]
    fn test_pipeline_stage_flags() {
        let compute_stage = VkPipelineStageFlags::COMPUTE_SHADER;
        assert!(compute_stage.contains(VkPipelineStageFlags::COMPUTE_SHADER));
        assert!(!compute_stage.contains(VkPipelineStageFlags::HOST));
        
        let all_commands = VkPipelineStageFlags::ALL_COMMANDS;
        assert!(!all_commands.is_empty());
    }
    
    #[test]
    fn test_access_flags() {
        let shader_access = VkAccessFlags::SHADER_READ | VkAccessFlags::SHADER_WRITE;
        assert!(shader_access.contains(VkAccessFlags::SHADER_READ));
        assert!(shader_access.contains(VkAccessFlags::SHADER_WRITE));
        assert!(!shader_access.contains(VkAccessFlags::HOST_READ));
        
        // Test removal
        let read_only = shader_access - VkAccessFlags::SHADER_WRITE;
        assert_eq!(read_only, VkAccessFlags::SHADER_READ);
    }
    
    #[test]
    fn test_shader_stage_flags() {
        let compute = VkShaderStageFlags::COMPUTE;
        assert!(compute.contains(VkShaderStageFlags::COMPUTE));
        assert!(!compute.contains(VkShaderStageFlags::ALL));
        
        let all = VkShaderStageFlags::ALL;
        assert!(all.contains(VkShaderStageFlags::COMPUTE));
    }
    
    #[test]
    fn test_fence_create_flags() {
        let signaled = VkFenceCreateFlags::SIGNALED;
        assert!(!signaled.is_empty());
        assert!(signaled.contains(VkFenceCreateFlags::SIGNALED));
        
        let empty = VkFenceCreateFlags::empty();
        assert!(empty.is_empty());
    }
}