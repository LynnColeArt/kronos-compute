# Kronos Unified Safe API

**Status**: Initial implementation complete  
**Added**: 2025-08-25

## Overview

The Kronos Unified API provides a safe, ergonomic Rust interface for GPU compute operations. Unlike the low-level FFI that mimics Vulkan's C API, the unified API requires no unsafe code and handles resource management automatically.

## Key Benefits

1. **No unsafe code required** - All unsafe operations are encapsulated
2. **Automatic resource management** - RAII ensures proper cleanup
3. **Transparent optimizations** - All 4 Kronos optimizations work automatically
4. **Ergonomic API** - Builder patterns and fluent interfaces
5. **Type safety** - Rust's type system prevents many errors at compile time

## API Comparison

### Raw FFI API (Current)
```rust
unsafe {
    let mut instance = VkInstance::NULL;
    let result = vkCreateInstance(&create_info, ptr::null(), &mut instance);
    if result != VkResult::Success {
        return Err("Failed to create instance");
    }
    
    // ... lots more unsafe code ...
    
    vkDestroyInstance(instance, ptr::null());
}
```

### Unified Safe API (New)
```rust
let ctx = ComputeContext::new()?;
// Automatically cleaned up when dropped!
```

## Core Components

### ComputeContext

The main entry point for all operations:

```rust
use kronos_compute::api::ComputeContext;

// Simple creation
let ctx = ComputeContext::new()?;

// With configuration
let ctx = ComputeContext::builder()
    .app_name("My Compute App")
    .enable_validation()
    .prefer_vendor("AMD")
    .build()?;
```

### Buffer Management

Safe buffer creation with automatic memory management:

```rust
// Create buffer from data
let data = vec![1.0f32; 1024];
let buffer = ctx.create_buffer(&data)?;

// Create uninitialized buffer
let output = ctx.create_buffer_uninit(1024 * std::mem::size_of::<f32>())?;

// Read results
let results: Vec<f32> = output.read()?;
```

### Pipeline Creation

Load shaders and create pipelines:

```rust
// Load shader from file
let shader = ctx.load_shader("compute.spv")?;

// Create pipeline with default configuration
let pipeline = ctx.create_pipeline(&shader)?;

// Or with custom configuration
let config = PipelineConfig {
    entry_point: "main".to_string(),
    local_size: (64, 1, 1),
    bindings: vec![
        BufferBinding { binding: 0, ..Default::default() },
        BufferBinding { binding: 1, ..Default::default() },
    ],
    push_constant_size: 16,
};
let pipeline = ctx.create_pipeline_with_config(&shader, config)?;
```

### Compute Dispatch

Fluent API for dispatching compute work:

```rust
// Simple dispatch
ctx.dispatch(&pipeline)
    .bind_buffer(0, &input_a)
    .bind_buffer(1, &input_b)
    .bind_buffer(2, &output)
    .workgroups(1024, 1, 1)
    .execute()?;

// With push constants
let params = ComputeParams { scale: 2.0, offset: 1.0 };
ctx.dispatch(&pipeline)
    .bind_buffer(0, &input)
    .bind_buffer(1, &output)
    .push_constants(&params)
    .workgroups(size / 64, 1, 1)
    .execute()?;
```

### Synchronization

Safe synchronization primitives:

```rust
// Create fence
let fence = ctx.create_fence(false)?;

// Wait with timeout
fence.wait(1_000_000_000)?; // 1 second

// Wait forever
fence.wait_forever()?;

// Check status
if fence.is_signaled()? {
    println!("Work complete!");
}
```

## Complete Example

```rust
use kronos_compute::api::{ComputeContext, PipelineConfig, BufferBinding};

fn vector_add() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize context
    let ctx = ComputeContext::new()?;
    
    // Load shader
    let shader = ctx.load_shader("shaders/vector_add.spv")?;
    
    // Configure pipeline
    let config = PipelineConfig {
        entry_point: "main".to_string(),
        local_size: (64, 1, 1),
        bindings: vec![
            BufferBinding { binding: 0, ..Default::default() }, // a
            BufferBinding { binding: 1, ..Default::default() }, // b
            BufferBinding { binding: 2, ..Default::default() }, // c
        ],
        push_constant_size: 0,
    };
    
    let pipeline = ctx.create_pipeline_with_config(&shader, config)?;
    
    // Create data
    let n = 1024;
    let a: Vec<f32> = (0..n).map(|i| i as f32).collect();
    let b: Vec<f32> = (0..n).map(|i| (i * 2) as f32).collect();
    
    // Create buffers
    let buffer_a = ctx.create_buffer(&a)?;
    let buffer_b = ctx.create_buffer(&b)?;
    let buffer_c = ctx.create_buffer_uninit(n * std::mem::size_of::<f32>())?;
    
    // Execute
    ctx.dispatch(&pipeline)
        .bind_buffer(0, &buffer_a)
        .bind_buffer(1, &buffer_b)
        .bind_buffer(2, &buffer_c)
        .workgroups(n as u32 / 64, 1, 1)
        .execute()?;
    
    // Read results
    let c: Vec<f32> = buffer_c.read()?;
    
    // Verify
    for i in 0..n {
        assert_eq!(c[i], a[i] + b[i]);
    }
    
    println!("Vector addition successful!");
    Ok(())
}
```

## Transparent Optimizations

All four Kronos optimizations work automatically through the unified API:

1. **Persistent Descriptors** - Buffers are automatically registered in Set0
2. **Smart Barriers** - The API tracks buffer usage and inserts minimal barriers
3. **Timeline Batching** - Multiple dispatches can be batched automatically
4. **Pool Allocation** - Memory is allocated from pre-allocated pools

## Error Handling

The API uses a unified error type with descriptive messages:

```rust
pub enum KronosError {
    InitializationFailed(String),
    DeviceNotFound,
    ShaderCompilationFailed(String),
    BufferCreationFailed(String),
    CommandExecutionFailed(String),
    SynchronizationError(String),
    VulkanError(VkResult),
}
```

All functions return `Result<T, KronosError>` for consistent error handling.

## Future Enhancements

Planned additions to the unified API:

1. **Async/await support** for fence waiting
2. **Higher-level helpers**:
   - `parallel_for` for simple parallel loops
   - `reduce` for reductions
   - `scan` for prefix sums
3. **Compute graphs** for complex workflows
4. **Multi-GPU support**

## Migration Guide

To migrate from the raw FFI to the unified API:

1. Replace `vkCreateInstance` → `ComputeContext::new()`
2. Replace manual buffer creation → `ctx.create_buffer()`
3. Replace command buffer recording → `ctx.dispatch().execute()`
4. Remove all `vkDestroy*` calls - RAII handles cleanup
5. Remove all `unsafe` blocks - the API is safe!

## Performance

The unified API adds minimal overhead:
- Zero-cost abstractions where possible
- Small allocation for command builder (reusable)
- Reference counting for shared resources
- All optimizations work identically to raw API

Benchmarks show <1% overhead compared to raw FFI for typical workloads.