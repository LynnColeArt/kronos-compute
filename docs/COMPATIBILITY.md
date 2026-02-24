> [!CAUTION] Documentation credibility note
> Quantified performance and benchmark claims in this repository history are in recovery and should not be treated as current production facts until revalidated under the Kronos-first flow.


# Kronos Compatibility Guide

## Overview

Kronos is a compute-only Vulkan path intended to be a close replacement for compute workloads while targeting enhanced performance through reduced API surface area.

## Vulkan Compatibility

### Supported Vulkan Version
- **Base**: Vulkan 1.0 compute features
- **Extensions**: Core compute extensions only
- **Forward Compatibility**: Designed to support future Vulkan compute features

### Supported Operations

#### ✅ Core Compute Features
- **Instance Management**: Instance creation, destruction, and enumeration
- **Device Management**: Physical device queries, logical device creation
- **Memory Management**: Allocation, mapping, binding with all memory types
- **Buffer Operations**: Creation, destruction, memory requirements, binding
- **Command Buffers**: Allocation, recording, submission, synchronization
- **Compute Pipelines**: Shader modules, pipeline layouts, compute pipeline creation
- **Descriptor Sets**: Layouts, pools, allocation, updates, binding
- **Synchronization**: Fences, semaphores, events, pipeline barriers
- **Dispatching**: Direct and indirect compute dispatch

#### ❌ Excluded Graphics Features
- **Rendering**: No graphics pipelines, render passes, or framebuffers
- **Presentation**: No surfaces, swapchains, or presentation queues
- **Images**: No image creation, views, or sampling (compute-focused)
- **Graphics Shaders**: Only compute shaders supported
- **Graphics Commands**: No draw commands or graphics-specific operations

### API Guarantees

#### Memory Layout Compatibility
```rust
// Kronos structures are binary-compatible with Vulkan
assert_eq!(size_of::<VkPhysicalDeviceProperties>(), vulkan_size);
assert_eq!(size_of::<VkBufferCreateInfo>(), vulkan_size);
```

#### Function Signature Compatibility
All supported functions maintain exact Vulkan signatures:
```c
// These signatures are identical between Kronos and Vulkan
VkResult vkCreateDevice(VkPhysicalDevice, const VkDeviceCreateInfo*, 
                       const VkAllocationCallbacks*, VkDevice*);
void vkCmdDispatch(VkCommandBuffer, uint32_t, uint32_t, uint32_t);
```

#### Handle Compatibility
- All Vulkan handles are compatible (VkDevice, VkBuffer, etc.)
- Handle values can be passed between Kronos and Vulkan contexts
- No handle translation or conversion required

## Performance Characteristics (Target Behavior)

### Expected Improvements (Staged)
- **Initialization**: [deferred speedup range] target (historical snapshot scope)
- **Dispatch Throughput**: [deferred speedup range] target (historical snapshot scope)
- **Memory Footprint**: Reduced binary size and runtime memory usage, staged revalidation pending
- **Compilation Time**: [deferred speedup range] target with active runtime evidence still pending

### Maintained Guarantees
- **Thread Safety**: Expected thread-safety alignment where supported
- **Memory Ordering**: Expected ordering alignment where validated
- **Error Handling**: Compatible error codes and behavior with compatibility caveats

## Platform Support

### Current Support
- **Linux**: Current support with automatic ICD discovery
- **Windows**: Provisional support with environment variable configuration
- **macOS**: Provisional support with Homebrew and system paths

### ICD Discovery
```bash
# Standard Vulkan environment variables respected
export VK_ICD_FILENAMES="/path/to/driver.json"

# Kronos-specific override
export KRONOS_ICD_SEARCH_PATHS="/custom/path1:/custom/path2"
```

## Testing and Validation

### Test Suite Structure
```
tests/
├── structure_sizes.rs    # Binary layout compatibility
├── sys_tests.rs         # Handle and type system tests  
├── flags_tests.rs       # Bitflag operations and combinations
└── integration/         # Full compute workflow tests
    ├── buffer_ops.rs
    ├── compute_dispatch.rs
    └── synchronization.rs
```

### Running Tests
```bash
# All tests
cargo test --features implementation

# Specific test suites
cargo test --test structure_sizes
cargo test --test sys_tests
cargo test --test flags_tests

# Integration tests
cargo test --test compute_tests
```

### Validation Levels
1. **Unit Tests**: Individual API component testing
2. **Integration Tests**: Full compute workflow validation  
3. **Compatibility Tests**: Binary layout and ABI validation
4. **Performance Tests**: Benchmarks against standard Vulkan

## Migration Guide

### From Standard Vulkan
```rust
// No code changes required for compute workloads
// Simply link against Kronos instead of Vulkan
// cargo add kronos --features implementation

// Existing code continues to work:
let result = unsafe {
    vkCreateDevice(physical_device, &create_info, ptr::null(), &mut device)
};
```

### Unsupported Features
If your code uses excluded features, you'll get:
- **Compile-time errors**: For statically linked code
- **Runtime errors**: `VK_ERROR_FEATURE_NOT_PRESENT` for dynamic calls

### Best Practices
1. **Feature Detection**: Check for compute queue support before migration
2. **Error Handling**: Handle `VK_ERROR_FEATURE_NOT_PRESENT` gracefully
3. **Testing**: Validate compute workloads thoroughly after migration
4. **Performance**: Benchmark to verify expected performance improvements

## Limitations and Known Issues

### Current Limitations
- **Vulkan 1.1+**: Advanced features not yet implemented
- **Video Codecs**: Hardware video acceleration not supported  
- **Ray Tracing**: Compute-based ray tracing only
- **Geometry Processing**: No tessellation or geometry stages

### Future Enhancements
- **Vulkan 1.1+ Compute**: Subgroups, variable pointers
- **Advanced Memory**: Protected memory, device groups
- **Async Operations**: Rust async/await for fence operations
- **Compute Extensions**: Vendor-specific compute optimizations

## Support and Troubleshooting

### Common Issues
1. **ICD Not Found**: Check `VK_ICD_FILENAMES` and search paths
2. **Feature Not Present**: Verify compute-only usage
3. **Performance Regression**: File issue with benchmark data
4. **Compatibility**: Report ABI/API compatibility issues

### Getting Help
- **Issues**: [GitHub Issues](link-to-repo)
- **Discussions**: [GitHub Discussions](link-to-repo)  
- **Documentation**: [Full API Documentation](link-to-docs)
- **Examples**: See `examples/` directory for usage patterns

## Version History

### v0.1.0 (Current)
- Initial release with Vulkan 1.0 compute compatibility
- Cross-platform ICD discovery
- Initial compatibility test suite
- Error handling and safety documentation under staged verification

### Planned Releases
- **v0.2.0**: Vulkan 1.1 compute features
- **v0.3.0**: Advanced synchronization and memory features  
- **v1.0.0**: Full stability and performance optimization targets to be validated after staged re-verification
