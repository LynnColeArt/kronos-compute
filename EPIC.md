# Kronos: Compute-Only Vulkan Fork

## Epic Overview

Kronos is a streamlined, compute-focused fork of Vulkan that removes all graphics functionality to achieve maximum GPU compute performance. This is a full Rust port (not just FFI bindings) that forwards compute calls to real Vulkan drivers via the ICD (Installable Client Driver) mechanism.

## Project Goals

1. **Remove all graphics-only functionality** - Strip out rendering, surfaces, swapchains, etc.
2. **Maintain compute compatibility** - Ensure all compute operations work identically to Vulkan
3. **Zero-copy performance** - Direct forwarding to real drivers, no intermediate translation
4. **Production-ready** - Robust error handling, no panics, proper logging
5. **Cross-platform** - Support Linux, Windows, and macOS

## Architecture

```
Application
    ↓
Kronos API (Rust)
    ↓
ICD Loader
    ↓
Real Vulkan Driver (nvidia, amd, intel, etc.)
```

## Current Status

### ✅ Completed
- Full Rust port of core Vulkan compute APIs
- ICD loader implementation with driver discovery
- Function forwarding to real Vulkan drivers
- Removal of all graphics-only code
- Thread-safe implementations
- Basic test suite

### 🚧 In Progress
- Production hardening (removing unwraps, adding safety docs)
- Cross-platform support
- Comprehensive documentation

### 📋 Pending
See [TODO.md](TODO.md) for detailed task list.

## Key Decisions

1. **No Mock Implementation in Production**
   - All functions either forward to real ICD or return errors
   - No fallback behavior that could hide failures

2. **Compute-Only Design**
   - Removed all graphics enums, structures, and functions
   - Kept only compute-necessary features

3. **Full Rust Port**
   - Not just FFI bindings
   - Rust-native types with zero-copy forwarding

## Testing

```bash
# Run all tests
cargo test --features implementation

# Run specific test suites
cargo test --test structure_sizes
cargo test --test compute_tests

# Run examples
cargo run --example compute_simple --features implementation
cargo run --example test_forwarding --features implementation
```

## Building

```bash
# Standard build
cargo build --release --features implementation

# With validation
cargo build --release --features "implementation validation"
```

## Contributing

1. All unsafe code must have safety documentation
2. No unwrap() in production code paths
3. Use proper error types (not String)
4. Add tests for new functionality
5. Keep graphics code out!

## Performance

Kronos adds minimal overhead:
- Zero-copy command buffer recording
- Direct function pointer forwarding
- No intermediate translation layers
- Native Rust performance

## Compatibility

Kronos implements a subset of Vulkan 1.0 compute functionality:
- Compute pipelines and shaders
- Buffer and memory management
- Descriptor sets and layouts
- Command buffers and queues
- Synchronization primitives (fences, semaphores, events)

## Future Plans

1. **Vulkan 1.1+ Compute Features**
   - Subgroups
   - Variable pointers
   - Protected memory

2. **Rust-Native Enhancements**
   - Async/await for fence waiting
   - Iterator-based APIs
   - Builder patterns for all structures

3. **Compute-Specific Optimizations**
   - Streamlined memory allocation
   - Batch command submission
   - Automatic synchronization

## License

[To be determined]

## Acknowledgments

Built as a compute-focused alternative to full Vulkan implementations.