//! Unit tests for implementation modules

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
    fn test_pool_type_methods() {
        // Test PoolType methods which are public
        assert!(PoolType::HostVisibleCoherent.should_map());
        assert!(!PoolType::DeviceLocal.should_map());
        assert!(PoolType::HostVisibleCached.should_map());
        
        let flags = PoolType::DeviceLocal.required_flags();
        assert!(flags.contains(VkMemoryPropertyFlags::DEVICE_LOCAL));
    }
    
    #[test]
    fn test_pool_stats_default() {
        let stats = PoolStats::default();
        assert_eq!(stats.total_allocated, 0);
        assert_eq!(stats.total_slabs, 0);
        assert_eq!(stats.allocations_in_flight, 0);
    }
}

#[cfg(test)]
mod timeline_tests {
    use crate::implementation::timeline_batching::*;
    use crate::sys::*;
    use crate::core::*;
    
    #[test]
    fn test_batch_submission_new() {
        let batch = BatchSubmission::new();
        // Just test that we can create a new batch
        // We can't check private fields
    }
    
    #[test]
    fn test_batch_submission_add_operations() {
        let mut batch = BatchSubmission::new();
        
        // Test that we can add command buffers
        let cb = VkCommandBuffer::from_raw(0x5678);
        batch.add_command_buffer(cb);
        
        // Test that we can add wait operations
        let sem = VkSemaphore::from_raw(0x9ABC);
        batch.add_wait(sem, 42, VkPipelineStageFlags::COMPUTE_SHADER);
        
        // We can't check private fields, but at least verify the methods work
    }
    
    #[test]
    fn test_batch_stats_calculation() {
        let mut stats = BatchStats::default();
        
        stats.record_submission(16);
        assert_eq!(stats.total_submissions, 1);
        assert_eq!(stats.total_command_buffers, 16);
        assert_eq!(stats.average_batch_size, 16.0);
        
        stats.record_submission(32);
        assert_eq!(stats.total_submissions, 2);
        assert_eq!(stats.total_command_buffers, 48);
        assert_eq!(stats.average_batch_size, 24.0);
    }
}

#[cfg(test)]
mod error_tests {
    use crate::implementation::error::IcdError;
    use crate::core::*;
    use crate::VkResult;
    
    #[test]
    fn test_icd_error_variants() {
        // Test that all error variants can be created and displayed
        let errors = vec![
            IcdError::NoManifestsFound,
            IcdError::NoIcdLoaded,
            IcdError::InvalidManifest("test".to_string()),
            IcdError::LibraryLoadFailed("lib.so".to_string()),
            IcdError::MissingFunction("vkCreateDevice"),
            IcdError::VulkanError(VkResult::ErrorOutOfHostMemory),
            IcdError::InvalidPath("bad/path".to_string()),
            IcdError::InvalidOperation("op"),
            IcdError::MutexPoisoned,
        ];
        
        for err in errors {
            // Test Display
            let _ = format!("{}", err);
            
            // Test Debug
            let _ = format!("{:?}", err);
        }
    }
    
    #[test]
    fn test_nul_error_conversion() {
        use std::ffi::NulError;
        
        let nul_err = std::ffi::CString::new("test\0test").unwrap_err();
        let icd_err: IcdError = nul_err.into();
        
        match icd_err {
            IcdError::InvalidString(_) => (),
            _ => panic!("Wrong error conversion"),
        }
    }
}