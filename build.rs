//! Build script for Kronos Rust bindings

use std::env;
use std::path::PathBuf;

fn main() {
    // Tell cargo to look for shared libraries in the Kronos loader directory
    let kronos_dir = PathBuf::from(env::var("CARGO_MANIFEST_DIR").unwrap())
        .parent()
        .unwrap()
        .join("Kronos")
        .join("loader");
    
    println!("cargo:rustc-link-search={}", kronos_dir.display());
    
    // Link to the Kronos loader library (when available)
    // println!("cargo:rustc-link-lib=kronos_loader");
    
    // For now, we can link to standard Vulkan for testing
    if cfg!(target_os = "linux") {
        println!("cargo:rustc-link-lib=vulkan");
    } else if cfg!(target_os = "windows") {
        println!("cargo:rustc-link-lib=vulkan-1");
    }
    
    // Re-run build if the Kronos headers change
    println!("cargo:rerun-if-changed=../Kronos/core/vulkan_compute_optimized.h");
    println!("cargo:rerun-if-changed=../Kronos/core/vulkan_compute_complete.h");
}