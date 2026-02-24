//! Actual implementation of Kronos compute APIs

use std::sync::Mutex;
use log::error;

pub mod error;
pub mod instance;
pub mod device;
pub mod memory;
pub mod buffer;
pub mod pipeline;
pub mod descriptor;
pub mod sync;
pub mod icd_loader;
pub mod forward;
pub mod persistent_descriptors;
pub mod barrier_policy;
pub mod timeline_batching;
pub mod pool_allocator;

#[cfg(test)]
mod tests;

// Re-export all implementation functions
pub use instance::*;
pub use device::*;
pub use memory::*;
pub use buffer::*;
pub use pipeline::*;
pub use descriptor::*;
pub use sync::*;

// ICD initialization state
lazy_static::lazy_static! {
    pub static ref ICD_INITIALIZED: Mutex<bool> = Mutex::new(false);
}

/// Initialize Kronos (loads ICD if available)
pub fn initialize_kronos() -> Result<(), error::KronosError> {
    log::info!("=== Kronos Implementation Initializing ===");
    let mut initialized = ICD_INITIALIZED.lock()?;
    if *initialized {
        log::info!("Kronos already initialized");
        return Ok(());
    }
    
    // Try to load ICD. Fail fast because a missing ICD means no real GPU runtime.
    match icd_loader::initialize_icd_loader() {
        Ok(()) => {
            *initialized = true;
            log::info!("Kronos initialized successfully with ICD forwarding");
            Ok(())
        }
        Err(e) => {
            error!("Failed to initialize Vulkan ICD loader: {}", e);
            Err(error::KronosError::from(e))
        }
    }
}
