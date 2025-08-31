//! Test safe API with AMD driver explicitly

use kronos_compute::api::ComputeContext;

fn main() {
    // Initialize logging
    env_logger::init();
    
    println!("Testing Safe API with AMD Driver");
    println!("================================");
    
    println!("Creating ComputeContext with AMD preference...");
    
    match ComputeContext::builder()
        .app_name("AMD Test")
        .prefer_icd_path("/usr/lib/x86_64-linux-gnu/libvulkan_radeon.so")
        .build() 
    {
        Ok(ctx) => {
            println!("✓ ComputeContext created successfully with AMD driver!");
            
            // Show which ICD was actually selected
            if let Some(info) = ctx.icd_info() {
                println!("  Using ICD: {}", info.library_path.display());
            }
            
            // Show device properties
            let props = ctx.device_properties();
            // Convert device name from i8 array
            let device_name_bytes = &props.deviceName;
            let null_pos = device_name_bytes.iter().position(|&c| c == 0).unwrap_or(device_name_bytes.len());
            let device_name_u8: Vec<u8> = device_name_bytes[..null_pos]
                .iter()
                .map(|&c| c as u8)
                .collect();
            let device_name = std::str::from_utf8(&device_name_u8).unwrap_or("Unknown Device");
            println!("  Device: {} ({:?})", device_name, props.deviceType);
        }
        Err(e) => {
            println!("✗ Failed to create ComputeContext: {:?}", e);
        }
    }
}