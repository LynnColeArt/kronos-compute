//! Tests for the flags module

use kronos_compute::core::flags::*;

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