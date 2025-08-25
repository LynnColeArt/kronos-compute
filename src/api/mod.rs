//! Unified Safe API for Kronos Compute
//! 
//! This module provides a safe, ergonomic Rust API that wraps the low-level
//! Vulkan-style FFI interface. All Kronos optimizations (persistent descriptors,
//! smart barriers, timeline batching, and pool allocation) work transparently.

use crate::core::*;
use crate::sys::*;
use crate::ffi::VkResult;
use crate::implementation;
use thiserror::Error;

pub mod context;
pub mod buffer;
pub mod pipeline;
pub mod command;
pub mod sync;

pub use context::ComputeContext;
pub use buffer::Buffer;
pub use pipeline::{Pipeline, Shader};
pub use command::CommandBuilder;
pub use sync::{Fence, Semaphore};

/// Result type for the unified API
pub type Result<T> = std::result::Result<T, KronosError>;

/// Unified API errors
#[derive(Error, Debug)]
pub enum KronosError {
    #[error("Initialization failed: {0}")]
    InitializationFailed(String),
    
    #[error("Device not found")]
    DeviceNotFound,
    
    #[error("Shader compilation failed: {0}")]
    ShaderCompilationFailed(String),
    
    #[error("Buffer creation failed: {0}")]
    BufferCreationFailed(String),
    
    #[error("Command execution failed: {0}")]
    CommandExecutionFailed(String),
    
    #[error("Synchronization error: {0}")]
    SynchronizationError(String),
    
    #[error("Vulkan error: {0:?}")]
    VulkanError(VkResult),
    
    #[error("Implementation error: {0}")]
    ImplementationError(#[from] implementation::error::IcdError),
}

impl From<VkResult> for KronosError {
    fn from(result: VkResult) -> Self {
        KronosError::VulkanError(result)
    }
}

/// Configuration for ComputeContext creation
#[derive(Default)]
pub struct ContextConfig {
    /// Application name
    pub app_name: String,
    /// Enable validation layers
    pub enable_validation: bool,
    /// Preferred GPU vendor (AMD, NVIDIA, Intel)
    pub preferred_vendor: Option<String>,
}

/// Builder for ComputeContext
pub struct ContextBuilder {
    config: ContextConfig,
}

impl ContextBuilder {
    pub fn new() -> Self {
        Self {
            config: ContextConfig::default(),
        }
    }
    
    pub fn app_name(mut self, name: impl Into<String>) -> Self {
        self.config.app_name = name.into();
        self
    }
    
    pub fn enable_validation(mut self) -> Self {
        self.config.enable_validation = true;
        self
    }
    
    pub fn prefer_vendor(mut self, vendor: impl Into<String>) -> Self {
        self.config.preferred_vendor = Some(vendor.into());
        self
    }
    
    pub fn build(self) -> Result<ComputeContext> {
        ComputeContext::new_with_config(self.config)
    }
}

/// Entry point for the unified API
/// 
/// Example:
/// ```no_run
/// use kronos_compute::api;
/// 
/// let ctx = api::ComputeContext::builder()
///     .app_name("My Compute App")
///     .enable_validation()
///     .build()?;
/// ```
impl ComputeContext {
    /// Create a new ComputeContext with default settings
    pub fn new() -> Result<Self> {
        Self::builder().build()
    }
    
    /// Create a builder for customized context creation
    pub fn builder() -> ContextBuilder {
        ContextBuilder::new()
    }
}