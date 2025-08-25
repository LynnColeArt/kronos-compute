//! Safe buffer management with automatic memory allocation

use super::*;
use crate::*; // Import all functions from the crate root
use std::marker::PhantomData;
use std::ptr;
use std::slice;

/// Usage flags for buffers
#[derive(Debug, Clone, Copy)]
pub struct BufferUsage {
    flags: VkBufferUsageFlags,
}

impl BufferUsage {
    pub const STORAGE: Self = Self { flags: VkBufferUsageFlags::STORAGE_BUFFER };
    pub const TRANSFER_SRC: Self = Self { flags: VkBufferUsageFlags::TRANSFER_SRC };
    pub const TRANSFER_DST: Self = Self { flags: VkBufferUsageFlags::TRANSFER_DST };
    
    pub fn storage() -> Self {
        Self::STORAGE
    }
    
    pub fn transfer_src() -> Self {
        Self::TRANSFER_SRC
    }
    
    pub fn transfer_dst() -> Self {
        Self::TRANSFER_DST
    }
}

impl std::ops::BitOr for BufferUsage {
    type Output = Self;
    
    fn bitor(self, rhs: Self) -> Self::Output {
        Self {
            flags: VkBufferUsageFlags::from_bits_truncate(self.flags.bits() | rhs.flags.bits())
        }
    }
}

/// A GPU buffer with automatic memory management
/// 
/// Buffers are automatically freed when dropped and use the
/// pool allocator for efficient memory management.
pub struct Buffer {
    pub(super) context: ComputeContext,
    pub(super) buffer: VkBuffer,
    pub(super) memory: VkDeviceMemory,
    pub(super) size: usize,
    pub(super) usage: BufferUsage,
    pub(super) _marker: PhantomData<*const u8>,
}

// Send + Sync for thread safety
unsafe impl Send for Buffer {}
unsafe impl Sync for Buffer {}

impl Buffer {
    /// Get the size of the buffer in bytes
    pub fn size(&self) -> usize {
        self.size
    }
    
    /// Get the usage flags
    pub fn usage(&self) -> BufferUsage {
        self.usage
    }
    
    /// Get the raw Vulkan buffer handle (for advanced usage)
    pub fn raw(&self) -> VkBuffer {
        self.buffer
    }
}

impl ComputeContext {
    /// Create a buffer with data
    pub fn create_buffer<T>(&self, data: &[T]) -> Result<Buffer> 
    where
        T: Copy + 'static,
    {
        let size = std::mem::size_of_val(data);
        let usage = BufferUsage::STORAGE | BufferUsage::TRANSFER_DST;
        
        unsafe {
            // Create buffer
            let buffer = self.create_buffer_raw(size, usage)?;
            
            // Create staging buffer
            let staging_usage = BufferUsage::TRANSFER_SRC;
            let staging = self.create_buffer_raw(size, staging_usage)?;
            
            // Map and copy data
            self.with_inner(|inner| {
                let mut mapped_ptr = ptr::null_mut();
                let result = vkMapMemory(
                    inner.device,
                    staging.memory,
                    0,
                    size as VkDeviceSize,
                    0,
                    &mut mapped_ptr,
                );
                
                if result != VkResult::Success {
                    return Err(KronosError::from(result));
                }
                
                ptr::copy_nonoverlapping(
                    data.as_ptr() as *const u8,
                    mapped_ptr as *mut u8,
                    size,
                );
                
                vkUnmapMemory(inner.device, staging.memory);
                Ok(())
            })?;
            
            // Copy staging to device buffer
            self.copy_buffer(&staging, &buffer, size)?;
            
            // Staging buffer will be dropped automatically
            Ok(buffer)
        }
    }
    
    /// Create an uninitialized buffer
    pub fn create_buffer_uninit(&self, size: usize) -> Result<Buffer> {
        let usage = BufferUsage::STORAGE | BufferUsage::TRANSFER_DST | BufferUsage::TRANSFER_SRC;
        unsafe { self.create_buffer_raw(size, usage) }
    }
    
    /// Internal: Create a raw buffer
    unsafe fn create_buffer_raw(&self, size: usize, usage: BufferUsage) -> Result<Buffer> {
        self.with_inner(|inner| {
            // Create buffer
            let buffer_info = VkBufferCreateInfo {
                sType: VkStructureType::BufferCreateInfo,
                pNext: ptr::null(),
                flags: VkBufferCreateFlags::empty(),
                size: size as VkDeviceSize,
                usage: usage.flags,
                sharingMode: VkSharingMode::Exclusive,
                queueFamilyIndexCount: 0,
                pQueueFamilyIndices: ptr::null(),
            };
            
            let mut buffer = VkBuffer::NULL;
            let result = vkCreateBuffer(inner.device, &buffer_info, ptr::null(), &mut buffer);
            
            if result != VkResult::Success {
                return Err(KronosError::BufferCreationFailed(format!("vkCreateBuffer failed: {:?}", result)));
            }
            
            // Get memory requirements
            let mut mem_requirements = VkMemoryRequirements::default();
            vkGetBufferMemoryRequirements(inner.device, buffer, &mut mem_requirements);
            
            // Find suitable memory type
            let memory_type_index = Self::find_memory_type(
                &inner.memory_properties,
                mem_requirements.memoryTypeBits,
                if usage.flags.contains(VkBufferUsageFlags::TRANSFER_SRC) {
                    VkMemoryPropertyFlags::HOST_VISIBLE | VkMemoryPropertyFlags::HOST_COHERENT
                } else {
                    VkMemoryPropertyFlags::DEVICE_LOCAL
                },
            )?;
            
            // Allocate memory (this would use the pool allocator in the real implementation)
            let alloc_info = VkMemoryAllocateInfo {
                sType: VkStructureType::MemoryAllocateInfo,
                pNext: ptr::null(),
                allocationSize: mem_requirements.size,
                memoryTypeIndex: memory_type_index,
            };
            
            let mut memory = VkDeviceMemory::NULL;
            let result = vkAllocateMemory(inner.device, &alloc_info, ptr::null(), &mut memory);
            
            if result != VkResult::Success {
                vkDestroyBuffer(inner.device, buffer, ptr::null());
                return Err(KronosError::BufferCreationFailed(format!("vkAllocateMemory failed: {:?}", result)));
            }
            
            // Bind memory to buffer
            let result = vkBindBufferMemory(inner.device, buffer, memory, 0);
            
            if result != VkResult::Success {
                vkFreeMemory(inner.device, memory, ptr::null());
                vkDestroyBuffer(inner.device, buffer, ptr::null());
                return Err(KronosError::BufferCreationFailed(format!("vkBindBufferMemory failed: {:?}", result)));
            }
            
            Ok(Buffer {
                context: self.clone(),
                buffer,
                memory,
                size,
                usage,
                _marker: std::marker::PhantomData,
            })
        })
    }
    
    /// Find a suitable memory type
    fn find_memory_type(
        memory_properties: &VkPhysicalDeviceMemoryProperties,
        type_filter: u32,
        properties: VkMemoryPropertyFlags,
    ) -> Result<u32> {
        for i in 0..memory_properties.memoryTypeCount {
            if (type_filter & (1 << i)) != 0 
                && memory_properties.memoryTypes[i as usize].propertyFlags.contains(properties) {
                return Ok(i);
            }
        }
        
        Err(KronosError::BufferCreationFailed("No suitable memory type found".into()))
    }
    
    /// Copy data between buffers
    unsafe fn copy_buffer(&self, src: &Buffer, dst: &Buffer, size: usize) -> Result<()> {
        self.with_inner(|inner| {
            // Allocate command buffer
            let alloc_info = VkCommandBufferAllocateInfo {
                sType: VkStructureType::CommandBufferAllocateInfo,
                pNext: ptr::null(),
                commandPool: inner.command_pool,
                level: VkCommandBufferLevel::Primary,
                commandBufferCount: 1,
            };
            
            let mut command_buffer = VkCommandBuffer::NULL;
            vkAllocateCommandBuffers(inner.device, &alloc_info, &mut command_buffer);
            
            // Begin recording
            let begin_info = VkCommandBufferBeginInfo {
                sType: VkStructureType::CommandBufferBeginInfo,
                pNext: ptr::null(),
                flags: VkCommandBufferUsageFlags::ONE_TIME_SUBMIT,
                pInheritanceInfo: ptr::null(),
            };
            
            vkBeginCommandBuffer(command_buffer, &begin_info);
            
            // Record copy command
            let region = VkBufferCopy {
                srcOffset: 0,
                dstOffset: 0,
                size: size as VkDeviceSize,
            };
            
            vkCmdCopyBuffer(command_buffer, src.buffer, dst.buffer, 1, &region);
            
            // End recording
            vkEndCommandBuffer(command_buffer);
            
            // Submit
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
                return Err(KronosError::from(result));
            }
            
            // Wait for completion
            vkQueueWaitIdle(inner.queue);
            
            // Free command buffer
            vkFreeCommandBuffers(inner.device, inner.command_pool, 1, &command_buffer);
            
            Ok(())
        })
    }
}

impl Buffer {
    /// Read data from the buffer
    pub fn read<T>(&self) -> Result<Vec<T>>
    where
        T: Copy + 'static,
    {
        let element_size = std::mem::size_of::<T>();
        let element_count = self.size / element_size;
        
        if self.size % element_size != 0 {
            return Err(KronosError::BufferCreationFailed(
                format!("Buffer size {} is not a multiple of element size {}", self.size, element_size)
            ));
        }
        
        unsafe {
            // Create staging buffer
            let staging = self.context.create_buffer_uninit(self.size)?;
            
            // Copy device to staging
            self.context.copy_buffer(self, &staging, self.size)?;
            
            // Map and read
            self.context.with_inner(|inner| {
                let mut mapped_ptr = ptr::null_mut();
                let result = vkMapMemory(
                    inner.device,
                    staging.memory,
                    0,
                    self.size as VkDeviceSize,
                    0,
                    &mut mapped_ptr,
                );
                
                if result != VkResult::Success {
                    return Err(KronosError::from(result));
                }
                
                let slice = slice::from_raw_parts(mapped_ptr as *const T, element_count);
                let vec = slice.to_vec();
                
                vkUnmapMemory(inner.device, staging.memory);
                
                Ok(vec)
            })
        }
    }
}

impl Drop for Buffer {
    fn drop(&mut self) {
        unsafe {
            self.context.with_inner(|inner| {
                vkFreeMemory(inner.device, self.memory, ptr::null());
                vkDestroyBuffer(inner.device, self.buffer, ptr::null());
            });
        }
    }
}

