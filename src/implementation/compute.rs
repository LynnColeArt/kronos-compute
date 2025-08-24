//! Compute pipeline and command buffer implementation

use std::sync::{Arc, Mutex};
use std::collections::HashMap;
use crate::sys::*;
use crate::core::*;
use crate::ffi::*;

lazy_static::lazy_static! {
    // Global storage for compute resources
    static ref SHADER_MODULES: Mutex<HashMap<u64, ShaderModule>> = Mutex::new(HashMap::new());
    static ref PIPELINES: Mutex<HashMap<u64, ComputePipeline>> = Mutex::new(HashMap::new());
    static ref PIPELINE_LAYOUTS: Mutex<HashMap<u64, PipelineLayout>> = Mutex::new(HashMap::new());
    static ref DESCRIPTOR_SET_LAYOUTS: Mutex<HashMap<u64, DescriptorSetLayout>> = Mutex::new(HashMap::new());
    static ref COMMAND_POOLS: Mutex<HashMap<u64, CommandPool>> = Mutex::new(HashMap::new());
    pub(crate) static ref COMMAND_BUFFERS: Mutex<HashMap<u64, CommandBuffer>> = Mutex::new(HashMap::new());
}

struct ShaderModule {
    handle: VkShaderModule,
    code: Vec<u32>,
}

struct ComputePipeline {
    handle: VkPipeline,
    layout: VkPipelineLayout,
    shader: VkShaderModule,
}

struct PipelineLayout {
    handle: VkPipelineLayout,
    set_layouts: Vec<VkDescriptorSetLayout>,
    push_constant_ranges: Vec<VkPushConstantRange>,
}

struct DescriptorSetLayout {
    handle: VkDescriptorSetLayout,
    bindings: Vec<VkDescriptorSetLayoutBinding>,
}

struct CommandPool {
    handle: VkCommandPool,
    queue_family: u32,
    buffers: Vec<VkCommandBuffer>,
}

pub(crate) struct CommandBuffer {
    pub handle: VkCommandBuffer,
    pub pool: VkCommandPool,
    pub state: CommandBufferState,
    pub commands: Vec<Command>,
}

#[derive(Clone)]
enum CommandBufferState {
    Initial,
    Recording,
    Executable,
}

#[derive(Clone)]
pub enum Command {
    BindPipeline { pipeline: VkPipeline },
    BindDescriptorSets {
        layout: VkPipelineLayout,
        first_set: u32,
        sets: Vec<VkDescriptorSet>,
        dynamic_offsets: Vec<u32>,
    },
    Dispatch { x: u32, y: u32, z: u32 },
    PipelineBarrier {
        src_stage: VkPipelineStageFlags,
        dst_stage: VkPipelineStageFlags,
        memory_barriers: Vec<VkMemoryBarrier>,
        buffer_barriers: Vec<VkBufferMemoryBarrier>,
    },
    SetEvent {
        event: VkEvent,
        stage_mask: VkPipelineStageFlags,
    },
    ResetEvent {
        event: VkEvent,
        stage_mask: VkPipelineStageFlags,
    },
    WaitEvents {
        events: Vec<VkEvent>,
        src_stage: VkPipelineStageFlags,
        dst_stage: VkPipelineStageFlags,
        memory_barriers: Vec<VkMemoryBarrier>,
        buffer_barriers: Vec<VkBufferMemoryBarrier>,
    },
}

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
    
    let create_info = &*pCreateInfo;
    
    if create_info.sType != VkStructureType::ShaderModuleCreateInfo {
        return VkResult::ErrorInitializationFailed;
    }
    
    // Copy shader code
    let code_words = create_info.codeSize / 4;
    let code = std::slice::from_raw_parts(create_info.pCode, code_words).to_vec();
    
    // Generate handle
    let handle = VkShaderModule::from_raw(SHADER_MODULES.lock().unwrap().len() as u64 + 1);
    
    let module = ShaderModule {
        handle,
        code,
    };
    
    SHADER_MODULES.lock().unwrap().insert(handle.as_raw(), module);
    
    *pShaderModule = handle;
    VkResult::Success
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
    if device.is_null() || pCreateInfos.is_null() || pPipelines.is_null() {
        return VkResult::ErrorInitializationFailed;
    }
    
    // Forward to real driver if enabled
    if let Some(icd) = super::forward::get_icd_if_enabled() {
        if let Some(create_compute_pipelines) = icd.create_compute_pipelines {
            return create_compute_pipelines(device, pipelineCache, createInfoCount, pCreateInfos, pAllocator, pPipelines);
        }
    }
    
    for i in 0..createInfoCount {
        let create_info = &*pCreateInfos.add(i as usize);
        
        if create_info.sType != VkStructureType::ComputePipelineCreateInfo {
            return VkResult::ErrorInitializationFailed;
        }
        
        // Generate handle
        let handle = VkPipeline::from_raw(PIPELINES.lock().unwrap().len() as u64 + 1);
        
        let pipeline = ComputePipeline {
            handle,
            layout: create_info.layout,
            shader: create_info.stage.module,
        };
        
        PIPELINES.lock().unwrap().insert(handle.as_raw(), pipeline);
        
        *pPipelines.add(i as usize) = handle;
    }
    
    VkResult::Success
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
    
    let create_info = &*pCreateInfo;
    
    if create_info.sType != VkStructureType::PipelineLayoutCreateInfo {
        return VkResult::ErrorInitializationFailed;
    }
    
    // Copy set layouts
    let set_layouts = if create_info.setLayoutCount > 0 {
        std::slice::from_raw_parts(create_info.pSetLayouts, create_info.setLayoutCount as usize).to_vec()
    } else {
        Vec::new()
    };
    
    // Copy push constant ranges
    let push_constant_ranges = if create_info.pushConstantRangeCount > 0 {
        std::slice::from_raw_parts(create_info.pPushConstantRanges, create_info.pushConstantRangeCount as usize).to_vec()
    } else {
        Vec::new()
    };
    
    let handle = VkPipelineLayout::from_raw(PIPELINE_LAYOUTS.lock().unwrap().len() as u64 + 1);
    
    let layout = PipelineLayout {
        handle,
        set_layouts,
        push_constant_ranges,
    };
    
    PIPELINE_LAYOUTS.lock().unwrap().insert(handle.as_raw(), layout);
    
    *pPipelineLayout = handle;
    VkResult::Success
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
    
    let create_info = &*pCreateInfo;
    
    if create_info.sType != VkStructureType::CommandPoolCreateInfo {
        return VkResult::ErrorInitializationFailed;
    }
    
    let handle = VkCommandPool::from_raw(COMMAND_POOLS.lock().unwrap().len() as u64 + 1);
    
    let pool = CommandPool {
        handle,
        queue_family: create_info.queueFamilyIndex,
        buffers: Vec::new(),
    };
    
    COMMAND_POOLS.lock().unwrap().insert(handle.as_raw(), pool);
    
    *pCommandPool = handle;
    VkResult::Success
}

/// Destroy command pool
#[no_mangle]
pub unsafe extern "C" fn vkDestroyCommandPool(
    device: VkDevice,
    commandPool: VkCommandPool,
    _pAllocator: *const VkAllocationCallbacks,
) {
    if device.is_null() || commandPool.is_null() {
        return;
    }
    
    // Remove all command buffers associated with this pool
    let pools = COMMAND_POOLS.lock().unwrap();
    if let Some(pool) = pools.get(&commandPool.as_raw()) {
        let mut buffers = COMMAND_BUFFERS.lock().unwrap();
        for &buffer in &pool.buffers {
            buffers.remove(&buffer.as_raw());
        }
    }
    drop(pools);
    
    // Remove the pool itself
    COMMAND_POOLS.lock().unwrap().remove(&commandPool.as_raw());
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
    
    let alloc_info = &*pAllocateInfo;
    
    if alloc_info.sType != VkStructureType::CommandBufferAllocateInfo {
        return VkResult::ErrorInitializationFailed;
    }
    
    let mut pools = COMMAND_POOLS.lock().unwrap();
    let pool = match pools.get_mut(&alloc_info.commandPool.as_raw()) {
        Some(p) => p,
        None => return VkResult::ErrorInitializationFailed,
    };
    
    for i in 0..alloc_info.commandBufferCount {
        let handle = VkCommandBuffer::from_raw(COMMAND_BUFFERS.lock().unwrap().len() as u64 + 1);
        
        let buffer = CommandBuffer {
            handle,
            pool: alloc_info.commandPool,
            state: CommandBufferState::Initial,
            commands: Vec::new(),
        };
        
        pool.buffers.push(handle);
        COMMAND_BUFFERS.lock().unwrap().insert(handle.as_raw(), buffer);
        
        *pCommandBuffers.add(i as usize) = handle;
    }
    
    VkResult::Success
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
    
    let begin_info = &*pBeginInfo;
    
    if begin_info.sType != VkStructureType::CommandBufferBeginInfo {
        return VkResult::ErrorInitializationFailed;
    }
    
    let mut buffers = COMMAND_BUFFERS.lock().unwrap();
    let buffer = match buffers.get_mut(&commandBuffer.as_raw()) {
        Some(b) => b,
        None => return VkResult::ErrorInitializationFailed,
    };
    
    // Reset command buffer
    buffer.commands.clear();
    buffer.state = CommandBufferState::Recording;
    
    VkResult::Success
}

/// End command buffer recording
#[no_mangle]
pub unsafe extern "C" fn vkEndCommandBuffer(
    commandBuffer: VkCommandBuffer,
) -> VkResult {
    if commandBuffer.is_null() {
        return VkResult::ErrorInitializationFailed;
    }
    
    let mut buffers = COMMAND_BUFFERS.lock().unwrap();
    let buffer = match buffers.get_mut(&commandBuffer.as_raw()) {
        Some(b) => b,
        None => return VkResult::ErrorInitializationFailed,
    };
    
    buffer.state = CommandBufferState::Executable;
    
    VkResult::Success
}

/// Bind compute pipeline
#[no_mangle]
pub unsafe extern "C" fn vkCmdBindPipeline(
    commandBuffer: VkCommandBuffer,
    pipelineBindPoint: VkPipelineBindPoint,
    pipeline: VkPipeline,
) {
    if commandBuffer.is_null() || pipeline.is_null() {
        return;
    }
    
    if pipelineBindPoint != VkPipelineBindPoint::Compute {
        return; // We only support compute
    }
    
    let mut buffers = COMMAND_BUFFERS.lock().unwrap();
    if let Some(buffer) = buffers.get_mut(&commandBuffer.as_raw()) {
        buffer.commands.push(Command::BindPipeline { pipeline });
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
    
    // Forward to real driver if enabled
    if let Some(icd) = super::forward::get_icd_if_enabled() {
        if let Some(cmd_dispatch) = icd.cmd_dispatch {
            cmd_dispatch(commandBuffer, groupCountX, groupCountY, groupCountZ);
            return;
        }
    }
    
    let mut buffers = COMMAND_BUFFERS.lock().unwrap();
    if let Some(buffer) = buffers.get_mut(&commandBuffer.as_raw()) {
        buffer.commands.push(Command::Dispatch {
            x: groupCountX,
            y: groupCountY,
            z: groupCountZ,
        });
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
    
    // Copy barriers
    let memory_barriers = if memoryBarrierCount > 0 {
        std::slice::from_raw_parts(pMemoryBarriers, memoryBarrierCount as usize).to_vec()
    } else {
        Vec::new()
    };
    
    let buffer_barriers = if bufferMemoryBarrierCount > 0 {
        std::slice::from_raw_parts(pBufferMemoryBarriers, bufferMemoryBarrierCount as usize).to_vec()
    } else {
        Vec::new()
    };
    
    let mut buffers = COMMAND_BUFFERS.lock().unwrap();
    if let Some(buffer) = buffers.get_mut(&commandBuffer.as_raw()) {
        buffer.commands.push(Command::PipelineBarrier {
            src_stage: srcStageMask,
            dst_stage: dstStageMask,
            memory_barriers,
            buffer_barriers,
        });
    }
}