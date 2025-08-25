//! Simple example using the unified safe API
//! 
//! This demonstrates how much simpler the unified API is compared
//! to the raw Vulkan-style API.

use kronos_compute::api::{self, ComputeContext, PipelineConfig, BufferBinding};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("Kronos Unified API Example");
    println!("=========================");
    
    // Create context - no unsafe code needed!
    let ctx = ComputeContext::builder()
        .app_name("Vector Addition Example")
        .build()?;
    
    println!("✓ Context created");
    
    // Load shader
    let shader = ctx.load_shader("shaders/saxpy.spv")?;
    println!("✓ Shader loaded");
    
    // Create pipeline with configuration
    let pipeline_config = PipelineConfig {
        entry_point: "main".to_string(),
        local_size: (64, 1, 1),
        bindings: vec![
            BufferBinding { binding: 0, ..Default::default() }, // Input A
            BufferBinding { binding: 1, ..Default::default() }, // Input B  
            BufferBinding { binding: 2, ..Default::default() }, // Output C
        ],
        push_constant_size: std::mem::size_of::<f32>() as u32, // For scalar parameter
    };
    
    let pipeline = ctx.create_pipeline_with_config(&shader, pipeline_config)?;
    println!("✓ Pipeline created");
    
    // Create data
    let n = 1024;
    let a_data: Vec<f32> = (0..n).map(|i| i as f32).collect();
    let b_data: Vec<f32> = (0..n).map(|i| (i * 2) as f32).collect();
    
    // Create buffers - automatic memory management!
    let a = ctx.create_buffer(&a_data)?;
    let b = ctx.create_buffer(&b_data)?;
    let c = ctx.create_buffer_uninit(n * std::mem::size_of::<f32>())?;
    
    println!("✓ Buffers created ({} elements each)", n);
    
    // Execute compute shader with fluent API
    let scalar = 2.0f32;
    
    ctx.dispatch(&pipeline)
        .bind_buffer(0, &a)
        .bind_buffer(1, &b)
        .bind_buffer(2, &c)
        .push_constants(&scalar)
        .workgroups(n as u32 / 64, 1, 1)
        .execute()?;
    
    println!("✓ Compute dispatched");
    
    // Read results - safe and easy!
    let results: Vec<f32> = c.read()?;
    
    // Verify results
    let mut correct = 0;
    for i in 0..n {
        let expected = scalar * a_data[i] + b_data[i];
        if (results[i] - expected).abs() < 0.001 {
            correct += 1;
        }
    }
    
    println!("\nResults (first 10):");
    for i in 0..10.min(n) {
        let expected = scalar * a_data[i] + b_data[i];
        println!("  c[{}] = {} (expected {})", i, results[i], expected);
    }
    
    println!("\n✓ Verification: {}/{} correct", correct, n);
    
    if correct == n {
        println!("\n🎉 All results correct! The unified API works!");
    } else {
        println!("\n⚠️  Some results incorrect");
    }
    
    // Everything is automatically cleaned up when dropped
    // No manual destroy calls needed!
    
    Ok(())
}

// Compare this to compute_simple.rs - much cleaner!