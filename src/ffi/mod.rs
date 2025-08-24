//! Foreign Function Interface for Kronos
//! 
//! This module provides C-compatible function signatures for interop

use std::ffi::{c_char, c_void};
use crate::sys::*;
use crate::core::*;

/// Result codes for Kronos API operations
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

/// Allocation callbacks (optional)
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct VkAllocationCallbacks {
    pub pUserData: *mut c_void,
    pub pfnAllocation: Option<unsafe extern "C" fn(*mut c_void, usize, usize, VkSystemAllocationScope) -> *mut c_void>,
    pub pfnReallocation: Option<unsafe extern "C" fn(*mut c_void, *mut c_void, usize, usize, VkSystemAllocationScope) -> *mut c_void>,
    pub pfnFree: Option<unsafe extern "C" fn(*mut c_void, *mut c_void)>,
    pub pfnInternalAllocation: Option<unsafe extern "C" fn(*mut c_void, usize, VkInternalAllocationType, VkSystemAllocationScope)>,
    pub pfnInternalFree: Option<unsafe extern "C" fn(*mut c_void, usize, VkInternalAllocationType, VkSystemAllocationScope)>,
}

// VkAllocationCallbacks contains function pointers that are safe to send between threads
unsafe impl Send for VkAllocationCallbacks {}
unsafe impl Sync for VkAllocationCallbacks {}

#[repr(i32)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum VkSystemAllocationScope {
    Command = 0,
    Object = 1,
    Cache = 2,
    Device = 3,
    Instance = 4,
}

#[repr(i32)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum VkInternalAllocationType {
    Executable = 0,
}

// Function pointer types
pub type PFN_vkVoidFunction = Option<unsafe extern "C" fn()>;

// Core function pointers
pub type PFN_vkGetInstanceProcAddr = Option<unsafe extern "C" fn(
    instance: VkInstance,
    pName: *const c_char,
) -> PFN_vkVoidFunction>;

pub type PFN_vkGetDeviceProcAddr = Option<unsafe extern "C" fn(
    device: VkDevice,
    pName: *const c_char,
) -> PFN_vkVoidFunction>;

// Instance functions
pub type PFN_vkCreateInstance = Option<unsafe extern "C" fn(
    pCreateInfo: *const VkInstanceCreateInfo,
    pAllocator: *const VkAllocationCallbacks,
    pInstance: *mut VkInstance,
) -> VkResult>;

pub type PFN_vkDestroyInstance = Option<unsafe extern "C" fn(
    instance: VkInstance,
    pAllocator: *const VkAllocationCallbacks,
)>;

pub type PFN_vkEnumeratePhysicalDevices = Option<unsafe extern "C" fn(
    instance: VkInstance,
    pPhysicalDeviceCount: *mut u32,
    pPhysicalDevices: *mut VkPhysicalDevice,
) -> VkResult>;

pub type PFN_vkGetPhysicalDeviceProperties = Option<unsafe extern "C" fn(
    physicalDevice: VkPhysicalDevice,
    pProperties: *mut VkPhysicalDeviceProperties,
)>;

pub type PFN_vkGetPhysicalDeviceQueueFamilyProperties = Option<unsafe extern "C" fn(
    physicalDevice: VkPhysicalDevice,
    pQueueFamilyPropertyCount: *mut u32,
    pQueueFamilyProperties: *mut VkQueueFamilyProperties,
)>;

pub type PFN_vkGetPhysicalDeviceMemoryProperties = Option<unsafe extern "C" fn(
    physicalDevice: VkPhysicalDevice,
    pMemoryProperties: *mut VkPhysicalDeviceMemoryProperties,
)>;

pub type PFN_vkGetPhysicalDeviceFeatures = Option<unsafe extern "C" fn(
    physicalDevice: VkPhysicalDevice,
    pFeatures: *mut VkPhysicalDeviceFeatures,
)>;

// Device functions
pub type PFN_vkCreateDevice = Option<unsafe extern "C" fn(
    physicalDevice: VkPhysicalDevice,
    pCreateInfo: *const VkDeviceCreateInfo,
    pAllocator: *const VkAllocationCallbacks,
    pDevice: *mut VkDevice,
) -> VkResult>;

pub type PFN_vkDestroyDevice = Option<unsafe extern "C" fn(
    device: VkDevice,
    pAllocator: *const VkAllocationCallbacks,
)>;

pub type PFN_vkGetDeviceQueue = Option<unsafe extern "C" fn(
    device: VkDevice,
    queueFamilyIndex: u32,
    queueIndex: u32,
    pQueue: *mut VkQueue,
)>;

pub type PFN_vkQueueSubmit = Option<unsafe extern "C" fn(
    queue: VkQueue,
    submitCount: u32,
    pSubmits: *const VkSubmitInfo,
    fence: VkFence,
) -> VkResult>;

pub type PFN_vkQueueWaitIdle = Option<unsafe extern "C" fn(
    queue: VkQueue,
) -> VkResult>;

pub type PFN_vkDeviceWaitIdle = Option<unsafe extern "C" fn(
    device: VkDevice,
) -> VkResult>;

// Memory functions
pub type PFN_vkAllocateMemory = Option<unsafe extern "C" fn(
    device: VkDevice,
    pAllocateInfo: *const VkMemoryAllocateInfo,
    pAllocator: *const VkAllocationCallbacks,
    pMemory: *mut VkDeviceMemory,
) -> VkResult>;

pub type PFN_vkFreeMemory = Option<unsafe extern "C" fn(
    device: VkDevice,
    memory: VkDeviceMemory,
    pAllocator: *const VkAllocationCallbacks,
)>;

pub type PFN_vkMapMemory = Option<unsafe extern "C" fn(
    device: VkDevice,
    memory: VkDeviceMemory,
    offset: VkDeviceSize,
    size: VkDeviceSize,
    flags: VkMemoryMapFlags,
    ppData: *mut *mut c_void,
) -> VkResult>;

pub type PFN_vkUnmapMemory = Option<unsafe extern "C" fn(
    device: VkDevice,
    memory: VkDeviceMemory,
)>;

// Buffer functions
pub type PFN_vkCreateBuffer = Option<unsafe extern "C" fn(
    device: VkDevice,
    pCreateInfo: *const VkBufferCreateInfo,
    pAllocator: *const VkAllocationCallbacks,
    pBuffer: *mut VkBuffer,
) -> VkResult>;

pub type PFN_vkDestroyBuffer = Option<unsafe extern "C" fn(
    device: VkDevice,
    buffer: VkBuffer,
    pAllocator: *const VkAllocationCallbacks,
)>;

pub type PFN_vkGetBufferMemoryRequirements = Option<unsafe extern "C" fn(
    device: VkDevice,
    buffer: VkBuffer,
    pMemoryRequirements: *mut VkMemoryRequirements,
)>;

pub type PFN_vkBindBufferMemory = Option<unsafe extern "C" fn(
    device: VkDevice,
    buffer: VkBuffer,
    memory: VkDeviceMemory,
    memoryOffset: VkDeviceSize,
) -> VkResult>;

// Command functions
pub type PFN_vkCreateCommandPool = Option<unsafe extern "C" fn(
    device: VkDevice,
    pCreateInfo: *const VkCommandPoolCreateInfo,
    pAllocator: *const VkAllocationCallbacks,
    pCommandPool: *mut VkCommandPool,
) -> VkResult>;

pub type PFN_vkDestroyCommandPool = Option<unsafe extern "C" fn(
    device: VkDevice,
    commandPool: VkCommandPool,
    pAllocator: *const VkAllocationCallbacks,
)>;

pub type PFN_vkAllocateCommandBuffers = Option<unsafe extern "C" fn(
    device: VkDevice,
    pAllocateInfo: *const VkCommandBufferAllocateInfo,
    pCommandBuffers: *mut VkCommandBuffer,
) -> VkResult>;

pub type PFN_vkFreeCommandBuffers = Option<unsafe extern "C" fn(
    device: VkDevice,
    commandPool: VkCommandPool,
    commandBufferCount: u32,
    pCommandBuffers: *const VkCommandBuffer,
)>;

pub type PFN_vkBeginCommandBuffer = Option<unsafe extern "C" fn(
    commandBuffer: VkCommandBuffer,
    pBeginInfo: *const VkCommandBufferBeginInfo,
) -> VkResult>;

pub type PFN_vkEndCommandBuffer = Option<unsafe extern "C" fn(
    commandBuffer: VkCommandBuffer,
) -> VkResult>;

// Command buffer commands
pub type PFN_vkCmdCopyBuffer = Option<unsafe extern "C" fn(
    commandBuffer: VkCommandBuffer,
    srcBuffer: VkBuffer,
    dstBuffer: VkBuffer,
    regionCount: u32,
    pRegions: *const VkBufferCopy,
)>;

pub type PFN_vkCmdPipelineBarrier = Option<unsafe extern "C" fn(
    commandBuffer: VkCommandBuffer,
    srcStageMask: VkPipelineStageFlags,
    dstStageMask: VkPipelineStageFlags,
    dependencyFlags: VkDependencyFlags,
    memoryBarrierCount: u32,
    pMemoryBarriers: *const VkMemoryBarrier,
    bufferMemoryBarrierCount: u32,
    pBufferMemoryBarriers: *const VkBufferMemoryBarrier,
    imageMemoryBarrierCount: u32,
    pImageMemoryBarriers: *const c_void, // We don't support images
)>;

pub type PFN_vkCmdBindPipeline = Option<unsafe extern "C" fn(
    commandBuffer: VkCommandBuffer,
    pipelineBindPoint: VkPipelineBindPoint,
    pipeline: VkPipeline,
)>;

pub type PFN_vkCmdBindDescriptorSets = Option<unsafe extern "C" fn(
    commandBuffer: VkCommandBuffer,
    pipelineBindPoint: VkPipelineBindPoint,
    layout: VkPipelineLayout,
    firstSet: u32,
    descriptorSetCount: u32,
    pDescriptorSets: *const VkDescriptorSet,
    dynamicOffsetCount: u32,
    pDynamicOffsets: *const u32,
)>;

pub type PFN_vkCmdDispatch = Option<unsafe extern "C" fn(
    commandBuffer: VkCommandBuffer,
    groupCountX: u32,
    groupCountY: u32,
    groupCountZ: u32,
)>;

// Synchronization functions
pub type PFN_vkCreateFence = Option<unsafe extern "C" fn(
    device: VkDevice,
    pCreateInfo: *const VkFenceCreateInfo,
    pAllocator: *const VkAllocationCallbacks,
    pFence: *mut VkFence,
) -> VkResult>;

pub type PFN_vkDestroyFence = Option<unsafe extern "C" fn(
    device: VkDevice,
    fence: VkFence,
    pAllocator: *const VkAllocationCallbacks,
)>;

pub type PFN_vkResetFences = Option<unsafe extern "C" fn(
    device: VkDevice,
    fenceCount: u32,
    pFences: *const VkFence,
) -> VkResult>;

pub type PFN_vkGetFenceStatus = Option<unsafe extern "C" fn(
    device: VkDevice,
    fence: VkFence,
) -> VkResult>;

pub type PFN_vkWaitForFences = Option<unsafe extern "C" fn(
    device: VkDevice,
    fenceCount: u32,
    pFences: *const VkFence,
    waitAll: VkBool32,
    timeout: u64,
) -> VkResult>;

pub type PFN_vkCreateSemaphore = Option<unsafe extern "C" fn(
    device: VkDevice,
    pCreateInfo: *const VkSemaphoreCreateInfo,
    pAllocator: *const VkAllocationCallbacks,
    pSemaphore: *mut VkSemaphore,
) -> VkResult>;

pub type PFN_vkDestroySemaphore = Option<unsafe extern "C" fn(
    device: VkDevice,
    semaphore: VkSemaphore,
    pAllocator: *const VkAllocationCallbacks,
)>;

pub type PFN_vkCreateEvent = Option<unsafe extern "C" fn(
    device: VkDevice,
    pCreateInfo: *const VkEventCreateInfo,
    pAllocator: *const VkAllocationCallbacks,
    pEvent: *mut VkEvent,
) -> VkResult>;

pub type PFN_vkDestroyEvent = Option<unsafe extern "C" fn(
    device: VkDevice,
    event: VkEvent,
    pAllocator: *const VkAllocationCallbacks,
)>;

pub type PFN_vkGetEventStatus = Option<unsafe extern "C" fn(
    device: VkDevice,
    event: VkEvent,
) -> VkResult>;

pub type PFN_vkSetEvent = Option<unsafe extern "C" fn(
    device: VkDevice,
    event: VkEvent,
) -> VkResult>;

pub type PFN_vkResetEvent = Option<unsafe extern "C" fn(
    device: VkDevice,
    event: VkEvent,
) -> VkResult>;

pub type PFN_vkCmdSetEvent = Option<unsafe extern "C" fn(
    commandBuffer: VkCommandBuffer,
    event: VkEvent,
    stageMask: VkPipelineStageFlags,
)>;

pub type PFN_vkCmdResetEvent = Option<unsafe extern "C" fn(
    commandBuffer: VkCommandBuffer,
    event: VkEvent,
    stageMask: VkPipelineStageFlags,
)>;

pub type PFN_vkCmdWaitEvents = Option<unsafe extern "C" fn(
    commandBuffer: VkCommandBuffer,
    eventCount: u32,
    pEvents: *const VkEvent,
    srcStageMask: VkPipelineStageFlags,
    dstStageMask: VkPipelineStageFlags,
    memoryBarrierCount: u32,
    pMemoryBarriers: *const VkMemoryBarrier,
    bufferMemoryBarrierCount: u32,
    pBufferMemoryBarriers: *const VkBufferMemoryBarrier,
    imageMemoryBarrierCount: u32,
    pImageMemoryBarriers: *const c_void,
)>;

pub type PFN_vkCmdDispatchIndirect = Option<unsafe extern "C" fn(
    commandBuffer: VkCommandBuffer,
    buffer: VkBuffer,
    offset: VkDeviceSize,
)>;

// Compute pipeline functions
pub type PFN_vkCreateShaderModule = Option<unsafe extern "C" fn(
    device: VkDevice,
    pCreateInfo: *const VkShaderModuleCreateInfo,
    pAllocator: *const VkAllocationCallbacks,
    pShaderModule: *mut VkShaderModule,
) -> VkResult>;

pub type PFN_vkDestroyShaderModule = Option<unsafe extern "C" fn(
    device: VkDevice,
    shaderModule: VkShaderModule,
    pAllocator: *const VkAllocationCallbacks,
)>;

pub type PFN_vkCreateComputePipelines = Option<unsafe extern "C" fn(
    device: VkDevice,
    pipelineCache: VkPipelineCache,
    createInfoCount: u32,
    pCreateInfos: *const VkComputePipelineCreateInfo,
    pAllocator: *const VkAllocationCallbacks,
    pPipelines: *mut VkPipeline,
) -> VkResult>;

pub type PFN_vkDestroyPipeline = Option<unsafe extern "C" fn(
    device: VkDevice,
    pipeline: VkPipeline,
    pAllocator: *const VkAllocationCallbacks,
)>;

pub type PFN_vkCreatePipelineLayout = Option<unsafe extern "C" fn(
    device: VkDevice,
    pCreateInfo: *const VkPipelineLayoutCreateInfo,
    pAllocator: *const VkAllocationCallbacks,
    pPipelineLayout: *mut VkPipelineLayout,
) -> VkResult>;

pub type PFN_vkDestroyPipelineLayout = Option<unsafe extern "C" fn(
    device: VkDevice,
    pipelineLayout: VkPipelineLayout,
    pAllocator: *const VkAllocationCallbacks,
)>;

// Descriptor functions
pub type PFN_vkCreateDescriptorSetLayout = Option<unsafe extern "C" fn(
    device: VkDevice,
    pCreateInfo: *const VkDescriptorSetLayoutCreateInfo,
    pAllocator: *const VkAllocationCallbacks,
    pSetLayout: *mut VkDescriptorSetLayout,
) -> VkResult>;

pub type PFN_vkDestroyDescriptorSetLayout = Option<unsafe extern "C" fn(
    device: VkDevice,
    descriptorSetLayout: VkDescriptorSetLayout,
    pAllocator: *const VkAllocationCallbacks,
)>;

pub type PFN_vkCreateDescriptorPool = Option<unsafe extern "C" fn(
    device: VkDevice,
    pCreateInfo: *const VkDescriptorPoolCreateInfo,
    pAllocator: *const VkAllocationCallbacks,
    pDescriptorPool: *mut VkDescriptorPool,
) -> VkResult>;

pub type PFN_vkDestroyDescriptorPool = Option<unsafe extern "C" fn(
    device: VkDevice,
    descriptorPool: VkDescriptorPool,
    pAllocator: *const VkAllocationCallbacks,
)>;

pub type PFN_vkAllocateDescriptorSets = Option<unsafe extern "C" fn(
    device: VkDevice,
    pAllocateInfo: *const VkDescriptorSetAllocateInfo,
    pDescriptorSets: *mut VkDescriptorSet,
) -> VkResult>;

pub type PFN_vkUpdateDescriptorSets = Option<unsafe extern "C" fn(
    device: VkDevice,
    descriptorWriteCount: u32,
    pDescriptorWrites: *const VkWriteDescriptorSet,
    descriptorCopyCount: u32,
    pDescriptorCopies: *const VkCopyDescriptorSet,
)>;

// Add missing struct
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct VkPhysicalDeviceProperties {
    pub apiVersion: u32,
    pub driverVersion: u32,
    pub vendorID: u32,
    pub deviceID: u32,
    pub deviceType: VkPhysicalDeviceType,
    pub deviceName: [c_char; 256],
    pub pipelineCacheUUID: [u8; 16],
    pub limits: VkPhysicalDeviceLimits,
    pub sparseProperties: VkPhysicalDeviceSparseProperties,
}

#[repr(i32)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum VkPhysicalDeviceType {
    Other = 0,
    IntegratedGpu = 1,
    DiscreteGpu = 2,
    VirtualGpu = 3,
    Cpu = 4,
}

#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct VkPhysicalDeviceLimits {
    pub maxComputeSharedMemorySize: u32,
    pub maxComputeWorkGroupCount: [u32; 3],
    pub maxComputeWorkGroupInvocations: u32,
    pub maxComputeWorkGroupSize: [u32; 3],
    // ... many more limits, simplified for compute
}

#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct VkPhysicalDeviceSparseProperties {
    pub residencyStandard2DBlockShape: VkBool32,
    pub residencyStandard2DMultisampleBlockShape: VkBool32,
    pub residencyStandard3DBlockShape: VkBool32,
    pub residencyAlignedMipSize: VkBool32,
    pub residencyNonResidentStrict: VkBool32,
}