//! Tests to verify structure sizes and layouts

use kronos_compute::*;
use std::mem;

#[test]
fn test_handle_sizes() {
    // All handles should be 8 bytes (64-bit)
    assert_eq!(mem::size_of::<VkInstance>(), 8);
    assert_eq!(mem::size_of::<VkDevice>(), 8);
    assert_eq!(mem::size_of::<VkBuffer>(), 8);
    assert_eq!(mem::size_of::<VkPipeline>(), 8);
}

#[test]
fn test_basic_types() {
    assert_eq!(mem::size_of::<VkFlags>(), 4);
    assert_eq!(mem::size_of::<VkBool32>(), 4);
    assert_eq!(mem::size_of::<VkDeviceSize>(), 8);
}

#[test]
fn test_structure_sizes() {
    // Verify optimized structure sizes
    assert_eq!(mem::size_of::<VkApplicationInfo>(), 48);
    assert_eq!(mem::size_of::<VkInstanceCreateInfo>(), 64);
    assert_eq!(mem::size_of::<VkDeviceCreateInfo>(), 72);
    
    // Optimized structures
    assert_eq!(mem::size_of::<VkPhysicalDeviceFeatures>(), 32); // 8 * 4 bytes
    assert_eq!(mem::size_of::<VkMemoryTypeCache>(), 16); // 4 * 4 bytes
    
    // Compute structures
    assert_eq!(mem::size_of::<VkComputePipelineCreateInfo>(), 96);
    assert_eq!(mem::size_of::<VkPipelineShaderStageCreateInfo>(), 48);
}

#[test]
fn test_alignment() {
    // Ensure proper alignment for FFI
    assert_eq!(mem::align_of::<VkApplicationInfo>(), 8);
    assert_eq!(mem::align_of::<VkBufferCreateInfo>(), 8);
    assert_eq!(mem::align_of::<VkSubmitInfo>(), 8);
}

#[test]
fn test_enum_values() {
    // Verify enum values match C API
    assert_eq!(VkResult::Success as i32, 0);
    assert_eq!(VkResult::ErrorOutOfHostMemory as i32, -1);
    
    assert_eq!(VkStructureType::ApplicationInfo as i32, 0);
    assert_eq!(VkStructureType::InstanceCreateInfo as i32, 1);
    assert_eq!(VkStructureType::ComputePipelineCreateInfo as i32, 29);
}

#[test]
fn test_flag_values() {
    assert_eq!(VkQueueFlags::COMPUTE.bits(), 0x00000002);
    assert_eq!(VkQueueFlags::TRANSFER.bits(), 0x00000004);
    
    assert_eq!(VkMemoryPropertyFlags::DEVICE_LOCAL.bits(), 0x00000001);
    assert_eq!(VkMemoryPropertyFlags::HOST_VISIBLE.bits(), 0x00000002);
    assert_eq!(VkMemoryPropertyFlags::HOST_COHERENT.bits(), 0x00000004);
}

#[test]
fn test_constants() {
    assert_eq!(VK_TRUE, 1);
    assert_eq!(VK_FALSE, 0);
    assert_eq!(VK_WHOLE_SIZE, !0u64);
    assert_eq!(VK_QUEUE_FAMILY_IGNORED, !0u32);
}

#[test]
fn test_default_values() {
    let app_info = VkApplicationInfo::default();
    assert_eq!(app_info.sType, VkStructureType::ApplicationInfo);
    assert!(app_info.pNext.is_null());
    assert_eq!(app_info.apiVersion, VK_API_VERSION_1_0);
    
    let buffer_info = VkBufferCreateInfo::default();
    assert_eq!(buffer_info.sType, VkStructureType::BufferCreateInfo);
    assert_eq!(buffer_info.sharingMode, VkSharingMode::Exclusive);
}

#[test]
fn test_memory_type_cache() {
    let cache = VkMemoryTypeCache::default();
    assert_eq!(cache.hostVisibleCoherent, 0);
    assert_eq!(cache.deviceLocal, 0);
    assert_eq!(cache.hostVisibleCached, 0);
    assert_eq!(cache.deviceLocalLazy, 0);
}

#[test]
fn test_extent3d() {
    let extent = VkExtent3D::default();
    assert_eq!(extent.width, 1);
    assert_eq!(extent.height, 1);
    assert_eq!(extent.depth, 1);
}