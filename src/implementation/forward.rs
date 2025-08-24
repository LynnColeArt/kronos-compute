//! Utilities for forwarding calls to real ICD

use crate::implementation::icd_loader;

/// Get the ICD if available
pub fn get_icd_if_enabled() -> Option<&'static icd_loader::LoadedICD> {
    icd_loader::get_icd()
}