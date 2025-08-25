//! Smart barrier policy for optimal synchronization
//! 
//! Implements the 3-barrier policy:
//! 1. Upload → Read (host write to device read)
//! 2. Read → Write (shader read to shader write)  
//! 3. Write → Read (shader write to shader read)

use crate::sys::*;
use crate::core::*;

/// Vendor IDs for GPU-specific optimizations
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GpuVendor {
    AMD,
    NVIDIA, 
    Intel,
    Other,
}

impl GpuVendor {
    pub fn from_vendor_id(id: u32) -> Self {
        match id {
            0x1002 => GpuVendor::AMD,      // AMD
            0x10DE => GpuVendor::NVIDIA,   // NVIDIA
            0x8086 => GpuVendor::Intel,    // Intel
            _ => GpuVendor::Other,
        }
    }
}

/// Barrier types in our 3-barrier policy
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BarrierType {
    /// Host write → Device read (upload)
    UploadToRead,
    /// Shader read → Shader write
    ReadToWrite,
    /// Shader write → Shader read
    WriteToRead,
}

/// Optimized barrier configuration per vendor
pub struct BarrierConfig {
    pub src_stage: VkPipelineStageFlags,
    pub dst_stage: VkPipelineStageFlags,
    pub src_access: VkAccessFlags,
    pub dst_access: VkAccessFlags,
}

impl BarrierConfig {
    /// Get optimal barrier config for vendor and barrier type
    pub fn optimal_for(vendor: GpuVendor, barrier_type: BarrierType) -> Self {
        match (vendor, barrier_type) {
            // Upload → Read barriers
            (_, BarrierType::UploadToRead) => BarrierConfig {
                src_stage: VkPipelineStageFlags::HOST,
                dst_stage: VkPipelineStageFlags::COMPUTE_SHADER,
                src_access: VkAccessFlags::HOST_WRITE,
                dst_access: VkAccessFlags::SHADER_READ,
            },
            
            // Read → Write barriers
            (GpuVendor::AMD, BarrierType::ReadToWrite) => BarrierConfig {
                // AMD: Minimal sync needed within shader stages
                src_stage: VkPipelineStageFlags::COMPUTE_SHADER,
                dst_stage: VkPipelineStageFlags::COMPUTE_SHADER,
                src_access: VkAccessFlags::SHADER_READ,
                dst_access: VkAccessFlags::SHADER_WRITE,
            },
            (GpuVendor::NVIDIA, BarrierType::ReadToWrite) => BarrierConfig {
                // NVIDIA: Can often elide read-to-write barriers
                src_stage: VkPipelineStageFlags::COMPUTE_SHADER,
                dst_stage: VkPipelineStageFlags::COMPUTE_SHADER,
                src_access: VkAccessFlags::SHADER_READ,
                dst_access: VkAccessFlags::SHADER_WRITE,
            },
            (_, BarrierType::ReadToWrite) => BarrierConfig {
                // Conservative for Intel/Other
                src_stage: VkPipelineStageFlags::COMPUTE_SHADER,
                dst_stage: VkPipelineStageFlags::COMPUTE_SHADER,
                src_access: VkAccessFlags::SHADER_READ,
                dst_access: VkAccessFlags::SHADER_WRITE,
            },
            
            // Write → Read barriers
            (GpuVendor::AMD, BarrierType::WriteToRead) => BarrierConfig {
                // AMD: Full barrier needed
                src_stage: VkPipelineStageFlags::COMPUTE_SHADER,
                dst_stage: VkPipelineStageFlags::COMPUTE_SHADER,
                src_access: VkAccessFlags::SHADER_WRITE,
                dst_access: VkAccessFlags::SHADER_READ,
            },
            (GpuVendor::NVIDIA, BarrierType::WriteToRead) => BarrierConfig {
                // NVIDIA: Can use more specific stages
                src_stage: VkPipelineStageFlags::COMPUTE_SHADER,
                dst_stage: VkPipelineStageFlags::COMPUTE_SHADER,
                src_access: VkAccessFlags::SHADER_WRITE,
                dst_access: VkAccessFlags::SHADER_READ,
            },
            (_, BarrierType::WriteToRead) => BarrierConfig {
                // Conservative default
                src_stage: VkPipelineStageFlags::COMPUTE_SHADER,
                dst_stage: VkPipelineStageFlags::COMPUTE_SHADER,
                src_access: VkAccessFlags::SHADER_WRITE,
                dst_access: VkAccessFlags::SHADER_READ,
            },
        }
    }
}

/// Barrier batch for efficient submission
pub struct BarrierBatch {
    memory_barriers: Vec<VkMemoryBarrier>,
    buffer_barriers: Vec<VkBufferMemoryBarrier>,
    vendor: GpuVendor,
}

impl BarrierBatch {
    pub fn new(vendor: GpuVendor) -> Self {
        Self {
            memory_barriers: Vec::new(),
            buffer_barriers: Vec::new(),
            vendor,
        }
    }
    
    /// Add a global memory barrier
    pub fn add_memory_barrier(&mut self, barrier_type: BarrierType) {
        let config = BarrierConfig::optimal_for(self.vendor, barrier_type);
        
        self.memory_barriers.push(VkMemoryBarrier {
            sType: VkStructureType::MemoryBarrier,
            pNext: std::ptr::null(),
            srcAccessMask: config.src_access,
            dstAccessMask: config.dst_access,
        });
    }
    
    /// Add a buffer-specific barrier
    pub fn add_buffer_barrier(
        &mut self,
        buffer: VkBuffer,
        barrier_type: BarrierType,
        offset: VkDeviceSize,
        size: VkDeviceSize,
    ) {
        let config = BarrierConfig::optimal_for(self.vendor, barrier_type);
        
        self.buffer_barriers.push(VkBufferMemoryBarrier {
            sType: VkStructureType::BufferMemoryBarrier,
            pNext: std::ptr::null(),
            srcAccessMask: config.src_access,
            dstAccessMask: config.dst_access,
            srcQueueFamilyIndex: VK_QUEUE_FAMILY_IGNORED,
            dstQueueFamilyIndex: VK_QUEUE_FAMILY_IGNORED,
            buffer,
            offset,
            size,
        });
    }
    
    /// Submit all barriers in the batch
    ///
    /// # Safety
    ///
    /// This function is unsafe because:
    /// - The command_buffer must be a valid VkCommandBuffer handle in recording state
    /// - The command buffer must not be in use by another thread
    /// - All buffer handles in buffer_barriers must be valid
    /// - The ICD loader must be initialized with valid function pointers
    /// - Submitting barriers with invalid parameters causes undefined behavior
    pub unsafe fn submit(
        &self,
        command_buffer: VkCommandBuffer,
        barrier_type: BarrierType,
    ) {
        if self.memory_barriers.is_empty() && self.buffer_barriers.is_empty() {
            return; // No barriers to submit
        }
        
        let config = BarrierConfig::optimal_for(self.vendor, barrier_type);
        
        if let Some(icd) = super::icd_loader::get_icd() {
            if let Some(barrier_fn) = icd.cmd_pipeline_barrier {
                barrier_fn(
                    command_buffer,
                    config.src_stage,
                    config.dst_stage,
                    VkDependencyFlags::empty(),
                    self.memory_barriers.len() as u32,
                    if self.memory_barriers.is_empty() { 
                        std::ptr::null() 
                    } else { 
                        self.memory_barriers.as_ptr() 
                    },
                    self.buffer_barriers.len() as u32,
                    if self.buffer_barriers.is_empty() { 
                        std::ptr::null() 
                    } else { 
                        self.buffer_barriers.as_ptr() 
                    },
                    0, // No image barriers for compute
                    std::ptr::null(),
                );
            }
        }
    }
    
    /// Clear the batch for reuse
    pub fn clear(&mut self) {
        self.memory_barriers.clear();
        self.buffer_barriers.clear();
    }
}

/// Smart barrier tracker to minimize redundant barriers
pub struct BarrierTracker {
    /// Last access type per buffer
    buffer_states: std::collections::HashMap<u64, VkAccessFlags>,
    /// Pending barriers
    pending: BarrierBatch,
    /// Statistics
    stats: BarrierStats,
}

#[derive(Default, Debug)]
pub struct BarrierStats {
    pub total_barriers: u64,
    pub elided_barriers: u64,
    pub upload_barriers: u64,
    pub read_write_barriers: u64,
    pub write_read_barriers: u64,
}

impl BarrierTracker {
    pub fn new(vendor: GpuVendor) -> Self {
        Self {
            buffer_states: std::collections::HashMap::new(),
            pending: BarrierBatch::new(vendor),
            stats: BarrierStats::default(),
        }
    }
    
    /// Track buffer usage and add barrier if needed
    pub fn track_buffer_access(
        &mut self,
        buffer: VkBuffer,
        new_access: VkAccessFlags,
        offset: VkDeviceSize,
        size: VkDeviceSize,
    ) -> bool {
        let buffer_key = buffer.as_raw();
        let last_access = self.buffer_states.get(&buffer_key).copied()
            .unwrap_or(VkAccessFlags::empty());
        
        // Determine if barrier is needed
        let barrier_type = if last_access.contains(VkAccessFlags::HOST_WRITE) 
            && new_access.contains(VkAccessFlags::SHADER_READ) {
            Some(BarrierType::UploadToRead)
        } else if last_access.contains(VkAccessFlags::SHADER_READ)
            && new_access.contains(VkAccessFlags::SHADER_WRITE) {
            Some(BarrierType::ReadToWrite)
        } else if last_access.contains(VkAccessFlags::SHADER_WRITE)
            && new_access.contains(VkAccessFlags::SHADER_READ) {
            Some(BarrierType::WriteToRead)
        } else if last_access == new_access {
            None // No barrier needed
        } else {
            Some(BarrierType::WriteToRead) // Conservative default
        };
        
        if let Some(barrier_type) = barrier_type {
            self.pending.add_buffer_barrier(buffer, barrier_type, offset, size);
            self.buffer_states.insert(buffer_key, new_access);
            
            // Update stats
            self.stats.total_barriers += 1;
            match barrier_type {
                BarrierType::UploadToRead => self.stats.upload_barriers += 1,
                BarrierType::ReadToWrite => self.stats.read_write_barriers += 1,
                BarrierType::WriteToRead => self.stats.write_read_barriers += 1,
            }
            
            true
        } else {
            self.stats.elided_barriers += 1;
            false
        }
    }
    
    /// Flush pending barriers
    ///
    /// # Safety
    ///
    /// This function is unsafe because:
    /// - The command_buffer must be a valid VkCommandBuffer handle in recording state
    /// - Calls the unsafe submit() method which requires valid command buffer
    /// - The command buffer must be properly synchronized if used across threads
    /// - All tracked buffers must still be valid when barriers are flushed
    pub unsafe fn flush_barriers(&mut self, command_buffer: VkCommandBuffer) {
        if !self.pending.buffer_barriers.is_empty() {
            // Determine dominant barrier type for batch
            let barrier_type = if self.stats.upload_barriers > 0 {
                BarrierType::UploadToRead
            } else if self.stats.write_read_barriers > self.stats.read_write_barriers {
                BarrierType::WriteToRead
            } else {
                BarrierType::ReadToWrite
            };
            
            self.pending.submit(command_buffer, barrier_type);
            self.pending.clear();
        }
    }
    
    /// Get barrier statistics
    pub fn stats(&self) -> &BarrierStats {
        &self.stats
    }
    
    /// Calculate barriers per dispatch ratio
    pub fn barriers_per_dispatch(&self, dispatch_count: u64) -> f64 {
        if dispatch_count == 0 {
            0.0
        } else {
            self.stats.total_barriers as f64 / dispatch_count as f64
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_vendor_detection() {
        assert_eq!(GpuVendor::from_vendor_id(0x1002), GpuVendor::AMD);
        assert_eq!(GpuVendor::from_vendor_id(0x10DE), GpuVendor::NVIDIA);
        assert_eq!(GpuVendor::from_vendor_id(0x8086), GpuVendor::Intel);
        assert_eq!(GpuVendor::from_vendor_id(0x9999), GpuVendor::Other);
    }
    
    #[test]
    fn test_barrier_config() {
        let config = BarrierConfig::optimal_for(GpuVendor::AMD, BarrierType::UploadToRead);
        assert_eq!(config.src_stage, VkPipelineStageFlags::HOST);
        assert_eq!(config.dst_stage, VkPipelineStageFlags::COMPUTE_SHADER);
        assert_eq!(config.src_access, VkAccessFlags::HOST_WRITE);
        assert_eq!(config.dst_access, VkAccessFlags::SHADER_READ);
    }
}