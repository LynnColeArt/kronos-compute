//! Pipeline and command buffer implementation

use crate::sys::*;
use crate::core::*;
use crate::ffi::*;

/// Create shader module
#[no_mangle]
pub unsafe extern "C" fn vkCreateShaderModule(
    device: VkDevice,
    pCreateInfo: *const VkShaderModuleCreateInfo,
    pAllocator: *const VkAllocationCallbacks,
    pShaderModule: *mut VkShaderModule,
) -> VkResult {
    if device.is_null() || pCreateInfo.is_null() || pShaderModule.is_null() {
        return VkResult::ErrorInitializationFailed;
    }
    
    // Forward to real ICD
    if let Some(icd) = super::forward::get_icd_if_enabled() {
        if let Some(create_shader_module) = icd.create_shader_module {
            return create_shader_module(device, pCreateInfo, pAllocator, pShaderModule);
        }
    }
    
    // No ICD available
    VkResult::ErrorInitializationFailed
}

/// Destroy shader module
#[no_mangle]
pub unsafe extern "C" fn vkDestroyShaderModule(
    device: VkDevice,
    shaderModule: VkShaderModule,
    pAllocator: *const VkAllocationCallbacks,
) {
    if device.is_null() || shaderModule.is_null() {
        return;
    }
    
    // Forward to real ICD
    if let Some(icd) = super::forward::get_icd_if_enabled() {
        if let Some(destroy_shader_module) = icd.destroy_shader_module {
            destroy_shader_module(device, shaderModule, pAllocator);
        }
    }
}

/// Create compute pipelines
#[no_mangle]
pub unsafe extern "C" fn vkCreateComputePipelines(
    device: VkDevice,
    pipelineCache: VkPipelineCache,
    createInfoCount: u32,
    pCreateInfos: *const VkComputePipelineCreateInfo,
    pAllocator: *const VkAllocationCallbacks,
    pPipelines: *mut VkPipeline,
) -> VkResult {
    if device.is_null() || pCreateInfos.is_null() || pPipelines.is_null() || createInfoCount == 0 {
        return VkResult::ErrorInitializationFailed;
    }
    
    // Forward to real driver
    if let Some(icd) = super::forward::get_icd_if_enabled() {
        if let Some(create_compute_pipelines) = icd.create_compute_pipelines {
            return create_compute_pipelines(device, pipelineCache, createInfoCount, pCreateInfos, pAllocator, pPipelines);
        }
    }
    
    // No ICD available
    VkResult::ErrorInitializationFailed
}

/// Destroy pipeline
#[no_mangle]
pub unsafe extern "C" fn vkDestroyPipeline(
    device: VkDevice,
    pipeline: VkPipeline,
    pAllocator: *const VkAllocationCallbacks,
) {
    if device.is_null() || pipeline.is_null() {
        return;
    }
    
    // Forward to real ICD
    if let Some(icd) = super::forward::get_icd_if_enabled() {
        if let Some(destroy_pipeline) = icd.destroy_pipeline {
            destroy_pipeline(device, pipeline, pAllocator);
        }
    }
}

/// Create pipeline layout
#[no_mangle]
pub unsafe extern "C" fn vkCreatePipelineLayout(
    device: VkDevice,
    pCreateInfo: *const VkPipelineLayoutCreateInfo,
    pAllocator: *const VkAllocationCallbacks,
    pPipelineLayout: *mut VkPipelineLayout,
) -> VkResult {
    if device.is_null() || pCreateInfo.is_null() || pPipelineLayout.is_null() {
        return VkResult::ErrorInitializationFailed;
    }
    
    // Forward to real ICD
    if let Some(icd) = super::forward::get_icd_if_enabled() {
        if let Some(create_pipeline_layout) = icd.create_pipeline_layout {
            return create_pipeline_layout(device, pCreateInfo, pAllocator, pPipelineLayout);
        }
    }
    
    // No ICD available
    VkResult::ErrorInitializationFailed
}

/// Destroy pipeline layout
#[no_mangle]
pub unsafe extern "C" fn vkDestroyPipelineLayout(
    device: VkDevice,
    pipelineLayout: VkPipelineLayout,
    pAllocator: *const VkAllocationCallbacks,
) {
    if device.is_null() || pipelineLayout.is_null() {
        return;
    }
    
    // Forward to real ICD
    if let Some(icd) = super::forward::get_icd_if_enabled() {
        if let Some(destroy_pipeline_layout) = icd.destroy_pipeline_layout {
            destroy_pipeline_layout(device, pipelineLayout, pAllocator);
        }
    }
}

/// Create command pool
#[no_mangle]
pub unsafe extern "C" fn vkCreateCommandPool(
    device: VkDevice,
    pCreateInfo: *const VkCommandPoolCreateInfo,
    pAllocator: *const VkAllocationCallbacks,
    pCommandPool: *mut VkCommandPool,
) -> VkResult {
    if device.is_null() || pCreateInfo.is_null() || pCommandPool.is_null() {
        return VkResult::ErrorInitializationFailed;
    }
    
    // Forward to real ICD
    if let Some(icd) = super::forward::get_icd_if_enabled() {
        if let Some(create_command_pool) = icd.create_command_pool {
            return create_command_pool(device, pCreateInfo, pAllocator, pCommandPool);
        }
    }
    
    // No ICD available
    VkResult::ErrorInitializationFailed
}

/// Destroy command pool
#[no_mangle]
pub unsafe extern "C" fn vkDestroyCommandPool(
    device: VkDevice,
    commandPool: VkCommandPool,
    pAllocator: *const VkAllocationCallbacks,
) {
    if device.is_null() || commandPool.is_null() {
        return;
    }
    
    // Forward to real ICD
    if let Some(icd) = super::forward::get_icd_if_enabled() {
        if let Some(destroy_command_pool) = icd.destroy_command_pool {
            destroy_command_pool(device, commandPool, pAllocator);
        }
    }
}

/// Allocate command buffers
#[no_mangle]
pub unsafe extern "C" fn vkAllocateCommandBuffers(
    device: VkDevice,
    pAllocateInfo: *const VkCommandBufferAllocateInfo,
    pCommandBuffers: *mut VkCommandBuffer,
) -> VkResult {
    if device.is_null() || pAllocateInfo.is_null() || pCommandBuffers.is_null() {
        return VkResult::ErrorInitializationFailed;
    }
    
    // Forward to real ICD
    if let Some(icd) = super::forward::get_icd_if_enabled() {
        if let Some(allocate_command_buffers) = icd.allocate_command_buffers {
            return allocate_command_buffers(device, pAllocateInfo, pCommandBuffers);
        }
    }
    
    // No ICD available
    VkResult::ErrorInitializationFailed
}

/// Free command buffers
#[no_mangle]
pub unsafe extern "C" fn vkFreeCommandBuffers(
    device: VkDevice,
    commandPool: VkCommandPool,
    commandBufferCount: u32,
    pCommandBuffers: *const VkCommandBuffer,
) {
    if device.is_null() || commandPool.is_null() || pCommandBuffers.is_null() || commandBufferCount == 0 {
        return;
    }
    
    // Forward to real ICD
    if let Some(icd) = super::forward::get_icd_if_enabled() {
        if let Some(free_command_buffers) = icd.free_command_buffers {
            free_command_buffers(device, commandPool, commandBufferCount, pCommandBuffers);
        }
    }
}

/// Begin command buffer recording
#[no_mangle]
pub unsafe extern "C" fn vkBeginCommandBuffer(
    commandBuffer: VkCommandBuffer,
    pBeginInfo: *const VkCommandBufferBeginInfo,
) -> VkResult {
    if commandBuffer.is_null() || pBeginInfo.is_null() {
        return VkResult::ErrorInitializationFailed;
    }
    
    // Forward to real ICD
    if let Some(icd) = super::forward::get_icd_if_enabled() {
        if let Some(begin_command_buffer) = icd.begin_command_buffer {
            return begin_command_buffer(commandBuffer, pBeginInfo);
        }
    }
    
    // No ICD available
    VkResult::ErrorInitializationFailed
}

/// End command buffer recording
#[no_mangle]
pub unsafe extern "C" fn vkEndCommandBuffer(
    commandBuffer: VkCommandBuffer,
) -> VkResult {
    if commandBuffer.is_null() {
        return VkResult::ErrorInitializationFailed;
    }
    
    // Forward to real ICD
    if let Some(icd) = super::forward::get_icd_if_enabled() {
        if let Some(end_command_buffer) = icd.end_command_buffer {
            return end_command_buffer(commandBuffer);
        }
    }
    
    // No ICD available
    VkResult::ErrorInitializationFailed
}

/// Bind pipeline
#[no_mangle]
pub unsafe extern "C" fn vkCmdBindPipeline(
    commandBuffer: VkCommandBuffer,
    pipelineBindPoint: VkPipelineBindPoint,
    pipeline: VkPipeline,
) {
    if commandBuffer.is_null() || pipeline.is_null() {
        return;
    }
    
    // Forward to real ICD
    if let Some(icd) = super::forward::get_icd_if_enabled() {
        if let Some(cmd_bind_pipeline) = icd.cmd_bind_pipeline {
            cmd_bind_pipeline(commandBuffer, pipelineBindPoint, pipeline);
        }
    }
}

/// Bind descriptor sets
#[no_mangle]
pub unsafe extern "C" fn vkCmdBindDescriptorSets(
    commandBuffer: VkCommandBuffer,
    pipelineBindPoint: VkPipelineBindPoint,
    layout: VkPipelineLayout,
    firstSet: u32,
    descriptorSetCount: u32,
    pDescriptorSets: *const VkDescriptorSet,
    dynamicOffsetCount: u32,
    pDynamicOffsets: *const u32,
) {
    if commandBuffer.is_null() || layout.is_null() || pDescriptorSets.is_null() || descriptorSetCount == 0 {
        return;
    }
    
    // Forward to real ICD
    if let Some(icd) = super::forward::get_icd_if_enabled() {
        if let Some(cmd_bind_descriptor_sets) = icd.cmd_bind_descriptor_sets {
            cmd_bind_descriptor_sets(commandBuffer, pipelineBindPoint, layout, firstSet, 
                                   descriptorSetCount, pDescriptorSets, dynamicOffsetCount, pDynamicOffsets);
        }
    }
}

/// Dispatch compute work
#[no_mangle]
pub unsafe extern "C" fn vkCmdDispatch(
    commandBuffer: VkCommandBuffer,
    groupCountX: u32,
    groupCountY: u32,
    groupCountZ: u32,
) {
    if commandBuffer.is_null() {
        return;
    }
    
    // Forward to real driver
    if let Some(icd) = super::forward::get_icd_if_enabled() {
        if let Some(cmd_dispatch) = icd.cmd_dispatch {
            cmd_dispatch(commandBuffer, groupCountX, groupCountY, groupCountZ);
        }
    }
}

/// Dispatch compute work with indirect buffer
#[no_mangle]
pub unsafe extern "C" fn vkCmdDispatchIndirect(
    commandBuffer: VkCommandBuffer,
    buffer: VkBuffer,
    offset: VkDeviceSize,
) {
    if commandBuffer.is_null() || buffer.is_null() {
        return;
    }
    
    // Forward to real driver
    if let Some(icd) = super::forward::get_icd_if_enabled() {
        if let Some(cmd_dispatch_indirect) = icd.cmd_dispatch_indirect {
            cmd_dispatch_indirect(commandBuffer, buffer, offset);
        }
    }
}

/// Pipeline barrier
#[no_mangle]
pub unsafe extern "C" fn vkCmdPipelineBarrier(
    commandBuffer: VkCommandBuffer,
    srcStageMask: VkPipelineStageFlags,
    dstStageMask: VkPipelineStageFlags,
    dependencyFlags: VkDependencyFlags,
    memoryBarrierCount: u32,
    pMemoryBarriers: *const VkMemoryBarrier,
    bufferMemoryBarrierCount: u32,
    pBufferMemoryBarriers: *const VkBufferMemoryBarrier,
    imageMemoryBarrierCount: u32,
    pImageMemoryBarriers: *const libc::c_void,
) {
    if commandBuffer.is_null() {
        return;
    }
    
    // Forward to real ICD
    if let Some(icd) = super::forward::get_icd_if_enabled() {
        if let Some(cmd_pipeline_barrier) = icd.cmd_pipeline_barrier {
            cmd_pipeline_barrier(commandBuffer, srcStageMask, dstStageMask, dependencyFlags,
                               memoryBarrierCount, pMemoryBarriers, bufferMemoryBarrierCount,
                               pBufferMemoryBarriers, imageMemoryBarrierCount, pImageMemoryBarriers);
        }
    }
}

/// Copy buffer
#[no_mangle]
pub unsafe extern "C" fn vkCmdCopyBuffer(
    commandBuffer: VkCommandBuffer,
    srcBuffer: VkBuffer,
    dstBuffer: VkBuffer,
    regionCount: u32,
    pRegions: *const VkBufferCopy,
) {
    if commandBuffer.is_null() || srcBuffer.is_null() || dstBuffer.is_null() || 
       regionCount == 0 || pRegions.is_null() {
        return;
    }
    
    // Forward to real ICD
    if let Some(icd) = super::forward::get_icd_if_enabled() {
        if let Some(cmd_copy_buffer) = icd.cmd_copy_buffer {
            cmd_copy_buffer(commandBuffer, srcBuffer, dstBuffer, regionCount, pRegions);
        }
    }
}

/// Set event
#[no_mangle]
pub unsafe extern "C" fn vkCmdSetEvent(
    commandBuffer: VkCommandBuffer,
    event: VkEvent,
    stageMask: VkPipelineStageFlags,
) {
    if commandBuffer.is_null() || event.is_null() {
        return;
    }
    
    // Forward to real ICD
    if let Some(icd) = super::forward::get_icd_if_enabled() {
        if let Some(cmd_set_event) = icd.cmd_set_event {
            cmd_set_event(commandBuffer, event, stageMask);
        }
    }
}

/// Reset event
#[no_mangle]
pub unsafe extern "C" fn vkCmdResetEvent(
    commandBuffer: VkCommandBuffer,
    event: VkEvent,
    stageMask: VkPipelineStageFlags,
) {
    if commandBuffer.is_null() || event.is_null() {
        return;
    }
    
    // Forward to real ICD
    if let Some(icd) = super::forward::get_icd_if_enabled() {
        if let Some(cmd_reset_event) = icd.cmd_reset_event {
            cmd_reset_event(commandBuffer, event, stageMask);
        }
    }
}

/// Wait for events
#[no_mangle]
pub unsafe extern "C" fn vkCmdWaitEvents(
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
    pImageMemoryBarriers: *const libc::c_void,
) {
    if commandBuffer.is_null() || eventCount == 0 || pEvents.is_null() {
        return;
    }
    
    // Forward to real ICD
    if let Some(icd) = super::forward::get_icd_if_enabled() {
        if let Some(cmd_wait_events) = icd.cmd_wait_events {
            cmd_wait_events(commandBuffer, eventCount, pEvents, srcStageMask, dstStageMask,
                          memoryBarrierCount, pMemoryBarriers, bufferMemoryBarrierCount,
                          pBufferMemoryBarriers, imageMemoryBarrierCount, pImageMemoryBarriers);
        }
    }
}