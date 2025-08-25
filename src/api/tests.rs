//! Tests for the unified safe API

#[cfg(test)]
mod tests {
    use super::super::*;
    use crate::core::*;
    use crate::sys::*;
    
    #[test]
    fn test_buffer_usage_creation() {
        // Test that we can create buffer usage flags
        let storage = BufferUsage::storage();
        let transfer_src = BufferUsage::transfer_src();
        let transfer_dst = BufferUsage::transfer_dst();
        
        // Test that we can combine them
        let _combined = storage | transfer_src;
    }
    
    #[test]
    fn test_context_config() {
        let config = ContextConfig {
            app_name: "Test App".to_string(),
            enable_validation: true,
            preferred_vendor: None,
        };
        
        assert_eq!(config.app_name, "Test App");
        assert!(config.enable_validation);
    }
    
    #[test]
    fn test_builder_pattern() {
        // Test that the builder pattern works correctly
        let builder = ComputeContext::builder()
            .app_name("Test App")
            .enable_validation();
        
        assert_eq!(builder.config.app_name, "Test App");
        assert!(builder.config.enable_validation);
    }
    
    #[test]
    fn test_error_conversions() {
        // Test VkResult to KronosError conversion
        let vk_error = VkResult::ErrorOutOfHostMemory;
        let kronos_error = KronosError::from(vk_error);
        
        match kronos_error {
            KronosError::VulkanError(e) => assert_eq!(e, VkResult::ErrorOutOfHostMemory),
            _ => panic!("Wrong error type"),
        }
        
        // Test IcdError to KronosError conversion
        use crate::implementation::error::IcdError;
        let icd_error = IcdError::NoIcdLoaded;
        let kronos_error = KronosError::from(icd_error);
        
        match kronos_error {
            KronosError::ImplementationError(_) => (),
            _ => panic!("Wrong error type"),
        }
    }
    
    #[test]
    fn test_context_builder_chain() {
        let builder = ComputeContext::builder()
            .app_name("MyApp")
            .enable_validation()
            .prefer_vendor("AMD");
        
        assert_eq!(builder.config.app_name, "MyApp");
        assert!(builder.config.enable_validation);
        assert_eq!(builder.config.preferred_vendor, Some("AMD".to_string()));
    }
}