//! Test setting ICD preference before ANY initialization

use kronos_compute::*;

fn main() {
    env_logger::init();
    
    println!("Test: Setting ICD preference before initialization");
    println!("=================================================");
    
    // Set preference FIRST, before any Kronos calls
    // Note: During init, AMD is at index 2 (software ICDs are filtered out)
    println!("Setting AMD preference (index 2 during init)...");
    kronos_compute::implementation::icd_loader::set_preferred_icd_index(2);
    
    // Now initialize
    println!("Initializing Kronos...");
    match initialize_kronos() {
        Ok(()) => println!("✓ Initialized"),
        Err(e) => {
            println!("✗ Failed: {:?}", e);
            return;
        }
    }
    
    // Check which ICD was selected
    if let Some(info) = kronos_compute::implementation::icd_loader::selected_icd_info() {
        println!("✓ Selected ICD: {}", info.library_path.display());
    } else {
        println!("✗ No ICD selected");
    }
    
    // Now test the safe API
    println!("\nCreating ComputeContext...");
    match kronos_compute::api::ComputeContext::new() {
        Ok(_) => println!("✓ ComputeContext created successfully!"),
        Err(e) => println!("✗ Failed: {:?}", e),
    }
}