//! Utilities for forwarding calls to real ICD

use crate::implementation::{BackendMode, BACKEND_MODE};
use crate::implementation::icd_loader;

/// Check if we should use the real ICD
pub fn should_forward_to_icd() -> bool {
    let mode = BACKEND_MODE.lock().unwrap();
    matches!(*mode, BackendMode::RealICD)
}

/// Get the ICD if we're in RealICD mode
pub fn get_icd_if_enabled() -> Option<&'static icd_loader::LoadedICD> {
    if should_forward_to_icd() {
        icd_loader::get_icd()
    } else {
        None
    }
}