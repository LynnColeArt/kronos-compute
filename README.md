# Kronos Compute 🚀

> **📦 Release Candidate 3 (v0.1.5-rc3): This project has reached release candidate status! The core functionality is stable, the unified safe API is complete, and all critical issues have been resolved. We welcome beta testing and feedback.**

[![Crates.io](https://img.shields.io/crates/v/kronos-compute.svg)](https://crates.io/crates/kronos-compute)
[![Documentation](https://docs.rs/kronos-compute/badge.svg)](https://docs.rs/kronos-compute)
[![License](https://img.shields.io/crates/l/kronos-compute.svg)](https://github.com/LynnColeArt/kronos-compute)

A high-performance, compute-only Vulkan implementation in Rust, featuring state-of-the-art GPU compute optimizations.

## Overview

Kronos Compute is a streamlined Vulkan implementation that removes all graphics functionality to achieve maximum GPU compute performance. This Rust port not only provides memory-safe abstractions over the C API but also implements cutting-edge optimizations that deliver:

- **Zero descriptor updates** per dispatch
- **≤0.5 barriers** per dispatch (83% reduction)
- **30-50% reduction** in CPU submit time
- **Zero memory allocations** in steady state
- **13.9% reduction** in structure sizes

## 🎯 Key Features

### 1. **Safe Unified API** 🆕

- Zero unsafe code required
- Automatic resource management (RAII)
- Builder patterns and fluent interfaces
- Type-safe abstractions
- All optimizations work transparently

### 2. **Advanced Optimizations**

#### Persistent Descriptors
- Set0 reserved for storage buffers with zero updates in hot path
- Parameters passed via push constants (≤128 bytes)
- Eliminates descriptor set allocation and update overhead

#### Intelligent Barrier Policy
- Smart tracking reduces barriers from 3 per dispatch to ≤0.5
- Only three transition types: upload→read, read→write, write→read
- Vendor-specific optimizations for AMD, NVIDIA, and Intel GPUs

#### Timeline Semaphore Batching
- One timeline semaphore per queue
- Batch multiple submissions with a single fence
- 30-50% reduction in CPU overhead

#### Advanced Memory Allocator
- Three-pool system: DEVICE_LOCAL, HOST_VISIBLE|COHERENT, HOST_VISIBLE|CACHED
- Slab-based sub-allocation with 256MB slabs
- Power-of-2 block sizes for O(1) allocation/deallocation

### 3. **Type-Safe Implementation**
- Safe handles with phantom types
- Proper error handling with Result types
- Zero-cost abstractions
- Memory safety guarantees

### 4. **Optimized Structures**
- `VkPhysicalDeviceFeatures`: 32 bytes (vs 220 in standard Vulkan)
- `VkBufferCreateInfo`: Reordered fields for better packing
- `VkMemoryTypeCache`: O(1) memory type lookups

## 📁 Project Structure

```
kronos/
├── src/
│   ├── lib.rs              # Main library entry point
│   ├── sys/                # Low-level FFI types
│   ├── core/               # Core Kronos types
│   ├── ffi/                # C-compatible function signatures
│   └── implementation/     # Kronos optimizations
├── benches/                # Performance benchmarks
├── examples/               # Usage examples
├── tests/                  # Integration and unit tests
├── shaders/                # SPIR-V compute shaders
├── scripts/                # Build and validation scripts
└── docs/                   # Documentation
    ├── architecture/       # Design documents
    │   ├── OPTIMIZATION_SUMMARY.md
    │   ├── VULKAN_COMPARISON.md
    │   ├── ICD_SUCCESS.md
    │   └── COMPATIBILITY.md
    ├── benchmarks/         # Performance results
    │   └── BENCHMARK_RESULTS.md
    ├── qa/                 # Quality assurance
    │   ├── QA_REPORT.md
    │   ├── MINI_REVIEW.md
    │   └── TEST_RESULTS.md
    ├── EPIC.md             # Project epic and vision
    └── TODO.md             # Development roadmap
```

## 🛠️ Installation

### From crates.io
```bash
cargo add kronos-compute
```

[![Crates.io](https://img.shields.io/crates/v/kronos-compute.svg)](https://crates.io/crates/kronos-compute)
[![Documentation](https://docs.rs/kronos-compute/badge.svg)](https://docs.rs/kronos-compute)

### From Source

#### Prerequisites
- Rust 1.70 or later
- Vulkan SDK (for ICD loader and validation layers)
- A Vulkan-capable GPU with compute support
- Build tools (gcc/clang on Linux, Visual Studio on Windows, Xcode on macOS)
- (Optional) SPIR-V compiler (glslc or glslangValidator) for shader development

See [Development Setup Guide](docs/DEVELOPMENT_SETUP.md) for detailed installation instructions.

#### Build Steps
```bash
# Clone the repository
git clone https://github.com/LynnColeArt/kronos-compute
cd kronos-compute

# Build SPIR-V shaders (optional, pre-built shaders included)
./scripts/build_shaders.sh

# Build with optimizations enabled
cargo build --release --features implementation

# Run tests
cargo test --features implementation

# Run benchmarks
cargo bench --features implementation

# Run validation scripts
./scripts/validate_bench.sh      # Run all validation tests
./scripts/amd_bench.sh          # AMD-specific validation
```

## 📊 Benchmarks

Kronos includes comprehensive benchmarks for common compute workloads:

- **SAXPY**: Vector multiply-add operations (c = a*x + b)
- **Reduction**: Parallel array summation
- **Prefix Sum**: Parallel scan algorithm
- **GEMM**: Dense matrix multiplication (C = A * B)

Each benchmark tests multiple configurations:
- Sizes: 64KB (small), 8MB (medium), 64MB (large)
- Batch sizes: 1, 16, 256 dispatches
- Metrics: descriptor updates, barriers, CPU time, memory allocations

```bash
# Run specific benchmark
cargo bench --bench compute_workloads --features implementation

# Run with custom parameters
cargo bench --bench compute_workloads -- --warm-up-time 5 --measurement-time 10
```

## 🚀 Usage Example

### Safe Unified API (Recommended)

```rust
use kronos_compute::api::{ComputeContext, PipelineConfig, BufferBinding};

// No unsafe code needed!
let ctx = ComputeContext::new()?;

// Load shader and create pipeline
let shader = ctx.load_shader("compute.spv")?;
let pipeline = ctx.create_pipeline(&shader)?;

// Create buffers
let input = ctx.create_buffer(&data)?;
let output = ctx.create_buffer_uninit(size)?;

// Dispatch compute work
ctx.dispatch(&pipeline)
    .bind_buffer(0, &input)
    .bind_buffer(1, &output)
    .workgroups(1024, 1, 1)
    .execute()?;

// Read results
let results: Vec<f32> = output.read()?;
```

All optimizations work transparently through the safe API!

### Low-Level FFI (Advanced)

```rust
use kronos_compute::*;

unsafe {
    // Traditional Vulkan-style API also available
    initialize_kronos()?;
    let mut instance = VkInstance::NULL;
    vkCreateInstance(&create_info, ptr::null(), &mut instance);
    // ... etc
}
```

## 📈 Performance

Based on Mini's optimization targets:

| Metric | Baseline Vulkan | Kronos | Improvement |
|--------|----------------|---------|-------------|
| Descriptor updates/dispatch | 3-5 | 0 | 100% ⬇️ |
| Barriers/dispatch | 3 | ≤0.5 | 83% ⬇️ |
| CPU submit time | 100% | 50-70% | 30-50% ⬇️ |
| Memory allocations | Continuous | 0* | 100% ⬇️ |
| Structure size (avg) | 100% | 86.1% | 13.9% ⬇️ |

*After initial warm-up

## 🔧 Configuration

Kronos can be configured via environment variables:

- `KRONOS_ICD_SEARCH_PATHS`: Custom Vulkan ICD search paths
- `VK_ICD_FILENAMES`: Standard Vulkan ICD override
- `RUST_LOG`: Logging level (info, debug, trace)

Runtime configuration through the API:
```rust
// Set timeline batch size
kronos::implementation::timeline_batching::set_batch_size(32)?;

// Configure memory pools
kronos::implementation::pool_allocator::set_slab_size(512 * 1024 * 1024)?;
```

## ⚡ How It Works

### Persistent Descriptors
Traditional Vulkan requires updating descriptor sets for each dispatch. Kronos pre-allocates all storage buffer descriptors in Set0 and uses push constants for parameters:

```rust
// Traditional: 3-5 descriptor updates per dispatch
vkUpdateDescriptorSets(device, 5, writes, 0, nullptr);
vkCmdBindDescriptorSets(cmd, COMPUTE, layout, 0, 1, &set, 0, nullptr);

// Kronos: 0 descriptor updates
vkCmdPushConstants(cmd, layout, COMPUTE, 0, 128, &params);
vkCmdDispatch(cmd, x, y, z);
```

### Smart Barriers
Kronos tracks buffer usage patterns and inserts only the minimum required barriers:

```rust
// Traditional: 3 barriers per dispatch
vkCmdPipelineBarrier(cmd, TRANSFER, COMPUTE, ...);  // upload→compute
vkCmdPipelineBarrier(cmd, COMPUTE, COMPUTE, ...);   // compute→compute  
vkCmdPipelineBarrier(cmd, COMPUTE, TRANSFER, ...);  // compute→download

// Kronos: ≤0.5 barriers per dispatch (automatic)
```

### Timeline Batching
Instead of submitting each command buffer individually:

```rust
// Traditional: N submits, N fences
for cmd in commands {
    vkQueueSubmit(queue, 1, &submit, fence);
}

// Kronos: 1 submit, 1 timeline semaphore
kronos::BatchBuilder::new(queue)
    .add_command_buffer(cmd1)
    .add_command_buffer(cmd2)
    .submit()?;
```

## 📚 Documentation

Comprehensive documentation is available in the `docs/` directory:

- **API Documentation**:
  - [Unified Safe API](docs/UNIFIED_API.md) - 🆕 Safe, ergonomic Rust API (recommended)
  
- **Architecture**: Design decisions, optimization details, and comparisons
  - [Optimization Summary](docs/architecture/OPTIMIZATION_SUMMARY.md) - Mini's 4 optimizations explained
  - [Vulkan Comparison](docs/architecture/VULKAN_COMPARISON.md) - Differences from standard Vulkan
  - [ICD Integration](docs/architecture/ICD_SUCCESS.md) - How Kronos integrates with existing drivers
  
- **Quality Assurance**: Test results and validation reports
  - [QA Report](docs/qa/QA_REPORT.md) - Comprehensive validation for Sporkle integration
  - [Test Results](docs/qa/TEST_RESULTS.md) - Unit and integration test details
  
- **Benchmarks**: Performance measurements and analysis
  - [Benchmark Results](docs/benchmarks/BENCHMARK_RESULTS.md) - Detailed performance metrics

## 🤝 Contributing

Contributions are welcome! Areas of interest:

1. SPIR-V shader integration for benchmarks
2. Additional vendor-specific optimizations
3. Performance profiling on different GPUs
4. Safe wrapper API design
5. Documentation improvements

Please read our [Contributing Guide](CONTRIBUTING.md) for details.

## 🔐 Safety

This crate uses `unsafe` for FFI compatibility but provides safe abstractions where possible:

```rust
// Unsafe C-style API (required for compatibility)
let result = unsafe { 
    vkCreateBuffer(device, &info, ptr::null(), &mut buffer) 
};

// Safe Rust wrapper (future work)
let buffer = device.create_buffer(&info)?;
```

All unsafe functions include comprehensive safety documentation.

## 📦 Features

- `implementation` - Enable Kronos optimizations and ICD forwarding
- `validation` - Enable additional safety checks (default)
- `compare-ash` - Enable comparison benchmarks with ash

## 📝 Status

- ✅ Core implementation complete
- ✅ All optimizations integrated  
- ✅ ICD loader with Vulkan forwarding
- ✅ Comprehensive benchmark suite
- ✅ Basic examples working
- ✅ Published to crates.io (v0.1.0)
- ✅ C header generation
- ✅ SPIR-V shader build scripts
- ✅ Safe unified API (NEW!)
- ✅ Compute correctness fixed (1024/1024 correct results)
- ✅ Safety documentation complete (100% coverage)
- ✅ CI/CD pipeline with multi-platform testing
- ✅ Test suite expanded (46 tests passing)
- ⏳ Production testing

## 🗺️ Roadmap

### v0.2.0 (Q1 2025)
- NVIDIA & Intel GPU optimizations
- Multi-queue concurrent dispatch support
- Dynamic memory pool resizing
- Vulkan validation layer support

### v0.3.0 (Q2 2025)
- Enhanced Sporkle integration
- Advanced timeline semaphore patterns
- Ray query & cooperative matrix support
- Performance regression testing

### v1.0.0 (Q3 2025)
- Production-ready status
- Full Vulkan 1.3 compute coverage
- Platform-specific optimizations
- Enterprise support

See [TODO.md](TODO.md) for the complete roadmap and contribution opportunities.

## 🙏 Acknowledgments

- Mini (@notmini) for the groundbreaking optimization techniques
- The Vulkan community for driver support
- Contributors who helped port these optimizations to Rust

## 📜 License

This project is dual-licensed under MIT OR Apache-2.0. See [LICENSE-MIT](LICENSE-MIT) and [LICENSE-APACHE](LICENSE-APACHE) for details.

---

Built with ❤️ and 🦀 for maximum GPU compute performance.

## Citation

If you use Kronos in your research, please cite:

```bibtex
@software{kronoscompute2025,
  author = {Cole, Lynn},
  title = {Kronos Compute: A High-Performance Compute-Only Vulkan Implementation},
  year = {2025},
  publisher = {GitHub},
  journal = {GitHub repository},
  url = {https://github.com/LynnColeArt/kronos-compute}
}
```