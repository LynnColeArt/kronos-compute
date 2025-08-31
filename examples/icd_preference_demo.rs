//! Demonstrates correct ICD preference usage

use kronos_compute::api::ComputeContext;

fn main() {
    env_logger::init();
    
    println!("ICD Preference Demo");
    println!("==================");
    println!();
    
    // Method 1: Aggregated mode (recommended - preferences work dynamically)
    println!("Method 1: Aggregated Mode");
    println!("-------------------------");
    std::env::set_var("KRONOS_AGGREGATE_ICD", "1");
    
    match ComputeContext::builder()
        .app_name("Aggregated Mode Demo")
        .prefer_icd_index(3)  // AMD in available_icds() list
        .build()
    {
        Ok(ctx) => {
            println!("✓ Success with aggregated mode!");
            if let Some(info) = ctx.icd_info() {
                println!("  Using: {}", info.library_path.display());
            }
        }
        Err(e) => println!("✗ Failed: {:?}", e),
    }
    
    println!();
    
    // Method 2: Path preference (also works in aggregated mode)
    println!("Method 2: Path Preference");
    println!("------------------------");
    
    match ComputeContext::builder()
        .app_name("Path Preference Demo")
        .prefer_icd_path("/usr/lib/x86_64-linux-gnu/libvulkan_radeon.so")
        .build()
    {
        Ok(ctx) => {
            println!("✓ Success with path preference!");
            if let Some(info) = ctx.icd_info() {
                println!("  Using: {}", info.library_path.display());
            }
        }
        Err(e) => println!("✗ Failed: {:?}", e),
    }
    
    println!();
    println!("Note: In single-ICD mode, preferences must be set");
    println!("      BEFORE any Kronos initialization occurs.");
    println!("      Aggregated mode is recommended for flexibility.");
}