//! Minimal test to debug safe API crash

use kronos_compute::api::ComputeContext;

fn main() {
    // Initialize logging
    env_logger::init();
    
    println!("Testing Safe API Crash Debug");
    println!("===========================");
    
    println!("Creating ComputeContext...");
    
    match ComputeContext::new() {
        Ok(_ctx) => {
            println!("✓ ComputeContext created successfully!");
        }
        Err(e) => {
            println!("✗ Failed to create ComputeContext: {:?}", e);
        }
    }
}