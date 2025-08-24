//! Kronos - A compute-only Vulkan implementation in Rust
//! 
//! This crate provides a streamlined, compute-focused subset of the Vulkan API,
//! removing all graphics functionality to achieve maximum GPU compute performance.

#![allow(non_camel_case_types)]
#![allow(non_snake_case)]

pub mod core;
pub mod sys;
pub mod ffi;

#[cfg(feature = "implementation")]
pub mod implementation;

// Re-export commonly used items
pub use core::*;
pub use sys::*;
pub use ffi::*;

#[cfg(feature = "implementation")]
pub use implementation::{BackendMode, set_backend_mode, get_backend_mode, initialize_kronos};

// For libc types
extern crate libc;

/// Version information
pub const KRONOS_VERSION_MAJOR: u32 = 0;
pub const KRONOS_VERSION_MINOR: u32 = 1;
pub const KRONOS_VERSION_PATCH: u32 = 0;

/// Make version number from major, minor, and patch numbers
#[inline]
pub const fn make_version(major: u32, minor: u32, patch: u32) -> u32 {
    (major << 22) | (minor << 12) | patch
}

/// Kronos API version
pub const KRONOS_API_VERSION: u32 = make_version(
    KRONOS_VERSION_MAJOR,
    KRONOS_VERSION_MINOR,
    KRONOS_VERSION_PATCH
);

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_version() {
        assert_eq!(KRONOS_API_VERSION, make_version(0, 1, 0));
    }
}