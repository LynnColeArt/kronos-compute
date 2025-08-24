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
        const INDEX_BUFFER = 0x00000040;
        const VERTEX_BUFFER = 0x00000080;
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
    #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
    pub struct VkPipelineStageFlags: VkFlags {
        const TOP_OF_PIPE = 0x00000001;
        const COMPUTE_SHADER = 0x00000800;
        const BOTTOM_OF_PIPE = 0x00002000;
        const HOST = 0x00004000;
        const ALL_GRAPHICS = 0x00008000;
        const ALL_COMMANDS = 0x00010000;
    }
}

bitflags! {
    #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
    pub struct VkAccessFlags: VkFlags {
        const INDIRECT_COMMAND_READ = 0x00000001;
        const INDEX_READ = 0x00000002;
        const VERTEX_ATTRIBUTE_READ = 0x00000004;
        const UNIFORM_READ = 0x00000008;
        const INPUT_ATTACHMENT_READ = 0x00000010;
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
    pub struct VkFenceCreateFlags: VkFlags {
        const SIGNALED = 0x00000001;
    }
}

bitflags! {
    #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
    pub struct VkDependencyFlags: VkFlags {
        const BY_REGION = 0x00000001;
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