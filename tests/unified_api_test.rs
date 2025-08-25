//! Comprehensive tests for the unified API

#[cfg(feature = "implementation")]
mod tests {
    use kronos_compute::api::{ComputeContext, PipelineConfig, BufferBinding, BufferUsage};
    
    #[test]
    fn test_api_types_exist() {
        // Just verify that all the types we expose exist and can be constructed
        
        // BufferUsage
        let _usage = BufferUsage::storage();
        let _combined = BufferUsage::storage() | BufferUsage::transfer_src();
        
        // PipelineConfig
        let _config = PipelineConfig {
            entry_point: "main".to_string(),
            local_size: (64, 1, 1),
            bindings: vec![
                BufferBinding::default(),
            ],
            push_constant_size: 16,
        };
        
        // Make sure we can at least try to create a context
        // (might fail due to no GPU, but the API should work)
        let _result = ComputeContext::builder()
            .app_name("Test App")
            .build();
    }
    
    #[test]
    fn test_builder_pattern() {
        let _builder = ComputeContext::builder()
            .app_name("My App")
            .enable_validation()
            .prefer_vendor("AMD");
        
        // Builder should be usable
    }
    
    #[test]
    fn test_error_types() {
        use kronos_compute::api::KronosError;
        
        // Make sure error types exist and can be constructed
        let _err1 = KronosError::InitializationFailed("test".to_string());
        let _err2 = KronosError::DeviceNotFound;
        let _err3 = KronosError::ShaderCompilationFailed("test".to_string());
        let _err4 = KronosError::BufferCreationFailed("test".to_string());
        let _err5 = KronosError::CommandExecutionFailed("test".to_string());
        let _err6 = KronosError::SynchronizationError("test".to_string());
    }
    
    #[test]
    fn test_api_surface() {
        // Verify the main API surface is accessible
        
        // Types should be accessible from api module
        let _ = std::any::type_name::<kronos_compute::api::ComputeContext>();
        let _ = std::any::type_name::<kronos_compute::api::Buffer>();
        let _ = std::any::type_name::<kronos_compute::api::Pipeline>();
        let _ = std::any::type_name::<kronos_compute::api::Shader>();
        let _ = std::any::type_name::<kronos_compute::api::CommandBuilder>();
        let _ = std::any::type_name::<kronos_compute::api::Fence>();
        let _ = std::any::type_name::<kronos_compute::api::Semaphore>();
    }
}