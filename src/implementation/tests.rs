//! Unit tests for implementation modules

// Tests for removed modules commented out since we removed ICD forwarding
/*
#[cfg(test)]
mod barrier_tests {
    use crate::implementation::barrier_policy::*;
    use crate::sys::*;
    use crate::core::*;
    
    #[test]
    fn test_barrier_batch_creation() {
        let _batch = BarrierBatch::new(GpuVendor::AMD);
        // Can't check private fields, but at least verify creation works
    }
    
    #[test]
    fn test_barrier_tracker_new() {
        let tracker = BarrierTracker::new(GpuVendor::NVIDIA);
        assert_eq!(tracker.stats().total_barriers, 0);
        assert_eq!(tracker.stats().elided_barriers, 0);
    }
    
    #[test]
    fn test_barrier_tracker_state_tracking() {
        let mut tracker = BarrierTracker::new(GpuVendor::Intel);
        
        // First access should create a barrier
        let buffer = VkBuffer::from_raw(0x1234);
        let needs_barrier = tracker.track_buffer_access(
            buffer,
            VkAccessFlags::SHADER_READ,
            0,
            1024
        );
        assert!(needs_barrier); // First access always needs barrier
        
        // Same access shouldn't need barrier
        let needs_barrier = tracker.track_buffer_access(
            buffer,
            VkAccessFlags::SHADER_READ,
            0,
            1024
        );
        assert!(!needs_barrier);
        assert_eq!(tracker.stats().elided_barriers, 1);
    }
}

#[cfg(test)]
mod pool_tests {
    use crate::implementation::pool_allocator::*;
    use crate::sys::*;
    use crate::core::*;
    
    #[test]
    fn test_memory_pool_creation() {
        let pool = MemoryPool::new(1024 * 1024, 0); // 1MB pool
        assert_eq!(pool.total_size(), 1024 * 1024);
        assert_eq!(pool.used_size(), 0);
        assert_eq!(pool.free_size(), 1024 * 1024);
    }
    
    #[test]
    fn test_pool_allocator_creation() {
        let allocator = PoolAllocator::new();
        assert_eq!(allocator.stats().total_allocations, 0);
    }
    
    #[test]
    fn test_descriptor_pool_creation() {
        let pool = PersistentDescriptorPool::new(1000);
        assert_eq!(pool.stats().sets_allocated, 0);
        assert_eq!(pool.stats().sets_freed, 0);
        assert_eq!(pool.stats().pools_created, 0);
    }
}

#[cfg(test)]
mod timeline_tests {
    use crate::implementation::timeline_batching::*;
    use crate::sys::*;
    use crate::core::*;
    
    #[test]
    fn test_batch_semaphore_pool() {
        let pool = BatchSemaphorePool::new();
        assert_eq!(pool.stats().semaphores_created, 0);
        assert_eq!(pool.stats().semaphores_recycled, 0);
    }
    
    #[test]
    fn test_timeline_batcher_creation() {
        let batcher = TimelineBatcher::new(1000);
        assert_eq!(batcher.stats().total_batches, 0);
        assert_eq!(batcher.stats().total_submissions, 0);
    }
    
    #[test]
    fn test_coherent_buffer_manager() {
        let manager = CoherentBufferManager::new();
        assert_eq!(manager.stats().total_buffers, 0);
        assert_eq!(manager.stats().coherent_buffers, 0);
    }
}

#[cfg(test)]
mod persistent_descriptor_tests {
    use crate::implementation::persistent_descriptors::*;
    use crate::sys::*;
    use crate::core::*;
    
    #[test]
    fn test_push_constant_validation() {
        // Basic size check
        assert!(MAX_PUSH_CONSTANT_SIZE <= 128);
        
        // Set0 reserved for persistent descriptors
        assert_eq!(PERSISTENT_DESCRIPTOR_SET, 0);
    }
}
*/

#[cfg(test)]
mod error_tests {
    use crate::implementation::error::*;
    use crate::core::*;
    
    #[test]
    fn test_kronos_error_conversion() {
        let lock_error = std::sync::PoisonError::new(());
        let kronos_error: KronosError = KronosError::from(lock_error);
        match kronos_error {
            KronosError::LockPoisoned => (),
            _ => panic!("Wrong error type"),
        }
    }
    
    #[test]
    fn test_vulkan_result_conversion() {
        let error = KronosError::NotReady;
        let vk_result: VkResult = error.into();
        assert_eq!(vk_result, VkResult::NotReady);
    }
    
    #[test]
    fn test_icd_error_creation() {
        let error = IcdError::VulkanError(VkResult::ErrorDeviceLost);
        match error {
            IcdError::VulkanError(result) => assert_eq!(result, VkResult::ErrorDeviceLost),
            _ => panic!("Wrong error type"),
        }
    }
    
    #[test]
    fn test_error_display() {
        use std::ffi::NulError;
        let nul_error = unsafe { std::ffi::CString::from_vec_unchecked(vec![b'a', 0, b'b']) };
        let error = KronosError::from(nul_error.into_bytes().into_iter().collect::<Vec<_>>());
        // Just ensure Display trait works
        let _ = format!("{}", error);
    }
}