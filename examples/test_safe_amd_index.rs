//! Test safe API with AMD driver by index

use kronos_compute::api::ComputeContext;

fn main() {
    // Initialize logging
    env_logger::init();
    
    println!("Testing Safe API with AMD (Index 0)");
    println!("==================================");
    
    println!("Creating ComputeContext with AMD preference by index...");
    
    match ComputeContext::builder()
        .app_name("AMD Index Test")
        .prefer_icd_index(3)  // AMD is at index 3
        .build() 
    {
        Ok(_ctx) => {
            println!("✓ ComputeContext created successfully!");
        }
        Err(e) => {
            println!("✗ Failed to create ComputeContext: {:?}", e);
        }
    }
}