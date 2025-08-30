//! Pipeline and command buffer implementation

use crate::sys::*;
use crate::core::*;
use crate::ffi::*;
// keep single import
use crate::implementation::icd_loader;

/// Create shader module
// SAFETY: This function is called from C code. Caller must ensure:
// 1. device is a valid VkDevice
// 2. pCreateInfo points to a valid VkShaderModuleCreateInfo structure
// 3. pAllocator is either null or points to valid allocation callbacks
// 4. pShaderModule points to valid memory for writing the shader module handle
// 5. The SPIR-V code in pCreateInfo is valid and contains only compute shader stages
// 6. Code size matches the actual SPIR-V bytecode length
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
    
    if let Some(icd) = icd_loader::icd_for_device(device) {
        if let Some(f) = icd.create_shader_module { return f(device, pCreateInfo, pAllocator, pShaderModule); }
    }
    if let Some(icd) = super::forward::get_icd_if_enabled() {
        if let Some(create_shader_module) = icd.create_shader_module { return create_shader_module(device, pCreateInfo, pAllocator, pShaderModule); }
    }
    VkResult::ErrorInitializationFailed
}

/// Destroy shader module
// SAFETY: This function is called from C code. Caller must ensure:
// 1. device is a valid VkDevice
// 2. shaderModule is a valid VkShaderModule, or VK_NULL_HANDLE
// 3. pAllocator matches the allocator used in vkCreateShaderModule
// 4. No pipelines are currently using this shader module
// 5. The shader module is not referenced by any pending operations
#[no_mangle]
pub unsafe extern "C" fn vkDestroyShaderModule(
    device: VkDevice,
    shaderModule: VkShaderModule,
    pAllocator: *const VkAllocationCallbacks,
) {
    if device.is_null() || shaderModule.is_null() {
        return;
    }
    
    if let Some(icd) = icd_loader::icd_for_device(device) {
        if let Some(f) = icd.destroy_shader_module { f(device, shaderModule, pAllocator); }
        return;
    }
    if let Some(icd) = super::forward::get_icd_if_enabled() {
        if let Some(destroy_shader_module) = icd.destroy_shader_module { destroy_shader_module(device, shaderModule, pAllocator); }
    }
}

/// Create compute pipelines
// SAFETY: This function is called from C code. Caller must ensure:
// 1. device is a valid VkDevice
// 2. pipelineCache is either VK_NULL_HANDLE or a valid VkPipelineCache
// 3. createInfoCount > 0 and matches the array sizes
// 4. pCreateInfos points to an array of createInfoCount valid pipeline create info structures
// 5. pAllocator is either null or points to valid allocation callbacks
// 6. pPipelines points to an array with space for createInfoCount pipeline handles
// 7. All shader modules, layouts, and descriptor set layouts referenced are valid
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
    
    if let Some(icd) = icd_loader::icd_for_device(device) {
        if let Some(f) = icd.create_compute_pipelines { return f(device, pipelineCache, createInfoCount, pCreateInfos, pAllocator, pPipelines); }
    }
    if let Some(icd) = super::forward::get_icd_if_enabled() {
        if let Some(create_compute_pipelines) = icd.create_compute_pipelines { return create_compute_pipelines(device, pipelineCache, createInfoCount, pCreateInfos, pAllocator, pPipelines); }
    }
    VkResult::ErrorInitializationFailed
}

/// Destroy pipeline
// SAFETY: This function is called from C code. Caller must ensure:
// 1. device is a valid VkDevice
// 2. pipeline is a valid VkPipeline, or VK_NULL_HANDLE
// 3. pAllocator matches the allocator used in vkCreateComputePipelines
// 4. The pipeline is not currently bound to any command buffers
// 5. No command buffers using this pipeline are executing on the GPU
#[no_mangle]
pub unsafe extern "C" fn vkDestroyPipeline(
    device: VkDevice,
    pipeline: VkPipeline,
    pAllocator: *const VkAllocationCallbacks,
) {
    if device.is_null() || pipeline.is_null() {
        return;
    }
    
    if let Some(icd) = icd_loader::icd_for_device(device) {
        if let Some(f) = icd.destroy_pipeline { f(device, pipeline, pAllocator); }
        return;
    }
    if let Some(icd) = super::forward::get_icd_if_enabled() {
        if let Some(destroy_pipeline) = icd.destroy_pipeline { destroy_pipeline(device, pipeline, pAllocator); }
    }
}

/// Create pipeline layout
// SAFETY: This function is called from C code. Caller must ensure:
// 1. device is a valid VkDevice
// 2. pCreateInfo points to a valid VkPipelineLayoutCreateInfo structure
// 3. pAllocator is either null or points to valid allocation callbacks
// 4. pPipelineLayout points to valid memory for writing the layout handle
// 5. All descriptor set layouts referenced in pCreateInfo are valid
// 6. Push constant ranges do not overlap and are within device limits
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
    
    if let Some(icd) = icd_loader::icd_for_device(device) {
        if let Some(f) = icd.create_pipeline_layout { return f(device, pCreateInfo, pAllocator, pPipelineLayout); }
    }
    if let Some(icd) = super::forward::get_icd_if_enabled() {
        if let Some(create_pipeline_layout) = icd.create_pipeline_layout { return create_pipeline_layout(device, pCreateInfo, pAllocator, pPipelineLayout); }
    }
    VkResult::ErrorInitializationFailed
}

/// Destroy pipeline layout
// SAFETY: This function is called from C code. Caller must ensure:
// 1. device is a valid VkDevice
// 2. pipelineLayout is a valid VkPipelineLayout, or VK_NULL_HANDLE
// 3. pAllocator matches the allocator used in vkCreatePipelineLayout
// 4. No pipelines are currently using this layout
// 5. No command buffers reference this layout in their current recordings
#[no_mangle]
pub unsafe extern "C" fn vkDestroyPipelineLayout(
    device: VkDevice,
    pipelineLayout: VkPipelineLayout,
    pAllocator: *const VkAllocationCallbacks,
) {
    if device.is_null() || pipelineLayout.is_null() {
        return;
    }
    
    if let Some(icd) = icd_loader::icd_for_device(device) {
        if let Some(f) = icd.destroy_pipeline_layout { f(device, pipelineLayout, pAllocator); }
        return;
    }
    if let Some(icd) = super::forward::get_icd_if_enabled() {
        if let Some(destroy_pipeline_layout) = icd.destroy_pipeline_layout { destroy_pipeline_layout(device, pipelineLayout, pAllocator); }
    }
}

/// Create command pool
// SAFETY: This function is called from C code. Caller must ensure:
// 1. device is a valid VkDevice
// 2. pCreateInfo points to a valid VkCommandPoolCreateInfo structure
// 3. pAllocator is either null or points to valid allocation callbacks
// 4. pCommandPool points to valid memory for writing the pool handle
// 5. The queue family index in pCreateInfo is valid for this device
// 6. Pool creation flags are appropriate for intended usage
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
    // Route via owning ICD if known
    if let Some(icd) = icd_loader::icd_for_device(device) {
        if let Some(f) = icd.create_command_pool {
            let res = f(device, pCreateInfo, pAllocator, pCommandPool);
            if res == VkResult::Success {
                icd_loader::register_command_pool_icd(*pCommandPool, &icd);
            }
            return res;
        }
    }
    // Fallback
    if let Some(icd) = super::forward::get_icd_if_enabled() {
        if let Some(create_command_pool) = icd.create_command_pool {
            return create_command_pool(device, pCreateInfo, pAllocator, pCommandPool);
        }
    }
    VkResult::ErrorInitializationFailed
}

/// Destroy command pool
// SAFETY: This function is called from C code. Caller must ensure:
// 1. device is a valid VkDevice
// 2. commandPool is a valid VkCommandPool, or VK_NULL_HANDLE
// 3. pAllocator matches the allocator used in vkCreateCommandPool
// 4. All command buffers allocated from this pool have finished execution
// 5. No command buffers from this pool are currently being recorded
#[no_mangle]
pub unsafe extern "C" fn vkDestroyCommandPool(
    device: VkDevice,
    commandPool: VkCommandPool,
    pAllocator: *const VkAllocationCallbacks,
) {
    if device.is_null() || commandPool.is_null() {
        return;
    }
    if let Some(icd) = icd_loader::icd_for_command_pool(commandPool) {
        if let Some(f) = icd.destroy_command_pool { f(device, commandPool, pAllocator); }
        icd_loader::unregister_command_pool(commandPool);
        return;
    }
    if let Some(icd) = super::forward::get_icd_if_enabled() {
        if let Some(destroy_command_pool) = icd.destroy_command_pool {
            destroy_command_pool(device, commandPool, pAllocator);
        }
    }
}

/// Allocate command buffers
// SAFETY: This function is called from C code. Caller must ensure:
// 1. device is a valid VkDevice
// 2. pAllocateInfo points to a valid VkCommandBufferAllocateInfo structure
// 3. pCommandBuffers points to an array with space for commandBufferCount handles
// 4. The command pool in pAllocateInfo is valid and supports the requested level
// 5. The command pool has sufficient space for the requested buffers
#[no_mangle]
pub unsafe extern "C" fn vkAllocateCommandBuffers(
    device: VkDevice,
    pAllocateInfo: *const VkCommandBufferAllocateInfo,
    pCommandBuffers: *mut VkCommandBuffer,
) -> VkResult {
    if device.is_null() || pAllocateInfo.is_null() || pCommandBuffers.is_null() {
        return VkResult::ErrorInitializationFailed;
    }
    // Prefer routing by command pool owner
    let pool = (*pAllocateInfo).commandPool;
    if let Some(icd) = icd_loader::icd_for_command_pool(pool) {
        if let Some(f) = icd.allocate_command_buffers {
            let res = f(device, pAllocateInfo, pCommandBuffers);
            if res == VkResult::Success {
                let count = (*pAllocateInfo).commandBufferCount as isize;
                for i in 0..count {
                    let cb = *pCommandBuffers.offset(i);
                    icd_loader::register_command_buffer_icd(cb, &icd);
                }
            }
            return res;
        }
    }
    if let Some(icd) = super::forward::get_icd_if_enabled() {
        if let Some(allocate_command_buffers) = icd.allocate_command_buffers {
            return allocate_command_buffers(device, pAllocateInfo, pCommandBuffers);
        }
    }
    VkResult::ErrorInitializationFailed
}

/// Free command buffers
// SAFETY: This function is called from C code. Caller must ensure:
// 1. device is a valid VkDevice
// 2. commandPool is a valid VkCommandPool
// 3. commandBufferCount > 0 and matches the array size
// 4. pCommandBuffers points to an array of commandBufferCount valid command buffers
// 5. All command buffers were allocated from the specified pool
// 6. None of the command buffers are currently executing on the GPU
// 7. Command buffers are not in the recording state
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
    if let Some(icd) = icd_loader::icd_for_command_pool(commandPool) {
        if let Some(f) = icd.free_command_buffers { f(device, commandPool, commandBufferCount, pCommandBuffers); }
        for i in 0..(commandBufferCount as isize) {
            let cb = *pCommandBuffers.offset(i);
            icd_loader::unregister_command_buffer(cb);
        }
        return;
    }
    if let Some(icd) = super::forward::get_icd_if_enabled() {
        if let Some(free_command_buffers) = icd.free_command_buffers {
            free_command_buffers(device, commandPool, commandBufferCount, pCommandBuffers);
        }
    }
}

/// Begin command buffer recording
// SAFETY: This function is called from C code. Caller must ensure:
// 1. commandBuffer is a valid VkCommandBuffer in the initial state
// 2. pBeginInfo points to a valid VkCommandBufferBeginInfo structure
// 3. The command buffer is not currently being recorded
// 4. The command buffer is not currently executing on the GPU
// 5. Usage flags in pBeginInfo match the intended recording pattern
#[no_mangle]
pub unsafe extern "C" fn vkBeginCommandBuffer(
    commandBuffer: VkCommandBuffer,
    pBeginInfo: *const VkCommandBufferBeginInfo,
) -> VkResult {
    if commandBuffer.is_null() || pBeginInfo.is_null() {
        return VkResult::ErrorInitializationFailed;
    }
    if let Some(icd) = icd_loader::icd_for_command_buffer(commandBuffer) {
        if let Some(f) = icd.begin_command_buffer { return f(commandBuffer, pBeginInfo); }
    }
    if let Some(icd) = super::forward::get_icd_if_enabled() {
        if let Some(begin_command_buffer) = icd.begin_command_buffer {
            return begin_command_buffer(commandBuffer, pBeginInfo);
        }
    }
    VkResult::ErrorInitializationFailed
}

/// End command buffer recording
// SAFETY: This function is called from C code. Caller must ensure:
// 1. commandBuffer is a valid VkCommandBuffer in the recording state
// 2. All commands recorded since vkBeginCommandBuffer are valid
// 3. The command buffer was successfully put into recording state
// 4. All nested command buffer recordings have been properly ended
#[no_mangle]
pub unsafe extern "C" fn vkEndCommandBuffer(
    commandBuffer: VkCommandBuffer,
) -> VkResult {
    if commandBuffer.is_null() {
        return VkResult::ErrorInitializationFailed;
    }
    if let Some(icd) = icd_loader::icd_for_command_buffer(commandBuffer) {
        if let Some(f) = icd.end_command_buffer { return f(commandBuffer); }
    }
    if let Some(icd) = super::forward::get_icd_if_enabled() {
        if let Some(end_command_buffer) = icd.end_command_buffer {
            return end_command_buffer(commandBuffer);
        }
    }
    VkResult::ErrorInitializationFailed
}

/// Bind pipeline
// SAFETY: This function is called from C code. Caller must ensure:
// 1. commandBuffer is a valid VkCommandBuffer in the recording state
// 2. pipelineBindPoint is VK_PIPELINE_BIND_POINT_COMPUTE for compute pipelines
// 3. pipeline is a valid VkPipeline compatible with the bind point
// 4. The pipeline's layout is compatible with subsequently bound descriptor sets
// 5. The command buffer supports the queue family that created the pipeline
#[no_mangle]
pub unsafe extern "C" fn vkCmdBindPipeline(
    commandBuffer: VkCommandBuffer,
    pipelineBindPoint: VkPipelineBindPoint,
    pipeline: VkPipeline,
) {
    if commandBuffer.is_null() || pipeline.is_null() {
        return;
    }
    if let Some(icd) = icd_loader::icd_for_command_buffer(commandBuffer) {
        if let Some(f) = icd.cmd_bind_pipeline { f(commandBuffer, pipelineBindPoint, pipeline); }
        return;
    }
    if let Some(icd) = super::forward::get_icd_if_enabled() {
        if let Some(cmd_bind_pipeline) = icd.cmd_bind_pipeline {
            cmd_bind_pipeline(commandBuffer, pipelineBindPoint, pipeline);
        }
    }
}

/// Bind descriptor sets
// SAFETY: This function is called from C code. Caller must ensure:
// 1. commandBuffer is a valid VkCommandBuffer in the recording state
// 2. pipelineBindPoint is VK_PIPELINE_BIND_POINT_COMPUTE
// 3. layout is a valid VkPipelineLayout
// 4. descriptorSetCount > 0 and pDescriptorSets points to that many valid descriptor sets
// 5. All descriptor sets are compatible with the pipeline layout
// 6. Dynamic offsets array matches the dynamic descriptors in the sets
// 7. firstSet + descriptorSetCount <= max sets supported by layout
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
    if let Some(icd) = icd_loader::icd_for_command_buffer(commandBuffer) {
        if let Some(f) = icd.cmd_bind_descriptor_sets { f(commandBuffer, pipelineBindPoint, layout, firstSet, descriptorSetCount, pDescriptorSets, dynamicOffsetCount, pDynamicOffsets); }
        return;
    }
    if let Some(icd) = super::forward::get_icd_if_enabled() {
        if let Some(cmd_bind_descriptor_sets) = icd.cmd_bind_descriptor_sets {
            cmd_bind_descriptor_sets(commandBuffer, pipelineBindPoint, layout, firstSet, 
                                   descriptorSetCount, pDescriptorSets, dynamicOffsetCount, pDynamicOffsets);
        }
    }
}

/// Push constants
// SAFETY: This function is called from C code. Caller must ensure:
// 1. commandBuffer is a valid VkCommandBuffer in the recording state
// 2. layout is a valid VkPipelineLayout with push constant ranges
// 3. stageFlags specifies valid pipeline stages (VK_SHADER_STAGE_COMPUTE_BIT)
// 4. offset and size are within the push constant range defined in the layout
// 5. pValues points to at least size bytes of valid memory
// 6. The push constant data is properly aligned for the target architecture
#[no_mangle]
pub unsafe extern "C" fn vkCmdPushConstants(
    commandBuffer: VkCommandBuffer,
    layout: VkPipelineLayout,
    stageFlags: VkShaderStageFlags,
    offset: u32,
    size: u32,
    pValues: *const libc::c_void,
) {
    if commandBuffer.is_null() || layout.is_null() || pValues.is_null() || size == 0 {
        return;
    }
    if let Some(icd) = icd_loader::icd_for_command_buffer(commandBuffer) {
        if let Some(f) = icd.cmd_push_constants { f(commandBuffer, layout, stageFlags, offset, size, pValues); }
        return;
    }
    if let Some(icd) = super::forward::get_icd_if_enabled() {
        if let Some(cmd_push_constants) = icd.cmd_push_constants {
            cmd_push_constants(commandBuffer, layout, stageFlags, offset, size, pValues);
        }
    }
}

/// Dispatch compute work
// SAFETY: This function is called from C code. Caller must ensure:
// 1. commandBuffer is a valid VkCommandBuffer in the recording state
// 2. A compute pipeline is currently bound to the command buffer
// 3. groupCountX, Y, Z are within device limits and > 0
// 4. All descriptor sets required by the pipeline are bound
// 5. All resources referenced by descriptors are valid and accessible
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
    if let Some(icd) = icd_loader::icd_for_command_buffer(commandBuffer) {
        if let Some(f) = icd.cmd_dispatch { f(commandBuffer, groupCountX, groupCountY, groupCountZ); }
        return;
    }
    if let Some(icd) = super::forward::get_icd_if_enabled() {
        if let Some(cmd_dispatch) = icd.cmd_dispatch {
            cmd_dispatch(commandBuffer, groupCountX, groupCountY, groupCountZ);
        }
    }
}

/// Dispatch compute work with indirect buffer
// SAFETY: This function is called from C code. Caller must ensure:
// 1. commandBuffer is a valid VkCommandBuffer in the recording state
// 2. A compute pipeline is currently bound to the command buffer
// 3. buffer is a valid VkBuffer containing dispatch parameters
// 4. offset is within the buffer bounds and properly aligned
// 5. The buffer contains valid VkDispatchIndirectCommand structure at offset
// 6. All descriptor sets required by the pipeline are bound
#[no_mangle]
pub unsafe extern "C" fn vkCmdDispatchIndirect(
    commandBuffer: VkCommandBuffer,
    buffer: VkBuffer,
    offset: VkDeviceSize,
) {
    if commandBuffer.is_null() || buffer.is_null() {
        return;
    }
    if let Some(icd) = icd_loader::icd_for_command_buffer(commandBuffer) {
        if let Some(f) = icd.cmd_dispatch_indirect { f(commandBuffer, buffer, offset); }
        return;
    }
    if let Some(icd) = super::forward::get_icd_if_enabled() {
        if let Some(cmd_dispatch_indirect) = icd.cmd_dispatch_indirect {
            cmd_dispatch_indirect(commandBuffer, buffer, offset);
        }
    }
}

/// Pipeline barrier
// SAFETY: This function is called from C code. Caller must ensure:
// 1. commandBuffer is a valid VkCommandBuffer in the recording state
// 2. srcStageMask and dstStageMask specify valid pipeline stages
// 3. dependencyFlags is a valid combination of dependency flags
// 4. Memory barrier arrays match their respective counts
// 5. All buffer memory barriers reference valid buffers and ranges
// 6. Pipeline stages are appropriate for compute operations
// 7. Memory barriers provide necessary synchronization guarantees
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
    if let Some(icd) = icd_loader::icd_for_command_buffer(commandBuffer) {
        if let Some(f) = icd.cmd_pipeline_barrier { f(commandBuffer, srcStageMask, dstStageMask, dependencyFlags,
                               memoryBarrierCount, pMemoryBarriers, bufferMemoryBarrierCount,
                               pBufferMemoryBarriers, imageMemoryBarrierCount, pImageMemoryBarriers); }
        return;
    }
    if let Some(icd) = super::forward::get_icd_if_enabled() {
        if let Some(cmd_pipeline_barrier) = icd.cmd_pipeline_barrier {
            cmd_pipeline_barrier(commandBuffer, srcStageMask, dstStageMask, dependencyFlags,
                               memoryBarrierCount, pMemoryBarriers, bufferMemoryBarrierCount,
                               pBufferMemoryBarriers, imageMemoryBarrierCount, pImageMemoryBarriers);
        }
    }
}

/// Copy buffer
// SAFETY: This function is called from C code. Caller must ensure:
// 1. commandBuffer is a valid VkCommandBuffer in the recording state
// 2. srcBuffer and dstBuffer are valid VkBuffer objects
// 3. regionCount > 0 and pRegions points to that many VkBufferCopy structures
// 4. All copy regions are within the bounds of their respective buffers
// 5. Source and destination ranges do not overlap (unless same buffer with identical ranges)
// 6. Buffers support the required usage flags for transfer operations
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
    if let Some(icd) = icd_loader::icd_for_command_buffer(commandBuffer) {
        if let Some(f) = icd.cmd_copy_buffer { f(commandBuffer, srcBuffer, dstBuffer, regionCount, pRegions); }
        return;
    }
    if let Some(icd) = super::forward::get_icd_if_enabled() {
        if let Some(cmd_copy_buffer) = icd.cmd_copy_buffer {
            cmd_copy_buffer(commandBuffer, srcBuffer, dstBuffer, regionCount, pRegions);
        }
    }
}

/// Set event
// SAFETY: This function is called from C code. Caller must ensure:
// 1. commandBuffer is a valid VkCommandBuffer in the recording state
// 2. event is a valid VkEvent object
// 3. stageMask specifies valid pipeline stages for compute operations
// 4. The event will be signaled when the specified pipeline stages complete
// 5. Subsequent vkCmdWaitEvents calls can safely wait on this event
#[no_mangle]
pub unsafe extern "C" fn vkCmdSetEvent(
    commandBuffer: VkCommandBuffer,
    event: VkEvent,
    stageMask: VkPipelineStageFlags,
) {
    if commandBuffer.is_null() || event.is_null() {
        return;
    }
    if let Some(icd) = icd_loader::icd_for_command_buffer(commandBuffer) {
        if let Some(f) = icd.cmd_set_event { f(commandBuffer, event, stageMask); }
        return;
    }
    if let Some(icd) = super::forward::get_icd_if_enabled() {
        if let Some(cmd_set_event) = icd.cmd_set_event {
            cmd_set_event(commandBuffer, event, stageMask);
        }
    }
}

/// Reset event
// SAFETY: This function is called from C code. Caller must ensure:
// 1. commandBuffer is a valid VkCommandBuffer in the recording state
// 2. event is a valid VkEvent object
// 3. stageMask specifies valid pipeline stages for compute operations
// 4. The event will be reset when the specified pipeline stages complete
// 5. No other command buffers should be waiting on this event when it's reset
#[no_mangle]
pub unsafe extern "C" fn vkCmdResetEvent(
    commandBuffer: VkCommandBuffer,
    event: VkEvent,
    stageMask: VkPipelineStageFlags,
) {
    if commandBuffer.is_null() || event.is_null() {
        return;
    }
    if let Some(icd) = icd_loader::icd_for_command_buffer(commandBuffer) {
        if let Some(f) = icd.cmd_reset_event { f(commandBuffer, event, stageMask); }
        return;
    }
    if let Some(icd) = super::forward::get_icd_if_enabled() {
        if let Some(cmd_reset_event) = icd.cmd_reset_event {
            cmd_reset_event(commandBuffer, event, stageMask);
        }
    }
}

/// Wait for events
// SAFETY: This function is called from C code. Caller must ensure:
// 1. commandBuffer is a valid VkCommandBuffer in the recording state
// 2. eventCount > 0 and pEvents points to that many valid VkEvent objects
// 3. srcStageMask and dstStageMask specify valid pipeline stages
// 4. All events in pEvents have been signaled by previous commands
// 5. Memory barrier arrays match their respective counts and are valid
// 6. All buffer memory barriers reference valid buffers and ranges
// 7. Pipeline stages and barriers provide necessary synchronization
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
    if let Some(icd) = icd_loader::icd_for_command_buffer(commandBuffer) {
        if let Some(f) = icd.cmd_wait_events { f(commandBuffer, eventCount, pEvents, srcStageMask, dstStageMask,
                          memoryBarrierCount, pMemoryBarriers, bufferMemoryBarrierCount,
                          pBufferMemoryBarriers, imageMemoryBarrierCount, pImageMemoryBarriers); }
        return;
    }
    if let Some(icd) = super::forward::get_icd_if_enabled() {
        if let Some(cmd_wait_events) = icd.cmd_wait_events {
            cmd_wait_events(commandBuffer, eventCount, pEvents, srcStageMask, dstStageMask,
                          memoryBarrierCount, pMemoryBarriers, bufferMemoryBarrierCount,
                          pBufferMemoryBarriers, imageMemoryBarrierCount, pImageMemoryBarriers);
        }
    }
}
