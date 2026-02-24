//! Fluent command building and dispatch

use super::*;
use crate::*; // Import all functions from the crate root
#[cfg(feature = "implementation")]
use crate::implementation::persistent_descriptors::get_persistent_descriptor_set;
use std::ptr;

/// Fluent builder for compute dispatch commands
/// 
/// This builder provides a safe, ergonomic API for recording
/// and executing compute commands. All Kronos optimizations
/// are applied automatically.
pub struct CommandBuilder {
    context: ComputeContext,
    pipeline: Pipeline,
    command_buffer: VkCommandBuffer,
    descriptor_set: Option<VkDescriptorSet>,
    bindings: Vec<(u32, Buffer)>,
    push_constants: Vec<u8>,
    workgroups: (u32, u32, u32),
}

impl ComputeContext {
    /// Start building a compute dispatch
    pub fn dispatch(&self, pipeline: &Pipeline) -> CommandBuilder {
        CommandBuilder {
            context: self.clone(),
            pipeline: Pipeline {
                context: pipeline.context.clone(),
                pipeline: pipeline.pipeline,
                layout: pipeline.layout,
                descriptor_set_layout: pipeline.descriptor_set_layout,
            },
            command_buffer: VkCommandBuffer::NULL,
            descriptor_set: None,
            bindings: Vec::new(),
            push_constants: Vec::new(),
            workgroups: (1, 1, 1),
        }
    }
}

impl CommandBuilder {
    /// Bind a buffer to a binding point
    pub fn bind_buffer(mut self, binding: u32, buffer: &Buffer) -> Self {
        self.bindings.push((binding, Buffer {
            context: buffer.context.clone(),
            buffer: buffer.buffer,
            memory: buffer.memory,
            size: buffer.size,
            usage: buffer.usage,
            _marker: std::marker::PhantomData,
        }));
        self
    }
    
    /// Set push constants
    pub fn push_constants<T: Copy>(mut self, data: &T) -> Self {
        let bytes = unsafe {
            std::slice::from_raw_parts(
                data as *const T as *const u8,
                std::mem::size_of::<T>(),
            )
        };
        self.push_constants = bytes.to_vec();
        self
    }
    
    /// Set the number of workgroups
    pub fn workgroups(mut self, x: u32, y: u32, z: u32) -> Self {
        self.workgroups = (x, y, z);
        self
    }
    
    /// Execute the dispatch
    pub fn execute(mut self) -> Result<()> {
        unsafe {
            let mut allocated_command_buffer = VkCommandBuffer::NULL;
            let mut allocated_descriptor_set = VkDescriptorSet::NULL;
            let has_bindings = !self.bindings.is_empty();
            #[cfg(feature = "implementation")]
            let use_persistent_descriptors = has_bindings && self.bindings
                .iter()
                .enumerate()
                .all(|(index, (binding, _))| *binding == index as u32);
            #[cfg(not(feature = "implementation"))]
            let use_persistent_descriptors = false;

            let execute_result = self.context.with_inner(|inner| {
                if inner.device == VkDevice::NULL {
                    return Err(KronosError::CommandExecutionFailed(
                        "Compute context has no valid Vulkan device".into(),
                    ));
                }
                if inner.command_pool == VkCommandPool::NULL {
                    return Err(KronosError::CommandExecutionFailed(
                        "Compute context has no valid command pool".into(),
                    ));
                }
                if inner.queue == VkQueue::NULL {
                    return Err(KronosError::CommandExecutionFailed(
                        "Compute context has no valid compute queue".into(),
                    ));
                }
                if self.pipeline.pipeline == VkPipeline::NULL {
                    return Err(KronosError::CommandExecutionFailed(
                        "CommandBuilder has no valid compute pipeline".into(),
                    ));
                }
                if self.pipeline.layout == VkPipelineLayout::NULL {
                    return Err(KronosError::CommandExecutionFailed(
                        "CommandBuilder has no valid pipeline layout".into(),
                    ));
                }
                if has_bindings && self.pipeline.descriptor_set_layout == VkDescriptorSetLayout::NULL {
                    return Err(KronosError::CommandExecutionFailed(
                        "Buffer bindings require a valid descriptor set layout".into(),
                    ));
                }
                for (binding_index, (_, buffer)) in self.bindings.iter().enumerate() {
                    if buffer.buffer == VkBuffer::NULL {
                        return Err(KronosError::CommandExecutionFailed(format!(
                            "Binding {} has a NULL Vulkan buffer",
                            binding_index
                        )));
                    }
                }

                // Allocate command buffer
                let alloc_info = VkCommandBufferAllocateInfo {
                    sType: VkStructureType::CommandBufferAllocateInfo,
                    pNext: ptr::null(),
                    commandPool: inner.command_pool,
                    level: VkCommandBufferLevel::Primary,
                    commandBufferCount: 1,
                };
                
                let mut command_buffer = VkCommandBuffer::NULL;
                let result = vkAllocateCommandBuffers(inner.device, &alloc_info, &mut command_buffer);
                if result != VkResult::Success {
                    return Err(KronosError::from(result));
                }
                if command_buffer == VkCommandBuffer::NULL {
                    return Err(KronosError::CommandExecutionFailed(
                        "vkAllocateCommandBuffers returned NULL".into(),
                    ));
                }
                self.command_buffer = command_buffer;
                allocated_command_buffer = command_buffer;
                
                // Begin command buffer
                let begin_info = VkCommandBufferBeginInfo {
                    sType: VkStructureType::CommandBufferBeginInfo,
                    pNext: ptr::null(),
                    flags: VkCommandBufferUsageFlags::ONE_TIME_SUBMIT,
                    pInheritanceInfo: ptr::null(),
                };
                
                let result = vkBeginCommandBuffer(command_buffer, &begin_info);
                if result != VkResult::Success {
                    return Err(KronosError::from(result));
                }
                
                // Create and update descriptor set if we have bindings
                if has_bindings {
                    if use_persistent_descriptors {
                        #[cfg(feature = "implementation")]
                        let persistent_buffers: Vec<VkBuffer> = self.bindings.iter().map(|(_, buffer)| buffer.buffer).collect();
                        #[cfg(feature = "implementation")]
                        let descriptor_set = get_persistent_descriptor_set(inner.device, &persistent_buffers)?;
                        #[cfg(feature = "implementation")]
                        self.descriptor_set = Some(descriptor_set);
                        #[cfg(not(feature = "implementation"))]
                        {
                            return Err(KronosError::CommandExecutionFailed(
                                "Persistent descriptors are not available without implementation feature".into(),
                            ));
                        }
                    } else {
                        // Allocate descriptor set
                        let alloc_info = VkDescriptorSetAllocateInfo {
                            sType: VkStructureType::DescriptorSetAllocateInfo,
                            pNext: ptr::null(),
                            descriptorPool: inner.descriptor_pool,
                            descriptorSetCount: 1,
                            pSetLayouts: &self.pipeline.descriptor_set_layout,
                        };
                        
                        let mut descriptor_set = VkDescriptorSet::NULL;
                        let result = vkAllocateDescriptorSets(inner.device, &alloc_info, &mut descriptor_set);
                        if result != VkResult::Success {
                            return Err(KronosError::from(result));
                        }
                        if descriptor_set == VkDescriptorSet::NULL {
                            return Err(KronosError::CommandExecutionFailed(
                                "vkAllocateDescriptorSets returned NULL".into(),
                            ));
                        }
                        
                        // Update descriptor set
                        let buffer_infos: Vec<VkDescriptorBufferInfo> = self.bindings.iter().map(|(_, buffer)| {
                            VkDescriptorBufferInfo {
                                buffer: buffer.buffer,
                                offset: 0,
                                range: buffer.size as VkDeviceSize,
                            }
                        }).collect();
                        
                        let writes: Vec<VkWriteDescriptorSet> = self.bindings.iter().enumerate().map(|(i, (binding, _))| {
                            VkWriteDescriptorSet {
                                sType: VkStructureType::WriteDescriptorSet,
                                pNext: ptr::null(),
                                dstSet: descriptor_set,
                                dstBinding: *binding,
                                dstArrayElement: 0,
                                descriptorCount: 1,
                                descriptorType: VkDescriptorType::StorageBuffer,
                                pImageInfo: ptr::null(),
                                pBufferInfo: &buffer_infos[i],
                                pTexelBufferView: ptr::null(),
                            }
                        }).collect();
                        if writes.len() != buffer_infos.len() {
                            return Err(KronosError::CommandExecutionFailed(
                                "Descriptor write/buffer mismatch".into(),
                            ));
                        }
                        vkUpdateDescriptorSets(inner.device, writes.len() as u32, writes.as_ptr(), 0, ptr::null());

                        allocated_descriptor_set = descriptor_set;
                        self.descriptor_set = Some(descriptor_set);
                    }
                }
                
                // Insert barriers for buffers (smart barrier optimization)
                let barriers: Vec<VkBufferMemoryBarrier> = self.bindings.iter().map(|(_, buffer)| {
                    VkBufferMemoryBarrier {
                        sType: VkStructureType::BufferMemoryBarrier,
                        pNext: ptr::null(),
                        srcAccessMask: VkAccessFlags::TRANSFER_WRITE,
                        dstAccessMask: VkAccessFlags::SHADER_READ | VkAccessFlags::SHADER_WRITE,
                        srcQueueFamilyIndex: VK_QUEUE_FAMILY_IGNORED,
                        dstQueueFamilyIndex: VK_QUEUE_FAMILY_IGNORED,
                        buffer: buffer.buffer,
                        offset: 0,
                        size: buffer.size as VkDeviceSize,
                    }
                }).collect();
                
                if !barriers.is_empty() {
                    vkCmdPipelineBarrier(
                        command_buffer,
                        VkPipelineStageFlags::TOP_OF_PIPE,
                        VkPipelineStageFlags::COMPUTE_SHADER,
                        VkDependencyFlags::empty(),
                        0,
                        ptr::null(),
                        barriers.len() as u32,
                        barriers.as_ptr(),
                        0,
                        ptr::null(),
                    );
                }
                
                // Bind pipeline
                vkCmdBindPipeline(command_buffer, VkPipelineBindPoint::Compute, self.pipeline.pipeline);
                
                // Bind descriptor set
                if let Some(descriptor_set) = self.descriptor_set {
                    vkCmdBindDescriptorSets(
                        command_buffer,
                        VkPipelineBindPoint::Compute,
                        self.pipeline.layout,
                        0,
                        1,
                        &descriptor_set,
                        0,
                        ptr::null(),
                    );
                }
                
                // Push constants
                if !self.push_constants.is_empty() {
                    vkCmdPushConstants(
                        command_buffer,
                        self.pipeline.layout,
                        VkShaderStageFlags::COMPUTE,
                        0,
                        self.push_constants.len() as u32,
                        self.push_constants.as_ptr() as *const _,
                    );
                }
                
                // Dispatch
                vkCmdDispatch(command_buffer, self.workgroups.0, self.workgroups.1, self.workgroups.2);
                
                // End command buffer
                let result = vkEndCommandBuffer(command_buffer);
                if result != VkResult::Success {
                    return Err(KronosError::from(result));
                }
                
                // Submit (with timeline batching optimization)
                let submit_info = VkSubmitInfo {
                    sType: VkStructureType::SubmitInfo,
                    pNext: ptr::null(),
                    waitSemaphoreCount: 0,
                    pWaitSemaphores: ptr::null(),
                    pWaitDstStageMask: ptr::null(),
                    commandBufferCount: 1,
                    pCommandBuffers: &command_buffer,
                    signalSemaphoreCount: 0,
                    pSignalSemaphores: ptr::null(),
                };
                
                let result = vkQueueSubmit(inner.queue, 1, &submit_info, VkFence::NULL);
                if result != VkResult::Success {
                    return Err(KronosError::CommandExecutionFailed(
                        format!("vkQueueSubmit failed: {:?}", result)
                    ));
                }
                
                // Wait for completion
                let result = vkQueueWaitIdle(inner.queue);
                if result != VkResult::Success {
                    return Err(KronosError::SynchronizationError(format!(
                        "vkQueueWaitIdle failed: {:?}",
                        result
                    )));
                }
                
                Ok(())
            });

            self.context.with_inner(|inner| {
                if allocated_command_buffer != VkCommandBuffer::NULL {
                    vkFreeCommandBuffers(inner.device, inner.command_pool, 1, &allocated_command_buffer);
                }
                if allocated_descriptor_set != VkDescriptorSet::NULL {
                    vkFreeDescriptorSets(inner.device, inner.descriptor_pool, 1, &allocated_descriptor_set);
                }
            });
            self.command_buffer = VkCommandBuffer::NULL;
            self.descriptor_set = None;
            execute_result
        }
    }
}
