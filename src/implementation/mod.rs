//! Actual implementation of Kronos compute APIs

use std::sync::Mutex;

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
    let mut initialized = ICD_INITIALIZED.lock()?;
    if *initialized {
        return Ok(());
    }
    
    // Try to load ICD, but don't fail if unavailable
    // Functions will return ErrorInitializationFailed when called without ICD
    match icd_loader::initialize_icd_loader() {
        Ok(()) => {
            *initialized = true;
            Ok(())
        }
        Err(e) => {
            // Log the error but don't fail initialization
            eprintln!("Warning: Failed to load Vulkan ICD: {}", e);
            Ok(())
        }
    }
}

