//! Safe synchronization primitives

use super::*;
use crate::*; // Import all functions from the crate root
use std::ptr;

/// A GPU fence for CPU-GPU synchronization
pub struct Fence {
    context: ComputeContext,
    fence: VkFence,
}

// Send + Sync for thread safety
unsafe impl Send for Fence {}
unsafe impl Sync for Fence {}

/// A GPU semaphore for GPU-GPU synchronization
pub struct Semaphore {
    context: ComputeContext,
    semaphore: VkSemaphore,
}

// Send + Sync for thread safety
unsafe impl Send for Semaphore {}
unsafe impl Sync for Semaphore {}

impl ComputeContext {
    /// Create a new fence
    pub fn create_fence(&self, signaled: bool) -> Result<Fence> {
        unsafe {
            self.with_inner(|inner| {
                let create_info = VkFenceCreateInfo {
                    sType: VkStructureType::FenceCreateInfo,
                    pNext: ptr::null(),
                    flags: if signaled { VkFenceCreateFlags::SIGNALED } else { VkFenceCreateFlags::empty() },
                };
                
                let mut fence = VkFence::NULL;
                let result = vkCreateFence(inner.device, &create_info, ptr::null(), &mut fence);
                
                if result != VkResult::Success {
                    return Err(KronosError::SynchronizationError(
                        format!("vkCreateFence failed: {:?}", result)
                    ));
                }
                
                Ok(Fence {
                    context: self.clone(),
                    fence,
                })
            })
        }
    }
    
    /// Create a new semaphore
    pub fn create_semaphore(&self) -> Result<Semaphore> {
        unsafe {
            self.with_inner(|inner| {
                let create_info = VkSemaphoreCreateInfo {
                    sType: VkStructureType::SemaphoreCreateInfo,
                    pNext: ptr::null(),
                    flags: 0,
                };
                
                let mut semaphore = VkSemaphore::NULL;
                let result = vkCreateSemaphore(inner.device, &create_info, ptr::null(), &mut semaphore);
                
                if result != VkResult::Success {
                    return Err(KronosError::SynchronizationError(
                        format!("vkCreateSemaphore failed: {:?}", result)
                    ));
                }
                
                Ok(Semaphore {
                    context: self.clone(),
                    semaphore,
                })
            })
        }
    }
}

impl Fence {
    /// Wait for the fence to be signaled
    pub fn wait(&self, timeout_ns: u64) -> Result<()> {
        unsafe {
            self.context.with_inner(|inner| {
                let result = vkWaitForFences(
                    inner.device,
                    1,
                    &self.fence,
                    VK_TRUE,
                    timeout_ns,
                );
                
                match result {
                    VkResult::Success => Ok(()),
                    VkResult::Timeout => Err(KronosError::SynchronizationError("Timeout waiting for fence".into())),
                    _ => Err(KronosError::from(result)),
                }
            })
        }
    }
    
    /// Wait indefinitely for the fence
    pub fn wait_forever(&self) -> Result<()> {
        self.wait(u64::MAX)
    }
    
    /// Reset the fence to unsignaled state
    pub fn reset(&self) -> Result<()> {
        unsafe {
            self.context.with_inner(|inner| {
                let result = vkResetFences(inner.device, 1, &self.fence);
                
                if result != VkResult::Success {
                    return Err(KronosError::from(result));
                }
                
                Ok(())
            })
        }
    }
    
    /// Check if the fence is signaled without waiting
    pub fn is_signaled(&self) -> Result<bool> {
        unsafe {
            self.context.with_inner(|inner| {
                let result = vkGetFenceStatus(inner.device, self.fence);
                
                match result {
                    VkResult::Success => Ok(true),
                    VkResult::NotReady => Ok(false),
                    _ => Err(KronosError::from(result)),
                }
            })
        }
    }
    
    /// Get the raw Vulkan fence handle
    pub fn raw(&self) -> VkFence {
        self.fence
    }
}

impl Semaphore {
    /// Get the raw Vulkan semaphore handle
    pub fn raw(&self) -> VkSemaphore {
        self.semaphore
    }
}

impl Drop for Fence {
    fn drop(&mut self) {
        unsafe {
            self.context.with_inner(|inner| {
                vkDestroyFence(inner.device, self.fence, ptr::null());
            });
        }
    }
}

impl Drop for Semaphore {
    fn drop(&mut self) {
        unsafe {
            self.context.with_inner(|inner| {
                vkDestroySemaphore(inner.device, self.semaphore, ptr::null());
            });
        }
    }
}