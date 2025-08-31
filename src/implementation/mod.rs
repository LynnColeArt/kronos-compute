//! Actual implementation of Kronos compute APIs

use std::sync::Mutex;
use log::warn;

pub mod error;
pub mod instance;
pub mod device;
pub mod memory;
pub mod buffer;
pub mod pipeline;
pub mod descriptor;
pub mod sync;
// REMOVED: pub mod icd_loader;
// REMOVED: pub mod forward;
// REMOVED: pub mod persistent_descriptors;  // Uses ICD
// REMOVED: pub mod barrier_policy;         // Uses ICD
// REMOVED: pub mod timeline_batching;      // Uses ICD
// REMOVED: pub mod pool_allocator;         // Uses ICD

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

// Kronos initialization state
lazy_static::lazy_static! {
    pub static ref KRONOS_INITIALIZED: Mutex<bool> = Mutex::new(false);
}

/// Initialize Kronos - pure Rust implementation
pub fn initialize_kronos() -> Result<(), error::KronosError> {
    log::info!("=== Kronos Pure Rust Implementation Initializing ===");
    let mut initialized = KRONOS_INITIALIZED.lock()?;
    if *initialized {
        log::info!("Kronos already initialized");
        return Ok(());
    }
    
    // Initialize our pure Rust implementation
    // No ICD loading, no system Vulkan dependency!
    *initialized = true;
    log::info!("Kronos initialized successfully - pure Rust compute implementation");
    Ok(())
}

