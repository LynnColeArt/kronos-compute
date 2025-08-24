//! Descriptor set management implementation

use std::sync::{Arc, Mutex};
use std::collections::HashMap;
use crate::sys::*;
use crate::core::*;
use crate::ffi::*;

lazy_static::lazy_static! {
    // Global storage for descriptor resources
    static ref DESCRIPTOR_SET_LAYOUTS: Mutex<HashMap<u64, DescriptorSetLayout>> = Mutex::new(HashMap::new());
    static ref DESCRIPTOR_POOLS: Mutex<HashMap<u64, DescriptorPool>> = Mutex::new(HashMap::new());
    static ref DESCRIPTOR_SETS: Mutex<HashMap<u64, DescriptorSet>> = Mutex::new(HashMap::new());
}

struct DescriptorSetLayout {
    handle: VkDescriptorSetLayout,
    bindings: Vec<VkDescriptorSetLayoutBinding>,
}

struct DescriptorPool {
    handle: VkDescriptorPool,
    max_sets: u32,
    pool_sizes: Vec<VkDescriptorPoolSize>,
    allocated_sets: Vec<VkDescriptorSet>,
    available_counts: HashMap<VkDescriptorType, u32>,
}

struct DescriptorSet {
    handle: VkDescriptorSet,
    layout: VkDescriptorSetLayout,
    pool: VkDescriptorPool,
    bindings: HashMap<u32, DescriptorBinding>,
}

#[derive(Clone)]
struct DescriptorBinding {
    binding: u32,
    descriptor_type: VkDescriptorType,
    descriptors: Vec<Descriptor>,
}

#[derive(Clone)]
enum Descriptor {
    Buffer {
        buffer: VkBuffer,
        offset: VkDeviceSize,
        range: VkDeviceSize,
    },
    Image {
        sampler: VkSampler,
        image_view: VkImageView,
        layout: VkImageLayout,
    },
    TexelBuffer {
        buffer_view: VkBufferView,
    },
}

/// Create descriptor set layout
#[no_mangle]
pub unsafe extern "C" fn vkCreateDescriptorSetLayout(
    device: VkDevice,
    pCreateInfo: *const VkDescriptorSetLayoutCreateInfo,
    pAllocator: *const VkAllocationCallbacks,
    pSetLayout: *mut VkDescriptorSetLayout,
) -> VkResult {
    if device.is_null() || pCreateInfo.is_null() || pSetLayout.is_null() {
        return VkResult::ErrorInitializationFailed;
    }
    
    // Forward to real ICD if enabled
    if let Some(icd) = super::forward::get_icd_if_enabled() {
        if let Some(create_descriptor_set_layout) = icd.create_descriptor_set_layout {
            return create_descriptor_set_layout(device, pCreateInfo, pAllocator, pSetLayout);
        }
    }
    
    let create_info = &*pCreateInfo;
    
    if create_info.sType != VkStructureType::DescriptorSetLayoutCreateInfo {
        return VkResult::ErrorInitializationFailed;
    }
    
    // Copy bindings
    let bindings = if create_info.bindingCount > 0 {
        std::slice::from_raw_parts(create_info.pBindings, create_info.bindingCount as usize).to_vec()
    } else {
        Vec::new()
    };
    
    // Validate bindings
    for binding in &bindings {
        // For compute, we only support certain descriptor types
        match binding.descriptorType {
            VkDescriptorType::Sampler |
            VkDescriptorType::SampledImage |
            VkDescriptorType::StorageImage |
            VkDescriptorType::UniformBuffer |
            VkDescriptorType::StorageBuffer |
            VkDescriptorType::UniformBufferDynamic |
            VkDescriptorType::StorageBufferDynamic => {},
            _ => return VkResult::ErrorFeatureNotPresent,
        }
        
        // Must have compute stage
        if !binding.stageFlags.contains(VkShaderStageFlags::COMPUTE) {
            return VkResult::ErrorInitializationFailed;
        }
    }
    
    let handle = VkDescriptorSetLayout::from_raw(
        DESCRIPTOR_SET_LAYOUTS.lock().unwrap().len() as u64 + 1
    );
    
    let layout = DescriptorSetLayout {
        handle,
        bindings,
    };
    
    DESCRIPTOR_SET_LAYOUTS.lock().unwrap().insert(handle.as_raw(), layout);
    
    *pSetLayout = handle;
    VkResult::Success
}

/// Destroy descriptor set layout
#[no_mangle]
pub unsafe extern "C" fn vkDestroyDescriptorSetLayout(
    device: VkDevice,
    descriptorSetLayout: VkDescriptorSetLayout,
    _pAllocator: *const VkAllocationCallbacks,
) {
    if device.is_null() || descriptorSetLayout.is_null() {
        return;
    }
    
    DESCRIPTOR_SET_LAYOUTS.lock().unwrap().remove(&descriptorSetLayout.as_raw());
}

/// Create descriptor pool
#[no_mangle]
pub unsafe extern "C" fn vkCreateDescriptorPool(
    device: VkDevice,
    pCreateInfo: *const VkDescriptorPoolCreateInfo,
    pAllocator: *const VkAllocationCallbacks,
    pDescriptorPool: *mut VkDescriptorPool,
) -> VkResult {
    if device.is_null() || pCreateInfo.is_null() || pDescriptorPool.is_null() {
        return VkResult::ErrorInitializationFailed;
    }
    
    // Forward to real ICD if enabled
    if let Some(icd) = super::forward::get_icd_if_enabled() {
        if let Some(create_descriptor_pool) = icd.create_descriptor_pool {
            return create_descriptor_pool(device, pCreateInfo, pAllocator, pDescriptorPool);
        }
    }
    
    let create_info = &*pCreateInfo;
    
    if create_info.sType != VkStructureType::DescriptorPoolCreateInfo {
        return VkResult::ErrorInitializationFailed;
    }
    
    // Copy pool sizes
    let pool_sizes = if create_info.poolSizeCount > 0 {
        std::slice::from_raw_parts(create_info.pPoolSizes, create_info.poolSizeCount as usize).to_vec()
    } else {
        Vec::new()
    };
    
    // Build available counts map
    let mut available_counts = HashMap::new();
    for size in &pool_sizes {
        available_counts.insert(size.type_, size.descriptorCount);
    }
    
    let handle = VkDescriptorPool::from_raw(
        DESCRIPTOR_POOLS.lock().unwrap().len() as u64 + 1
    );
    
    let pool = DescriptorPool {
        handle,
        max_sets: create_info.maxSets,
        pool_sizes,
        allocated_sets: Vec::new(),
        available_counts,
    };
    
    DESCRIPTOR_POOLS.lock().unwrap().insert(handle.as_raw(), pool);
    
    *pDescriptorPool = handle;
    VkResult::Success
}

/// Destroy descriptor pool
#[no_mangle]
pub unsafe extern "C" fn vkDestroyDescriptorPool(
    device: VkDevice,
    descriptorPool: VkDescriptorPool,
    _pAllocator: *const VkAllocationCallbacks,
) {
    if device.is_null() || descriptorPool.is_null() {
        return;
    }
    
    // Free all descriptor sets in this pool
    let pools = DESCRIPTOR_POOLS.lock().unwrap();
    if let Some(pool) = pools.get(&descriptorPool.as_raw()) {
        let mut sets = DESCRIPTOR_SETS.lock().unwrap();
        for &set in &pool.allocated_sets {
            sets.remove(&set.as_raw());
        }
    }
    drop(pools);
    
    DESCRIPTOR_POOLS.lock().unwrap().remove(&descriptorPool.as_raw());
}

/// Allocate descriptor sets
#[no_mangle]
pub unsafe extern "C" fn vkAllocateDescriptorSets(
    device: VkDevice,
    pAllocateInfo: *const VkDescriptorSetAllocateInfo,
    pDescriptorSets: *mut VkDescriptorSet,
) -> VkResult {
    if device.is_null() || pAllocateInfo.is_null() || pDescriptorSets.is_null() {
        return VkResult::ErrorInitializationFailed;
    }
    
    // Forward to real ICD if enabled
    if let Some(icd) = super::forward::get_icd_if_enabled() {
        if let Some(allocate_descriptor_sets) = icd.allocate_descriptor_sets {
            return allocate_descriptor_sets(device, pAllocateInfo, pDescriptorSets);
        }
    }
    
    let alloc_info = &*pAllocateInfo;
    
    if alloc_info.sType != VkStructureType::DescriptorSetAllocateInfo {
        return VkResult::ErrorInitializationFailed;
    }
    
    // Get pool
    let mut pools = DESCRIPTOR_POOLS.lock().unwrap();
    let pool = match pools.get_mut(&alloc_info.descriptorPool.as_raw()) {
        Some(p) => p,
        None => return VkResult::ErrorInitializationFailed,
    };
    
    // Check if we have space
    if pool.allocated_sets.len() + alloc_info.descriptorSetCount as usize > pool.max_sets as usize {
        return VkResult::ErrorOutOfPoolMemory;
    }
    
    // Get layouts
    let set_layouts = std::slice::from_raw_parts(
        alloc_info.pSetLayouts,
        alloc_info.descriptorSetCount as usize
    );
    
    let layouts = DESCRIPTOR_SET_LAYOUTS.lock().unwrap();
    let mut sets = DESCRIPTOR_SETS.lock().unwrap();
    
    for i in 0..alloc_info.descriptorSetCount {
        let layout_handle = set_layouts[i as usize];
        let layout = match layouts.get(&layout_handle.as_raw()) {
            Some(l) => l,
            None => return VkResult::ErrorInitializationFailed,
        };
        
        // Check if pool has enough descriptors
        for binding in &layout.bindings {
            let required = binding.descriptorCount;
            let available = pool.available_counts.get(&binding.descriptorType).copied().unwrap_or(0);
            if available < required {
                return VkResult::ErrorOutOfPoolMemory;
            }
        }
        
        // Allocate descriptor set
        let handle = VkDescriptorSet::from_raw(sets.len() as u64 + 1);
        
        // Create binding map
        let mut bindings = HashMap::new();
        for binding in &layout.bindings {
            bindings.insert(binding.binding, DescriptorBinding {
                binding: binding.binding,
                descriptor_type: binding.descriptorType,
                descriptors: vec![],
            });
            
            // Deduct from pool
            if let Some(count) = pool.available_counts.get_mut(&binding.descriptorType) {
                *count -= binding.descriptorCount;
            }
        }
        
        let set = DescriptorSet {
            handle,
            layout: layout_handle,
            pool: alloc_info.descriptorPool,
            bindings,
        };
        
        pool.allocated_sets.push(handle);
        sets.insert(handle.as_raw(), set);
        
        *pDescriptorSets.add(i as usize) = handle;
    }
    
    VkResult::Success
}

/// Update descriptor sets
#[no_mangle]
pub unsafe extern "C" fn vkUpdateDescriptorSets(
    device: VkDevice,
    descriptorWriteCount: u32,
    pDescriptorWrites: *const VkWriteDescriptorSet,
    descriptorCopyCount: u32,
    pDescriptorCopies: *const VkCopyDescriptorSet,
) {
    if device.is_null() {
        return;
    }
    
    let mut sets = DESCRIPTOR_SETS.lock().unwrap();
    
    // Process writes
    if descriptorWriteCount > 0 && !pDescriptorWrites.is_null() {
        let writes = std::slice::from_raw_parts(pDescriptorWrites, descriptorWriteCount as usize);
        
        for write in writes {
            if write.sType != VkStructureType::WriteDescriptorSet {
                continue;
            }
            
            let set = match sets.get_mut(&write.dstSet.as_raw()) {
                Some(s) => s,
                None => continue,
            };
            
            let binding = match set.bindings.get_mut(&write.dstBinding) {
                Some(b) => b,
                None => continue,
            };
            
            // Clear old descriptors in range
            let start = write.dstArrayElement as usize;
            let count = write.descriptorCount as usize;
            
            // Ensure vector is large enough
            if binding.descriptors.len() < start + count {
                binding.descriptors.resize(start + count, match binding.descriptor_type {
                    VkDescriptorType::UniformBuffer |
                    VkDescriptorType::StorageBuffer |
                    VkDescriptorType::UniformBufferDynamic |
                    VkDescriptorType::StorageBufferDynamic => {
                        Descriptor::Buffer {
                            buffer: VkBuffer::NULL,
                            offset: 0,
                            range: 0,
                        }
                    },
                    _ => Descriptor::Buffer {
                        buffer: VkBuffer::NULL,
                        offset: 0,
                        range: 0,
                    },
                });
            }
            
            // Update descriptors based on type
            match binding.descriptor_type {
                VkDescriptorType::UniformBuffer |
                VkDescriptorType::StorageBuffer |
                VkDescriptorType::UniformBufferDynamic |
                VkDescriptorType::StorageBufferDynamic => {
                    if !write.pBufferInfo.is_null() {
                        let buffer_infos = std::slice::from_raw_parts(
                            write.pBufferInfo,
                            write.descriptorCount as usize
                        );
                        
                        for (i, info) in buffer_infos.iter().enumerate() {
                            binding.descriptors[start + i] = Descriptor::Buffer {
                                buffer: info.buffer,
                                offset: info.offset,
                                range: info.range,
                            };
                        }
                    }
                },
                VkDescriptorType::SampledImage |
                VkDescriptorType::StorageImage |
                VkDescriptorType::Sampler => {
                    if !write.pImageInfo.is_null() {
                        let image_infos = std::slice::from_raw_parts(
                            write.pImageInfo,
                            write.descriptorCount as usize
                        );
                        
                        for (i, info) in image_infos.iter().enumerate() {
                            binding.descriptors[start + i] = Descriptor::Image {
                                sampler: info.sampler,
                                image_view: info.imageView,
                                layout: info.imageLayout,
                            };
                        }
                    }
                },
                _ => {},
            }
        }
    }
    
    // Process copies
    if descriptorCopyCount > 0 && !pDescriptorCopies.is_null() {
        let copies = std::slice::from_raw_parts(pDescriptorCopies, descriptorCopyCount as usize);
        
        for copy in copies {
            if copy.sType != VkStructureType::CopyDescriptorSet {
                continue;
            }
            
            // Clone source descriptors
            let src_descriptors = {
                let src_set = match sets.get(&copy.srcSet.as_raw()) {
                    Some(s) => s,
                    None => continue,
                };
                
                let src_binding = match src_set.bindings.get(&copy.srcBinding) {
                    Some(b) => b,
                    None => continue,
                };
                
                let start = copy.srcArrayElement as usize;
                let count = copy.descriptorCount as usize;
                
                if start + count > src_binding.descriptors.len() {
                    continue;
                }
                
                src_binding.descriptors[start..start + count].to_vec()
            };
            
            // Update destination
            let dst_set = match sets.get_mut(&copy.dstSet.as_raw()) {
                Some(s) => s,
                None => continue,
            };
            
            let dst_binding = match dst_set.bindings.get_mut(&copy.dstBinding) {
                Some(b) => b,
                None => continue,
            };
            
            let dst_start = copy.dstArrayElement as usize;
            let count = copy.descriptorCount as usize;
            
            // Ensure destination vector is large enough
            if dst_binding.descriptors.len() < dst_start + count {
                // Create default descriptor based on type
                let default_desc = match dst_binding.descriptor_type {
                    VkDescriptorType::UniformBuffer |
                    VkDescriptorType::StorageBuffer |
                    VkDescriptorType::UniformBufferDynamic |
                    VkDescriptorType::StorageBufferDynamic => {
                        Descriptor::Buffer {
                            buffer: VkBuffer::NULL,
                            offset: 0,
                            range: 0,
                        }
                    },
                    _ => Descriptor::Buffer {
                        buffer: VkBuffer::NULL,
                        offset: 0,
                        range: 0,
                    },
                };
                dst_binding.descriptors.resize(dst_start + count, default_desc);
            }
            
            // Copy descriptors
            dst_binding.descriptors[dst_start..dst_start + count]
                .clone_from_slice(&src_descriptors);
        }
    }
}

/// Bind descriptor sets to command buffer
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
    if commandBuffer.is_null() || pDescriptorSets.is_null() {
        return;
    }
    
    if pipelineBindPoint != VkPipelineBindPoint::Compute {
        return; // We only support compute
    }
    
    // Copy descriptor sets
    let sets = std::slice::from_raw_parts(pDescriptorSets, descriptorSetCount as usize).to_vec();
    
    // Copy dynamic offsets
    let dynamic_offsets = if dynamicOffsetCount > 0 && !pDynamicOffsets.is_null() {
        std::slice::from_raw_parts(pDynamicOffsets, dynamicOffsetCount as usize).to_vec()
    } else {
        Vec::new()
    };
    
    // Add command to command buffer
    let mut buffers = super::compute::COMMAND_BUFFERS.lock().unwrap();
    if let Some(buffer) = buffers.get_mut(&commandBuffer.as_raw()) {
        buffer.commands.push(super::compute::Command::BindDescriptorSets {
            layout,
            first_set: firstSet,
            sets,
            dynamic_offsets,
        });
    }
}