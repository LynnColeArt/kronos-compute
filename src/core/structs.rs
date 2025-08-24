//! Core structures for Kronos API

use std::ffi::{c_char, c_void};
use std::ptr;
use crate::sys::*;
use crate::core::enums::*;
use crate::core::flags::*;

/// Helper for null-terminated string pointers
pub type PtrCStr = *const c_char;

/// 3D extent
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct VkExtent3D {
    pub width: u32,
    pub height: u32,
    pub depth: u32,
}

impl Default for VkExtent3D {
    fn default() -> Self {
        Self {
            width: 1,
            height: 1,
            depth: 1,
        }
    }
}

/// Application information
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct VkApplicationInfo {
    pub sType: VkStructureType,
    pub pNext: *const c_void,
    pub pApplicationName: PtrCStr,
    pub applicationVersion: u32,
    pub pEngineName: PtrCStr,
    pub engineVersion: u32,
    pub apiVersion: u32,
}

impl Default for VkApplicationInfo {
    fn default() -> Self {
        Self {
            sType: VkStructureType::ApplicationInfo,
            pNext: ptr::null(),
            pApplicationName: ptr::null(),
            applicationVersion: 0,
            pEngineName: ptr::null(),
            engineVersion: 0,
            apiVersion: VK_API_VERSION_1_0,
        }
    }
}

/// Instance creation information
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct VkInstanceCreateInfo {
    pub sType: VkStructureType,
    pub pNext: *const c_void,
    pub flags: VkInstanceCreateFlags,
    pub pApplicationInfo: *const VkApplicationInfo,
    pub enabledLayerCount: u32,
    pub ppEnabledLayerNames: *const PtrCStr,
    pub enabledExtensionCount: u32,
    pub ppEnabledExtensionNames: *const PtrCStr,
}

impl Default for VkInstanceCreateInfo {
    fn default() -> Self {
        Self {
            sType: VkStructureType::InstanceCreateInfo,
            pNext: ptr::null(),
            flags: 0,
            pApplicationInfo: ptr::null(),
            enabledLayerCount: 0,
            ppEnabledLayerNames: ptr::null(),
            enabledExtensionCount: 0,
            ppEnabledExtensionNames: ptr::null(),
        }
    }
}

/// Queue family properties
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct VkQueueFamilyProperties {
    pub queueFlags: VkQueueFlags,
    pub queueCount: u32,
    pub timestampValidBits: u32,
    pub minImageTransferGranularity: VkExtent3D,
}

/// Physical device features (compute-relevant only)
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct VkPhysicalDeviceFeatures {
    pub robustBufferAccess: VkBool32,
    pub shaderFloat64: VkBool32,
    pub shaderInt64: VkBool32,
    pub shaderInt16: VkBool32,
    pub shaderStorageBufferArrayDynamicIndexing: VkBool32,
    pub shaderStorageImageArrayDynamicIndexing: VkBool32,
    pub shaderStorageImageReadWithoutFormat: VkBool32,
    pub shaderStorageImageWriteWithoutFormat: VkBool32,
}

impl Default for VkPhysicalDeviceFeatures {
    fn default() -> Self {
        Self {
            robustBufferAccess: VK_FALSE,
            shaderFloat64: VK_FALSE,
            shaderInt64: VK_FALSE,
            shaderInt16: VK_FALSE,
            shaderStorageBufferArrayDynamicIndexing: VK_FALSE,
            shaderStorageImageArrayDynamicIndexing: VK_FALSE,
            shaderStorageImageReadWithoutFormat: VK_FALSE,
            shaderStorageImageWriteWithoutFormat: VK_FALSE,
        }
    }
}

/// Device queue creation info
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct VkDeviceQueueCreateInfo {
    pub sType: VkStructureType,
    pub pNext: *const c_void,
    pub flags: VkDeviceQueueCreateFlags,
    pub queueFamilyIndex: u32,
    pub queueCount: u32,
    pub pQueuePriorities: *const f32,
}

impl Default for VkDeviceQueueCreateInfo {
    fn default() -> Self {
        Self {
            sType: VkStructureType::DeviceQueueCreateInfo,
            pNext: ptr::null(),
            flags: 0,
            queueFamilyIndex: 0,
            queueCount: 0,
            pQueuePriorities: ptr::null(),
        }
    }
}

/// Device creation info
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct VkDeviceCreateInfo {
    pub sType: VkStructureType,
    pub pNext: *const c_void,
    pub flags: VkDeviceCreateFlags,
    pub queueCreateInfoCount: u32,
    pub pQueueCreateInfos: *const VkDeviceQueueCreateInfo,
    pub enabledLayerCount: u32,
    pub ppEnabledLayerNames: *const PtrCStr,
    pub enabledExtensionCount: u32,
    pub ppEnabledExtensionNames: *const PtrCStr,
    pub pEnabledFeatures: *const VkPhysicalDeviceFeatures,
}

impl Default for VkDeviceCreateInfo {
    fn default() -> Self {
        Self {
            sType: VkStructureType::DeviceCreateInfo,
            pNext: ptr::null(),
            flags: 0,
            queueCreateInfoCount: 0,
            pQueueCreateInfos: ptr::null(),
            enabledLayerCount: 0,
            ppEnabledLayerNames: ptr::null(),
            enabledExtensionCount: 0,
            ppEnabledExtensionNames: ptr::null(),
            pEnabledFeatures: ptr::null(),
        }
    }
}

/// Memory type
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct VkMemoryType {
    pub propertyFlags: VkMemoryPropertyFlags,
    pub heapIndex: u32,
}

/// Memory heap
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct VkMemoryHeap {
    pub size: VkDeviceSize,
    pub flags: VkFlags,
}

/// Physical device memory properties
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct VkPhysicalDeviceMemoryProperties {
    pub memoryTypeCount: u32,
    pub memoryTypes: [VkMemoryType; VK_MAX_MEMORY_TYPES],
    pub memoryHeapCount: u32,
    pub memoryHeaps: [VkMemoryHeap; VK_MAX_MEMORY_HEAPS],
}

/// Memory type cache for O(1) lookups
#[repr(C)]
#[derive(Debug, Clone, Copy, Default)]
pub struct VkMemoryTypeCache {
    pub hostVisibleCoherent: u32,
    pub deviceLocal: u32,
    pub hostVisibleCached: u32,
    pub deviceLocalLazy: u32,
}

/// Memory allocate info
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct VkMemoryAllocateInfo {
    pub sType: VkStructureType,
    pub pNext: *const c_void,
    pub allocationSize: VkDeviceSize,
    pub memoryTypeIndex: u32,
}

impl Default for VkMemoryAllocateInfo {
    fn default() -> Self {
        Self {
            sType: VkStructureType::MemoryAllocateInfo,
            pNext: ptr::null(),
            allocationSize: 0,
            memoryTypeIndex: 0,
        }
    }
}

/// Memory requirements
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct VkMemoryRequirements {
    pub size: VkDeviceSize,
    pub alignment: VkDeviceSize,
    pub memoryTypeBits: u32,
}

/// Fence creation info
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct VkFenceCreateInfo {
    pub sType: VkStructureType,
    pub pNext: *const c_void,
    pub flags: VkFenceCreateFlags,
}

impl Default for VkFenceCreateInfo {
    fn default() -> Self {
        Self {
            sType: VkStructureType::FenceCreateInfo,
            pNext: ptr::null(),
            flags: VkFenceCreateFlags::empty(),
        }
    }
}

/// Semaphore creation info
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct VkSemaphoreCreateInfo {
    pub sType: VkStructureType,
    pub pNext: *const c_void,
    pub flags: VkFlags,
}

impl Default for VkSemaphoreCreateInfo {
    fn default() -> Self {
        Self {
            sType: VkStructureType::SemaphoreCreateInfo,
            pNext: ptr::null(),
            flags: 0,
        }
    }
}

/// Event creation info
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct VkEventCreateInfo {
    pub sType: VkStructureType,
    pub pNext: *const c_void,
    pub flags: VkFlags,
}

impl Default for VkEventCreateInfo {
    fn default() -> Self {
        Self {
            sType: VkStructureType::EventCreateInfo,
            pNext: ptr::null(),
            flags: 0,
        }
    }
}

/// Buffer creation info (optimized packing)
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct VkBufferCreateInfo {
    pub sType: VkStructureType,
    pub pNext: *const c_void,
    pub size: VkDeviceSize,
    pub usage: VkBufferUsageFlags,
    pub sharingMode: VkSharingMode,
    pub queueFamilyIndexCount: u32,
    pub pQueueFamilyIndices: *const u32,
    pub flags: VkBufferCreateFlags,
}

impl Default for VkBufferCreateInfo {
    fn default() -> Self {
        Self {
            sType: VkStructureType::BufferCreateInfo,
            pNext: ptr::null(),
            size: 0,
            usage: VkBufferUsageFlags::empty(),
            sharingMode: VkSharingMode::Exclusive,
            queueFamilyIndexCount: 0,
            pQueueFamilyIndices: ptr::null(),
            flags: VkBufferCreateFlags::empty(),
        }
    }
}

/// Command pool creation info
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct VkCommandPoolCreateInfo {
    pub sType: VkStructureType,
    pub pNext: *const c_void,
    pub flags: VkCommandPoolCreateFlags,
    pub queueFamilyIndex: u32,
}

impl Default for VkCommandPoolCreateInfo {
    fn default() -> Self {
        Self {
            sType: VkStructureType::CommandPoolCreateInfo,
            pNext: ptr::null(),
            flags: VkCommandPoolCreateFlags::empty(),
            queueFamilyIndex: 0,
        }
    }
}

/// Command buffer allocate info
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct VkCommandBufferAllocateInfo {
    pub sType: VkStructureType,
    pub pNext: *const c_void,
    pub commandPool: VkCommandPool,
    pub level: VkCommandBufferLevel,
    pub commandBufferCount: u32,
}

impl Default for VkCommandBufferAllocateInfo {
    fn default() -> Self {
        Self {
            sType: VkStructureType::CommandBufferAllocateInfo,
            pNext: ptr::null(),
            commandPool: VkCommandPool::NULL,
            level: VkCommandBufferLevel::Primary,
            commandBufferCount: 0,
        }
    }
}

/// Command buffer begin info
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct VkCommandBufferBeginInfo {
    pub sType: VkStructureType,
    pub pNext: *const c_void,
    pub flags: VkCommandBufferUsageFlags,
    pub pInheritanceInfo: *const c_void,
}

impl Default for VkCommandBufferBeginInfo {
    fn default() -> Self {
        Self {
            sType: VkStructureType::CommandBufferBeginInfo,
            pNext: ptr::null(),
            flags: VkCommandBufferUsageFlags::empty(),
            pInheritanceInfo: ptr::null(),
        }
    }
}

/// Memory barrier
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct VkMemoryBarrier {
    pub sType: VkStructureType,
    pub pNext: *const c_void,
    pub srcAccessMask: VkAccessFlags,
    pub dstAccessMask: VkAccessFlags,
}

impl Default for VkMemoryBarrier {
    fn default() -> Self {
        Self {
            sType: VkStructureType::MemoryBarrier,
            pNext: ptr::null(),
            srcAccessMask: VkAccessFlags::empty(),
            dstAccessMask: VkAccessFlags::empty(),
        }
    }
}

/// Buffer memory barrier
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct VkBufferMemoryBarrier {
    pub sType: VkStructureType,
    pub pNext: *const c_void,
    pub srcAccessMask: VkAccessFlags,
    pub dstAccessMask: VkAccessFlags,
    pub srcQueueFamilyIndex: u32,
    pub dstQueueFamilyIndex: u32,
    pub buffer: VkBuffer,
    pub offset: VkDeviceSize,
    pub size: VkDeviceSize,
}

impl Default for VkBufferMemoryBarrier {
    fn default() -> Self {
        Self {
            sType: VkStructureType::BufferMemoryBarrier,
            pNext: ptr::null(),
            srcAccessMask: VkAccessFlags::empty(),
            dstAccessMask: VkAccessFlags::empty(),
            srcQueueFamilyIndex: VK_QUEUE_FAMILY_IGNORED,
            dstQueueFamilyIndex: VK_QUEUE_FAMILY_IGNORED,
            buffer: VkBuffer::NULL,
            offset: 0,
            size: VK_WHOLE_SIZE,
        }
    }
}

/// Submit info
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct VkSubmitInfo {
    pub sType: VkStructureType,
    pub pNext: *const c_void,
    pub waitSemaphoreCount: u32,
    pub pWaitSemaphores: *const VkSemaphore,
    pub pWaitDstStageMask: *const VkPipelineStageFlags,
    pub commandBufferCount: u32,
    pub pCommandBuffers: *const VkCommandBuffer,
    pub signalSemaphoreCount: u32,
    pub pSignalSemaphores: *const VkSemaphore,
}

/// Buffer copy region
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct VkBufferCopy {
    pub srcOffset: VkDeviceSize,
    pub dstOffset: VkDeviceSize,
    pub size: VkDeviceSize,
}

impl Default for VkSubmitInfo {
    fn default() -> Self {
        Self {
            sType: VkStructureType::SubmitInfo,
            pNext: ptr::null(),
            waitSemaphoreCount: 0,
            pWaitSemaphores: ptr::null(),
            pWaitDstStageMask: ptr::null(),
            commandBufferCount: 0,
            pCommandBuffers: ptr::null(),
            signalSemaphoreCount: 0,
            pSignalSemaphores: ptr::null(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_extent3d_default() {
        let extent = VkExtent3D::default();
        assert_eq!(extent.width, 1);
        assert_eq!(extent.height, 1);
        assert_eq!(extent.depth, 1);
    }
    
    #[test]
    fn test_application_info_default() {
        let info = VkApplicationInfo::default();
        assert_eq!(info.sType, VkStructureType::ApplicationInfo);
        assert!(info.pNext.is_null());
        assert!(info.pApplicationName.is_null());
        assert_eq!(info.applicationVersion, 0);
        assert_eq!(info.apiVersion, VK_API_VERSION_1_0);
    }
    
    #[test]
    fn test_instance_create_info_default() {
        let info = VkInstanceCreateInfo::default();
        assert_eq!(info.sType, VkStructureType::InstanceCreateInfo);
        assert!(info.pNext.is_null());
        assert_eq!(info.flags, 0);
        assert_eq!(info.enabledLayerCount, 0);
        assert_eq!(info.enabledExtensionCount, 0);
    }
    
    #[test]
    fn test_buffer_create_info_default() {
        let info = VkBufferCreateInfo::default();
        assert_eq!(info.sType, VkStructureType::BufferCreateInfo);
        assert!(info.pNext.is_null());
        assert_eq!(info.size, 0);
        assert!(info.usage.is_empty());
        assert_eq!(info.sharingMode, VkSharingMode::Exclusive);
        assert!(info.flags.is_empty());
    }
    
    #[test]
    fn test_memory_allocate_info_default() {
        let info = VkMemoryAllocateInfo::default();
        assert_eq!(info.sType, VkStructureType::MemoryAllocateInfo);
        assert!(info.pNext.is_null());
        assert_eq!(info.allocationSize, 0);
        assert_eq!(info.memoryTypeIndex, 0);
    }
    
    #[test]
    fn test_fence_create_info_default() {
        let info = VkFenceCreateInfo::default();
        assert_eq!(info.sType, VkStructureType::FenceCreateInfo);
        assert!(info.pNext.is_null());
        assert!(info.flags.is_empty());
    }
    
    #[test]
    fn test_submit_info_default() {
        let info = VkSubmitInfo::default();
        assert_eq!(info.sType, VkStructureType::SubmitInfo);
        assert!(info.pNext.is_null());
        assert_eq!(info.waitSemaphoreCount, 0);
        assert_eq!(info.commandBufferCount, 0);
        assert_eq!(info.signalSemaphoreCount, 0);
    }
    
    #[test]
    fn test_command_buffer_begin_info_default() {
        let info = VkCommandBufferBeginInfo::default();
        assert_eq!(info.sType, VkStructureType::CommandBufferBeginInfo);
        assert!(info.pNext.is_null());
        assert!(info.flags.is_empty());
        assert!(info.pInheritanceInfo.is_null());
    }
    
    #[test]
    fn test_memory_barrier_default() {
        let barrier = VkMemoryBarrier::default();
        assert_eq!(barrier.sType, VkStructureType::MemoryBarrier);
        assert!(barrier.pNext.is_null());
        assert!(barrier.srcAccessMask.is_empty());
        assert!(barrier.dstAccessMask.is_empty());
    }
    
    #[test]
    fn test_buffer_memory_barrier_default() {
        let barrier = VkBufferMemoryBarrier::default();
        assert_eq!(barrier.sType, VkStructureType::BufferMemoryBarrier);
        assert!(barrier.pNext.is_null());
        assert_eq!(barrier.srcQueueFamilyIndex, VK_QUEUE_FAMILY_IGNORED);
        assert_eq!(barrier.dstQueueFamilyIndex, VK_QUEUE_FAMILY_IGNORED);
        assert_eq!(barrier.size, VK_WHOLE_SIZE);
    }
}