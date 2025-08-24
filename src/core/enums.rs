//! Enumerations for Kronos API

/// Structure type identifiers
#[repr(i32)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum VkStructureType {
    ApplicationInfo = 0,
    InstanceCreateInfo = 1,
    DeviceQueueCreateInfo = 2,
    DeviceCreateInfo = 3,
    SubmitInfo = 4,
    MemoryAllocateInfo = 5,
    MappedMemoryRange = 6,
    FenceCreateInfo = 8,
    SemaphoreCreateInfo = 9,
    EventCreateInfo = 10,
    QueryPoolCreateInfo = 11,
    BufferCreateInfo = 12,
    PipelineShaderStageCreateInfo = 18,
    ComputePipelineCreateInfo = 29,
    PipelineLayoutCreateInfo = 30,
    DescriptorSetLayoutCreateInfo = 32,
    DescriptorPoolCreateInfo = 33,
    DescriptorSetAllocateInfo = 34,
    WriteDescriptorSet = 35,
    CopyDescriptorSet = 36,
    ShaderModuleCreateInfo = 38,
    CommandPoolCreateInfo = 39,
    CommandBufferAllocateInfo = 40,
    CommandBufferBeginInfo = 42,
    BufferMemoryBarrier = 44,
    MemoryBarrier = 46,
    PipelineCacheCreateInfo = 47,
    // Timeline semaphore extensions
    SemaphoreTypeCreateInfo = 1000207002,
    TimelineSemaphoreSubmitInfo = 1000207003,
    SemaphoreWaitInfo = 1000207004,
}

/// Queue capability flags
#[repr(u32)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum VkQueueFlagBits {
    Compute = 0x00000002,
    Transfer = 0x00000004,
    SparseBinding = 0x00000008,
}

/// Memory property flags
#[repr(u32)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum VkMemoryPropertyFlagBits {
    DeviceLocal = 0x00000001,
    HostVisible = 0x00000002,
    HostCoherent = 0x00000004,
    HostCached = 0x00000008,
    LazilyAllocated = 0x00000010,
}

/// Buffer usage flags
#[repr(u32)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum VkBufferUsageFlagBits {
    TransferSrc = 0x00000001,
    TransferDst = 0x00000002,
    UniformBuffer = 0x00000010,
    StorageBuffer = 0x00000020,
    IndexBuffer = 0x00000040,
    VertexBuffer = 0x00000080,
    IndirectBuffer = 0x00000100,
}

/// Sharing mode
#[repr(i32)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum VkSharingMode {
    Exclusive = 0,
    Concurrent = 1,
}

/// Pipeline bind point
#[repr(i32)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum VkPipelineBindPoint {
    Compute = 1,
}

/// Command buffer level
#[repr(i32)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum VkCommandBufferLevel {
    Primary = 0,
    Secondary = 1,
}

/// Command buffer usage flags
#[repr(u32)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum VkCommandBufferUsageFlagBits {
    OneTimeSubmit = 0x00000001,
    SimultaneousUse = 0x00000004,
}

/// Shader stage flags
#[repr(u32)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum VkShaderStageFlagBits {
    Compute = 0x00000020,
}

/// Descriptor type (compute-relevant only)
#[repr(i32)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum VkDescriptorType {
    Sampler = 0,
    SampledImage = 2,
    StorageImage = 3,
    UniformBuffer = 6,
    StorageBuffer = 7,
    UniformBufferDynamic = 8,
    StorageBufferDynamic = 9,
}

/// Pipeline stage flags
#[repr(u32)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum VkPipelineStageFlagBits {
    TopOfPipe = 0x00000001,
    ComputeShader = 0x00000800,
    BottomOfPipe = 0x00002000,
    Host = 0x00004000,
    AllCommands = 0x00010000,
}

/// Access flags
#[repr(u32)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum VkAccessFlagBits {
    IndirectCommandRead = 0x00000001,
    IndexRead = 0x00000002,
    VertexAttributeRead = 0x00000004,
    UniformRead = 0x00000008,
    InputAttachmentRead = 0x00000010,
    ShaderRead = 0x00000020,
    ShaderWrite = 0x00000040,
    TransferRead = 0x00000800,
    TransferWrite = 0x00001000,
    HostRead = 0x00002000,
    HostWrite = 0x00004000,
    MemoryRead = 0x00008000,
    MemoryWrite = 0x00010000,
}

/// Semaphore type
#[repr(i32)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum VkSemaphoreType {
    Binary = 0,
    Timeline = 1,
}

/// Physical device type
#[repr(i32)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum VkPhysicalDeviceType {
    Other = 0,
    IntegratedGpu = 1,
    DiscreteGpu = 2,
    VirtualGpu = 3,
    Cpu = 4,
}