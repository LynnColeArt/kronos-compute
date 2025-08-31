//! Test safe API pipeline creation

use kronos_compute::api::ComputeContext;

fn main() {
    env_logger::init();
    
    println!("Testing Safe API Pipeline Creation");
    println!("==================================");
    
    // Create context
    let ctx = match ComputeContext::new() {
        Ok(ctx) => {
            println!("✓ ComputeContext created");
            ctx
        }
        Err(e) => {
            println!("✗ Failed to create context: {:?}", e);
            return;
        }
    };
    
    // Create a minimal compute shader
    let shader_code: Vec<u32> = vec![
        // Magic number
        0x07230203,
        // Version 1.0
        0x00010000,
        // Generator ID  
        0x00000000,
        // Bound
        0x00000005,
        // Schema
        0x00000000,
        // OpCapability Shader
        0x00020011, 0x00000001,
        // OpMemoryModel Logical GLSL450
        0x00030016, 0x00000000, 0x00000001,
        // OpEntryPoint GLCompute %main "main"
        0x00040015, 0x00000006, 0x00000004, 0x00000000,
        // OpExecutionMode %main LocalSize 1 1 1
        0x00060010, 0x00000004, 0x00000011, 0x00000001, 0x00000001, 0x00000001,
        // OpName %main "main"
        0x00040005, 0x00000004, 0x00000000,
        // OpTypeVoid
        0x00020013, 0x00000002,
        // OpTypeFunction %void
        0x00030021, 0x00000003, 0x00000002,
        // OpFunction %void %main %func
        0x00050036, 0x00000002, 0x00000004, 0x00000000, 0x00000003,
        // OpLabel
        0x000200F8, 0x00000005,
        // OpReturn
        0x000100FD,
        // OpFunctionEnd
        0x00010038,
    ];
    
    // Convert to bytes
    let shader_bytes: Vec<u8> = shader_code.iter()
        .flat_map(|&word| word.to_le_bytes())
        .collect();
    
    // Create shader
    println!("\nCreating shader module...");
    let shader = match ctx.create_shader_from_spirv(&shader_bytes) {
        Ok(shader) => {
            println!("✓ Shader module created");
            shader
        }
        Err(e) => {
            println!("✗ Failed to create shader: {:?}", e);
            return;
        }
    };
    
    // Create pipeline
    println!("\nCreating compute pipeline...");
    match ctx.create_pipeline(&shader) {
        Ok(_pipeline) => {
            println!("✓ Pipeline created successfully!");
            println!("\nSafe API pipeline creation is now working correctly!");
        }
        Err(e) => {
            println!("✗ Failed to create pipeline: {:?}", e);
        }
    }
}