//! Test ICD loading directly

use kronos_compute::*;

fn main() {
    env_logger::init();
    
    println!("Testing ICD Loading");
    println!("==================");
    
    // Initialize Kronos
    match initialize_kronos() {
        Ok(()) => println!("✓ Kronos initialized"),
        Err(e) => {
            println!("✗ Failed to initialize: {:?}", e);
            return;
        }
    }
    
    // Check if ICD is loaded
    if let Some(info) = kronos_compute::implementation::icd_loader::selected_icd_info() {
        println!("✓ ICD loaded: {}", info.library_path.display());
        println!("  Is software: {}", info.is_software);
        println!("  API version: 0x{:x}", info.api_version);
    } else {
        println!("✗ No ICD loaded!");
    }
}