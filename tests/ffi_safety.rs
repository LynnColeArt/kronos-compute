//! FFI safety tests - ensures bitflags are properly repr(C) or repr(transparent)

use kronos::core::flags::*;
use std::mem;

// Macro to test that a type is FFI-safe
macro_rules! assert_ffi_safe {
    ($type:ty) => {
        // Size should be same as underlying u32
        assert_eq!(mem::size_of::<$type>(), mem::size_of::<u32>());
        // Alignment should be same as u32
        assert_eq!(mem::align_of::<$type>(), mem::align_of::<u32>());
    };
}

#[test]
fn test_bitflags_ffi_safety() {
    // All bitflags exposed through FFI must be same size/alignment as u32
    assert_ffi_safe!(VkQueueFlags);
    assert_ffi_safe!(VkMemoryPropertyFlags);
    assert_ffi_safe!(VkBufferUsageFlags);
    assert_ffi_safe!(VkBufferCreateFlags);
    assert_ffi_safe!(VkCommandBufferUsageFlags);
    assert_ffi_safe!(VkCommandPoolCreateFlags);
    assert_ffi_safe!(VkShaderStageFlags);
    assert_ffi_safe!(VkPipelineStageFlags);
    assert_ffi_safe!(VkAccessFlags);
    assert_ffi_safe!(VkPipelineCreateFlags);
    assert_ffi_safe!(VkDescriptorPoolCreateFlags);
    assert_ffi_safe!(VkDescriptorPoolResetFlags);
    assert_ffi_safe!(VkFenceCreateFlags);
    assert_ffi_safe!(VkDependencyFlags);
    assert_ffi_safe!(VkSemaphoreWaitFlags);
    assert_ffi_safe!(VkPipelineShaderStageCreateFlags);
}

#[test]
fn test_type_aliases_ffi_safety() {
    // Type aliases should be transparent u32
    assert_eq!(mem::size_of::<VkInstanceCreateFlags>(), mem::size_of::<u32>());
    assert_eq!(mem::size_of::<VkDeviceCreateFlags>(), mem::size_of::<u32>());
    assert_eq!(mem::size_of::<VkDeviceQueueCreateFlags>(), mem::size_of::<u32>());
    assert_eq!(mem::size_of::<VkMemoryMapFlags>(), mem::size_of::<u32>());
    assert_eq!(mem::size_of::<VkSemaphoreCreateFlags>(), mem::size_of::<u32>());
    assert_eq!(mem::size_of::<VkEventCreateFlags>(), mem::size_of::<u32>());
    assert_eq!(mem::size_of::<VkQueryPoolCreateFlags>(), mem::size_of::<u32>());
    assert_eq!(mem::size_of::<VkPipelineLayoutCreateFlags>(), mem::size_of::<u32>());
    assert_eq!(mem::size_of::<VkDescriptorSetLayoutCreateFlags>(), mem::size_of::<u32>());
}

#[test]
fn test_bitflags_operations() {
    // Ensure bitflags operations work correctly
    let compute = VkQueueFlags::COMPUTE;
    let transfer = VkQueueFlags::TRANSFER;
    let combined = compute | transfer;
    
    assert!(combined.contains(compute));
    assert!(combined.contains(transfer));
    assert!(!combined.contains(VkQueueFlags::SPARSE_BINDING));
    
    // Test conversion to/from bits
    assert_eq!(compute.bits(), 0x00000002);
    assert_eq!(transfer.bits(), 0x00000004);
    assert_eq!(combined.bits(), 0x00000006);
    
    // Test from_bits
    assert_eq!(VkQueueFlags::from_bits(0x00000002), Some(VkQueueFlags::COMPUTE));
    assert_eq!(VkQueueFlags::from_bits(0xFFFFFFFF), None); // Invalid bits
}