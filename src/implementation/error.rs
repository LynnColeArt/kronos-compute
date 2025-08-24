//! Error types for Kronos implementation

use std::fmt;

/// Errors that can occur in the ICD loader
#[derive(Debug)]
pub enum IcdError {
    /// Failed to create CString (contains null byte)
    InvalidString(std::ffi::NulError),
    /// Failed to load dynamic library
    LibraryLoadFailed(String),
    /// Required function not found in library
    MissingFunction(&'static str),
    /// Failed to parse ICD manifest
    InvalidManifest(String),
    /// No ICD manifest files found
    NoManifestsFound,
    /// Mutex was poisoned
    MutexPoisoned,
    /// Path has no parent directory
    InvalidPath(String),
}

impl fmt::Display for IcdError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            IcdError::InvalidString(e) => write!(f, "Invalid string: {}", e),
            IcdError::LibraryLoadFailed(path) => write!(f, "Failed to load library: {}", path),
            IcdError::MissingFunction(name) => write!(f, "Missing function: {}", name),
            IcdError::InvalidManifest(msg) => write!(f, "Invalid manifest: {}", msg),
            IcdError::NoManifestsFound => write!(f, "No ICD manifest files found"),
            IcdError::MutexPoisoned => write!(f, "Mutex was poisoned"),
            IcdError::InvalidPath(path) => write!(f, "Invalid path: {}", path),
        }
    }
}

impl std::error::Error for IcdError {}

impl From<std::ffi::NulError> for IcdError {
    fn from(e: std::ffi::NulError) -> Self {
        IcdError::InvalidString(e)
    }
}

/// General Kronos errors
#[derive(Debug)]
pub enum KronosError {
    /// ICD loader error
    IcdError(IcdError),
    /// Mutex was poisoned
    MutexPoisoned,
}

impl fmt::Display for KronosError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            KronosError::IcdError(e) => write!(f, "ICD error: {}", e),
            KronosError::MutexPoisoned => write!(f, "Mutex was poisoned"),
        }
    }
}

impl std::error::Error for KronosError {}

impl From<IcdError> for KronosError {
    fn from(e: IcdError) -> Self {
        KronosError::IcdError(e)
    }
}

// Helper for mutex lock errors
impl<T> From<std::sync::PoisonError<T>> for IcdError {
    fn from(_: std::sync::PoisonError<T>) -> Self {
        IcdError::MutexPoisoned
    }
}

impl<T> From<std::sync::PoisonError<T>> for KronosError {
    fn from(_: std::sync::PoisonError<T>) -> Self {
        KronosError::MutexPoisoned
    }
}