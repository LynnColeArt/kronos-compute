//! Utilities for forwarding calls to real ICD

use crate::implementation::icd_loader;
use std::sync::Arc;

/// Get the ICD if available
pub fn get_icd_if_enabled() -> Option<Arc<icd_loader::LoadedICD>> {
    icd_loader::get_icd()
}
