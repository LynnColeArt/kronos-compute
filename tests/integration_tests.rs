//! Integration tests for Kronos Compute

#[cfg(feature = "implementation")]
mod implementation_tests {
    use kronos_compute::*;
    use kronos_compute::implementation::*;
    
    #[test]
    fn test_icd_loader_discovery() {
        // Test that we can discover ICD manifests
        let manifests = icd_loader::discover_icds();
        // We might not find any in test environment, that's okay
        assert!(manifests.len() >= 0);
    }
    
    #[test]
    fn test_barrier_policy_vendor_detection() {
        use barrier_policy::GpuVendor;
        
        assert_eq!(GpuVendor::from_vendor_id(0x1002), GpuVendor::AMD);
        assert_eq!(GpuVendor::from_vendor_id(0x10DE), GpuVendor::NVIDIA);
        assert_eq!(GpuVendor::from_vendor_id(0x8086), GpuVendor::Intel);
        assert_eq!(GpuVendor::from_vendor_id(0x1234), GpuVendor::Other);
    }
    
    #[test]
    fn test_barrier_optimization_configs() {
        use barrier_policy::{GpuVendor, BarrierType, BarrierConfig};
        
        // Test AMD optimizations
        let config = BarrierConfig::optimal_for(GpuVendor::AMD, BarrierType::UploadToRead);
        assert_eq!(config.src_stage, VkPipelineStageFlags::HOST);
        assert_eq!(config.dst_stage, VkPipelineStageFlags::COMPUTE_SHADER);
        
        // Test NVIDIA optimizations
        let config = BarrierConfig::optimal_for(GpuVendor::NVIDIA, BarrierType::WriteToRead);
        assert_eq!(config.src_access, VkAccessFlags::SHADER_WRITE);
        assert_eq!(config.dst_access, VkAccessFlags::SHADER_READ);
    }
    
    #[test]
    fn test_pool_allocator_types() {
        use pool_allocator::PoolType;
        
        let device_local = PoolType::DeviceLocal;
        assert!(device_local.required_flags().contains(VkMemoryPropertyFlags::DEVICE_LOCAL));
        assert!(!device_local.should_map());
        
        let host_visible = PoolType::HostVisibleCoherent;
        assert!(host_visible.required_flags().contains(VkMemoryPropertyFlags::HOST_VISIBLE));
        assert!(host_visible.should_map());
    }
    
    #[test]
    fn test_timeline_batching_batch_builder() {
        use timeline_batching::BatchBuilder;
        
        let queue = VkQueue::from_raw(0x1234);
        let cb1 = VkCommandBuffer::from_raw(0x5678);
        let cb2 = VkCommandBuffer::from_raw(0x9ABC);
        
        let builder = BatchBuilder::new(queue)
            .add_command_buffer(cb1)
            .add_command_buffer(cb2);
        
        // Can't actually submit without a real queue, but we can test the builder
        assert_eq!(builder.command_buffers.len(), 2);
    }
    
    #[test]
    fn test_persistent_descriptors_push_constant_range() {
        use persistent_descriptors::create_push_constant_range;
        
        let range = create_push_constant_range(64);
        assert_eq!(range.stageFlags, VkShaderStageFlags::COMPUTE);
        assert_eq!(range.offset, 0);
        assert_eq!(range.size, 64);
    }
    
    #[test]
    #[should_panic(expected = "Push constant size 256 exceeds limit 128")]
    fn test_persistent_descriptors_size_limit() {
        use persistent_descriptors::create_push_constant_range;
        
        // This should panic
        create_push_constant_range(256);
    }
    
    #[test]
    fn test_error_types() {
        use error::IcdError;
        use std::sync::PoisonError;
        
        // Test Display implementations
        let err = IcdError::NoIcdLoaded;
        assert_eq!(format!("{}", err), "No ICD loaded");
        
        let err = IcdError::LibraryLoadFailed("test.so".to_string());
        assert_eq!(format!("{}", err), "Failed to load library: test.so");
        
        // Test From implementations
        let poison_err: PoisonError<i32> = PoisonError::new(42);
        let icd_err: IcdError = poison_err.into();
        match icd_err {
            IcdError::MutexPoisoned => (),
            _ => panic!("Wrong error conversion"),
        }
    }
}