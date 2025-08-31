//! Kronos - A compute-only Vulkan implementation in Rust
//! 
//! This crate provides a streamlined, compute-focused subset of the Vulkan API,
//! removing all graphics functionality to achieve maximum GPU compute performance.
//!
//! # Important: Linking Considerations
//! 
//! When using Kronos with the `implementation` feature, do NOT link to system Vulkan.
//! Kronos provides its own Vulkan implementation. Linking to both will cause symbol
//! conflicts where system Vulkan functions may be called instead of Kronos functions,
//! breaking multi-GPU support.
//!
//! If you see `ErrorInitializationFailed` but don't see "KRONOS vkCreateBuffer called"
//! in logs, your application is likely using system Vulkan instead of Kronos.

#![allow(non_camel_case_types)]
#![allow(non_snake_case)]

pub mod core;
pub mod sys;
pub mod ffi;

// Unified safe API
pub mod api;

#[cfg(feature = "implementation")]
pub mod implementation;

// Re-export commonly used items
pub use core::*;
pub use sys::*;
pub use ffi::*;

// When implementation feature is enabled, export all implementation functions
// This MUST come after other exports to ensure our functions take precedence
#[cfg(feature = "implementation")]
pub use implementation::{initialize_kronos};

#[cfg(feature = "implementation")]
pub use implementation::*;

// Explicitly re-export key functions to ensure they're available
#[cfg(feature = "implementation")]
pub use implementation::{
    vkCreateBuffer, vkDestroyBuffer, vkAllocateMemory, vkFreeMemory,
    vkCreateDevice, vkDestroyDevice, vkCreateInstance, vkDestroyInstance,
};

// For libc types
extern crate libc;

/// Version information
pub const KRONOS_VERSION_MAJOR: u32 = 0;
pub const KRONOS_VERSION_MINOR: u32 = 2;
pub const KRONOS_VERSION_PATCH: u32 = 3;

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
        assert_eq!(KRONOS_API_VERSION, make_version(0, 2, 3));
    }
}