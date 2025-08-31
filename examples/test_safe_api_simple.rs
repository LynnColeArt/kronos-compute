//! Test safe API without any ICD preferences

use kronos_compute::api::ComputeContext;

fn main() {
    // Initialize logging
    env_logger::init();
    
    println!("Testing Safe API (Simple)");
    println!("========================");
    
    println!("Creating ComputeContext with default settings...");
    
    match ComputeContext::builder()
        .app_name("Simple Test")
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