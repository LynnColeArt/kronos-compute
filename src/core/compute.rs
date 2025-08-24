//! Compute-specific structures for Kronos

use std::ffi::c_void;
use std::ptr;
use crate::sys::*;
use crate::core::enums::*;
use crate::core::flags::*;

/// Shader module creation info
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct VkShaderModuleCreateInfo {
    pub sType: VkStructureType,
    pub pNext: *const c_void,
    pub flags: VkFlags,
    pub codeSize: usize,
    pub pCode: *const u32,
}

impl Default for VkShaderModuleCreateInfo {
    fn default() -> Self {
        Self {
            sType: VkStructureType::ShaderModuleCreateInfo,
            pNext: ptr::null(),
            flags: 0,
            codeSize: 0,
            pCode: ptr::null(),
        }
    }
}

/// Specialization map entry
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct VkSpecializationMapEntry {
    pub constantID: u32,
    pub offset: u32,
    pub size: usize,
}

/// Specialization info
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct VkSpecializationInfo {
    pub mapEntryCount: u32,
    pub pMapEntries: *const VkSpecializationMapEntry,
    pub dataSize: usize,
    pub pData: *const c_void,
}

/// Pipeline shader stage creation info
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct VkPipelineShaderStageCreateInfo {
    pub sType: VkStructureType,
    pub pNext: *const c_void,
    pub flags: VkPipelineShaderStageCreateFlags,
    pub stage: VkShaderStageFlagBits,
    pub module: VkShaderModule,
    pub pName: *const i8,
    pub pSpecializationInfo: *const VkSpecializationInfo,
}

impl Default for VkPipelineShaderStageCreateInfo {
    fn default() -> Self {
        Self {
            sType: VkStructureType::PipelineShaderStageCreateInfo,
            pNext: ptr::null(),
            flags: 0,
            stage: VkShaderStageFlagBits::Compute,
            module: VkShaderModule::NULL,
            pName: b"main\0".as_ptr() as *const i8,
            pSpecializationInfo: ptr::null(),
        }
    }
}

/// Compute pipeline creation info (optimized)
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct VkComputePipelineCreateInfo {
    pub sType: VkStructureType,
    pub pNext: *const c_void,
    pub flags: VkPipelineCreateFlags,
    pub stage: VkPipelineShaderStageCreateInfo,
    pub layout: VkPipelineLayout,
    pub basePipelineHandle: VkPipeline,
    pub basePipelineIndex: i32,
}

impl Default for VkComputePipelineCreateInfo {
    fn default() -> Self {
        Self {
            sType: VkStructureType::ComputePipelineCreateInfo,
            pNext: ptr::null(),
            flags: VkPipelineCreateFlags::empty(),
            stage: VkPipelineShaderStageCreateInfo::default(),
            layout: VkPipelineLayout::NULL,
            basePipelineHandle: VkPipeline::NULL,
            basePipelineIndex: -1,
        }
    }
}

/// Push constant range
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct VkPushConstantRange {
    pub stageFlags: VkShaderStageFlags,
    pub offset: u32,
    pub size: u32,
}

/// Pipeline layout creation info
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct VkPipelineLayoutCreateInfo {
    pub sType: VkStructureType,
    pub pNext: *const c_void,
    pub flags: VkPipelineLayoutCreateFlags,
    pub setLayoutCount: u32,
    pub pSetLayouts: *const VkDescriptorSetLayout,
    pub pushConstantRangeCount: u32,
    pub pPushConstantRanges: *const VkPushConstantRange,
}

impl Default for VkPipelineLayoutCreateInfo {
    fn default() -> Self {
        Self {
            sType: VkStructureType::PipelineLayoutCreateInfo,
            pNext: ptr::null(),
            flags: 0,
            setLayoutCount: 0,
            pSetLayouts: ptr::null(),
            pushConstantRangeCount: 0,
            pPushConstantRanges: ptr::null(),
        }
    }
}

/// Descriptor set layout binding
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct VkDescriptorSetLayoutBinding {
    pub binding: u32,
    pub descriptorType: VkDescriptorType,
    pub descriptorCount: u32,
    pub stageFlags: VkShaderStageFlags,
    pub pImmutableSamplers: *const VkSampler,
}

/// Descriptor set layout creation info
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct VkDescriptorSetLayoutCreateInfo {
    pub sType: VkStructureType,
    pub pNext: *const c_void,
    pub flags: VkDescriptorSetLayoutCreateFlags,
    pub bindingCount: u32,
    pub pBindings: *const VkDescriptorSetLayoutBinding,
}

impl Default for VkDescriptorSetLayoutCreateInfo {
    fn default() -> Self {
        Self {
            sType: VkStructureType::DescriptorSetLayoutCreateInfo,
            pNext: ptr::null(),
            flags: 0,
            bindingCount: 0,
            pBindings: ptr::null(),
        }
    }
}

/// Descriptor pool size
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct VkDescriptorPoolSize {
    pub type_: VkDescriptorType,
    pub descriptorCount: u32,
}

/// Descriptor pool creation info
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct VkDescriptorPoolCreateInfo {
    pub sType: VkStructureType,
    pub pNext: *const c_void,
    pub flags: VkDescriptorPoolCreateFlags,
    pub maxSets: u32,
    pub poolSizeCount: u32,
    pub pPoolSizes: *const VkDescriptorPoolSize,
}

impl Default for VkDescriptorPoolCreateInfo {
    fn default() -> Self {
        Self {
            sType: VkStructureType::DescriptorPoolCreateInfo,
            pNext: ptr::null(),
            flags: VkDescriptorPoolCreateFlags::empty(),
            maxSets: 0,
            poolSizeCount: 0,
            pPoolSizes: ptr::null(),
        }
    }
}

/// Descriptor set allocate info
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct VkDescriptorSetAllocateInfo {
    pub sType: VkStructureType,
    pub pNext: *const c_void,
    pub descriptorPool: VkDescriptorPool,
    pub descriptorSetCount: u32,
    pub pSetLayouts: *const VkDescriptorSetLayout,
}

impl Default for VkDescriptorSetAllocateInfo {
    fn default() -> Self {
        Self {
            sType: VkStructureType::DescriptorSetAllocateInfo,
            pNext: ptr::null(),
            descriptorPool: VkDescriptorPool::NULL,
            descriptorSetCount: 0,
            pSetLayouts: ptr::null(),
        }
    }
}

/// Descriptor buffer info
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct VkDescriptorBufferInfo {
    pub buffer: VkBuffer,
    pub offset: VkDeviceSize,
    pub range: VkDeviceSize,
}

/// Descriptor image info
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct VkDescriptorImageInfo {
    pub sampler: VkSampler,
    pub imageView: VkImageView,
    pub imageLayout: VkImageLayout,
}

/// Write descriptor set
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct VkWriteDescriptorSet {
    pub sType: VkStructureType,
    pub pNext: *const c_void,
    pub dstSet: VkDescriptorSet,
    pub dstBinding: u32,
    pub dstArrayElement: u32,
    pub descriptorCount: u32,
    pub descriptorType: VkDescriptorType,
    pub pImageInfo: *const VkDescriptorImageInfo,
    pub pBufferInfo: *const VkDescriptorBufferInfo,
    pub pTexelBufferView: *const VkBufferView,
}

impl Default for VkWriteDescriptorSet {
    fn default() -> Self {
        Self {
            sType: VkStructureType::WriteDescriptorSet,
            pNext: ptr::null(),
            dstSet: VkDescriptorSet::NULL,
            dstBinding: 0,
            dstArrayElement: 0,
            descriptorCount: 0,
            descriptorType: VkDescriptorType::StorageBuffer,
            pImageInfo: ptr::null(),
            pBufferInfo: ptr::null(),
            pTexelBufferView: ptr::null(),
        }
    }
}

/// Copy descriptor set
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct VkCopyDescriptorSet {
    pub sType: VkStructureType,
    pub pNext: *const c_void,
    pub srcSet: VkDescriptorSet,
    pub srcBinding: u32,
    pub srcArrayElement: u32,
    pub dstSet: VkDescriptorSet,
    pub dstBinding: u32,
    pub dstArrayElement: u32,
    pub descriptorCount: u32,
}

// Add missing handle types
pub type VkSampler = Handle<SamplerT>;
pub type VkImageView = Handle<ImageViewT>;
pub type VkBufferView = Handle<BufferViewT>;

#[derive(Debug, Clone, Copy)]
pub enum SamplerT {}
#[derive(Debug, Clone, Copy)]
pub enum ImageViewT {}
#[derive(Debug, Clone, Copy)]
pub enum BufferViewT {}

// Add missing enums
#[repr(i32)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum VkImageLayout {
    Undefined = 0,
    General = 1,
    TransferSrcOptimal = 6,
    TransferDstOptimal = 7,
    SharedPresentKHR = 1000111000,
}

// Add missing flags
pub type VkPipelineShaderStageCreateFlags = VkFlags;