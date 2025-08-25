//! Pipeline and shader management

use super::*;
use crate::*; // Import all functions from the crate root
use std::ffi::CString;
use std::fs;
use std::path::Path;
use std::ptr;

/// Compiled shader module
pub struct Shader {
    context: ComputeContext,
    module: VkShaderModule,
}

// Send + Sync for thread safety
unsafe impl Send for Shader {}
unsafe impl Sync for Shader {}

/// Compute pipeline with shader and layout
pub struct Pipeline {
    context: ComputeContext,
    pipeline: VkPipeline,
    layout: VkPipelineLayout,
    descriptor_set_layout: VkDescriptorSetLayout,
}

// Send + Sync for thread safety  
unsafe impl Send for Pipeline {}
unsafe impl Sync for Pipeline {}

/// Information about buffer bindings for a pipeline
#[derive(Debug, Clone)]
pub struct BufferBinding {
    pub binding: u32,
    pub descriptor_type: VkDescriptorType,
}

impl Default for BufferBinding {
    fn default() -> Self {
        Self {
            binding: 0,
            descriptor_type: VkDescriptorType::StorageBuffer,
        }
    }
}

/// Pipeline configuration
pub struct PipelineConfig {
    /// Entry point name (default: "main")
    pub entry_point: String,
    /// Local workgroup size (x, y, z)
    pub local_size: (u32, u32, u32),
    /// Buffer bindings
    pub bindings: Vec<BufferBinding>,
    /// Push constant size in bytes (max 128)
    pub push_constant_size: u32,
}

impl Default for PipelineConfig {
    fn default() -> Self {
        Self {
            entry_point: "main".to_string(),
            local_size: (64, 1, 1),
            bindings: Vec::new(),
            push_constant_size: 0,
        }
    }
}

impl ComputeContext {
    /// Load a shader from SPIR-V file
    pub fn load_shader<P: AsRef<Path>>(&self, path: P) -> Result<Shader> {
        let spv_data = fs::read(path)
            .map_err(|e| KronosError::ShaderCompilationFailed(format!("Failed to read shader file: {}", e)))?;
        
        self.create_shader_from_spirv(&spv_data)
    }
    
    /// Create a shader from SPIR-V bytes
    pub fn create_shader_from_spirv(&self, spirv: &[u8]) -> Result<Shader> {
        if spirv.len() % 4 != 0 {
            return Err(KronosError::ShaderCompilationFailed(
                "SPIR-V data must be 4-byte aligned".into()
            ));
        }
        
        unsafe {
            self.with_inner(|inner| {
                let create_info = VkShaderModuleCreateInfo {
                    sType: VkStructureType::ShaderModuleCreateInfo,
                    pNext: ptr::null(),
                    flags: 0,
                    codeSize: spirv.len(),
                    pCode: spirv.as_ptr() as *const u32,
                };
                
                let mut module = VkShaderModule::NULL;
                let result = vkCreateShaderModule(inner.device, &create_info, ptr::null(), &mut module);
                
                if result != VkResult::Success {
                    return Err(KronosError::ShaderCompilationFailed(
                        format!("vkCreateShaderModule failed: {:?}", result)
                    ));
                }
                
                Ok(Shader {
                    context: self.clone(),
                    module,
                })
            })
        }
    }
    
    /// Create a compute pipeline with default configuration
    pub fn create_pipeline(&self, shader: &Shader) -> Result<Pipeline> {
        self.create_pipeline_with_config(shader, PipelineConfig::default())
    }
    
    /// Create a compute pipeline with custom configuration
    pub fn create_pipeline_with_config(&self, shader: &Shader, config: PipelineConfig) -> Result<Pipeline> {
        if config.push_constant_size > 128 {
            return Err(KronosError::ShaderCompilationFailed(
                format!("Push constant size {} exceeds maximum 128 bytes", config.push_constant_size)
            ));
        }
        
        unsafe {
            self.with_inner(|inner| {
                // Create descriptor set layout for Set0 (persistent descriptors)
                let bindings: Vec<VkDescriptorSetLayoutBinding> = config.bindings.iter().map(|b| {
                    VkDescriptorSetLayoutBinding {
                        binding: b.binding,
                        descriptorType: b.descriptor_type,
                        descriptorCount: 1,
                        stageFlags: VkShaderStageFlags::COMPUTE,
                        pImmutableSamplers: ptr::null(),
                    }
                }).collect();
                
                let layout_info = VkDescriptorSetLayoutCreateInfo {
                    sType: VkStructureType::DescriptorSetLayoutCreateInfo,
                    pNext: ptr::null(),
                    flags: 0,
                    bindingCount: bindings.len() as u32,
                    pBindings: if bindings.is_empty() { ptr::null() } else { bindings.as_ptr() },
                };
                
                let mut descriptor_set_layout = VkDescriptorSetLayout::NULL;
                let result = vkCreateDescriptorSetLayout(inner.device, &layout_info, ptr::null(), &mut descriptor_set_layout);
                
                if result != VkResult::Success {
                    return Err(KronosError::from(result));
                }
                
                // Create pipeline layout
                let push_constant_range = if config.push_constant_size > 0 {
                    Some(VkPushConstantRange {
                        stageFlags: VkShaderStageFlags::COMPUTE,
                        offset: 0,
                        size: config.push_constant_size,
                    })
                } else {
                    None
                };
                
                let pipeline_layout_info = VkPipelineLayoutCreateInfo {
                    sType: VkStructureType::PipelineLayoutCreateInfo,
                    pNext: ptr::null(),
                    flags: 0,
                    setLayoutCount: 1,
                    pSetLayouts: &descriptor_set_layout,
                    pushConstantRangeCount: if push_constant_range.is_some() { 1 } else { 0 },
                    pPushConstantRanges: push_constant_range.as_ref().map_or(ptr::null(), |r| r as *const _),
                };
                
                let mut pipeline_layout = VkPipelineLayout::NULL;
                let result = vkCreatePipelineLayout(inner.device, &pipeline_layout_info, ptr::null(), &mut pipeline_layout);
                
                if result != VkResult::Success {
                    vkDestroyDescriptorSetLayout(inner.device, descriptor_set_layout, ptr::null());
                    return Err(KronosError::from(result));
                }
                
                // Create compute pipeline
                let entry_point = CString::new(config.entry_point.clone())
                    .map_err(|_| KronosError::ShaderCompilationFailed("Invalid entry point name".into()))?;
                
                let stage_info = VkPipelineShaderStageCreateInfo {
                    sType: VkStructureType::PipelineShaderStageCreateInfo,
                    pNext: ptr::null(),
                    flags: 0,
                    stage: VkShaderStageFlags::COMPUTE,
                    module: shader.module,
                    pName: entry_point.as_ptr(),
                    pSpecializationInfo: ptr::null(),
                };
                
                let pipeline_info = VkComputePipelineCreateInfo {
                    sType: VkStructureType::ComputePipelineCreateInfo,
                    pNext: ptr::null(),
                    flags: 0,
                    stage: stage_info,
                    layout: pipeline_layout,
                    basePipelineHandle: VkPipeline::NULL,
                    basePipelineIndex: -1,
                };
                
                let mut pipeline = VkPipeline::NULL;
                let result = vkCreateComputePipelines(
                    inner.device,
                    VkPipelineCache::NULL,
                    1,
                    &pipeline_info,
                    ptr::null(),
                    &mut pipeline,
                );
                
                if result != VkResult::Success {
                    vkDestroyPipelineLayout(inner.device, pipeline_layout, ptr::null());
                    vkDestroyDescriptorSetLayout(inner.device, descriptor_set_layout, ptr::null());
                    return Err(KronosError::from(result));
                }
                
                Ok(Pipeline {
                    context: self.clone(),
                    pipeline,
                    layout: pipeline_layout,
                    descriptor_set_layout,
                })
            })
        }
    }
}

impl Pipeline {
    /// Get the raw Vulkan pipeline handle (for advanced usage)
    pub fn raw(&self) -> VkPipeline {
        self.pipeline
    }
    
    /// Get the pipeline layout
    pub fn layout(&self) -> VkPipelineLayout {
        self.layout
    }
    
    /// Get the descriptor set layout
    pub fn descriptor_set_layout(&self) -> VkDescriptorSetLayout {
        self.descriptor_set_layout
    }
}

impl Drop for Shader {
    fn drop(&mut self) {
        unsafe {
            self.context.with_inner(|inner| {
                vkDestroyShaderModule(inner.device, self.module, ptr::null());
            });
        }
    }
}

impl Drop for Pipeline {
    fn drop(&mut self) {
        unsafe {
            self.context.with_inner(|inner| {
                vkDestroyPipeline(inner.device, self.pipeline, ptr::null());
                vkDestroyPipelineLayout(inner.device, self.layout, ptr::null());
                vkDestroyDescriptorSetLayout(inner.device, self.descriptor_set_layout, ptr::null());
            });
        }
    }
}