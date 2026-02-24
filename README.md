# Kronos Compute 🚀

> **📦 Release Candidate 3 (v0.2.3-rc3): Rust-first compute API with hardened Vulkan integration.**

[![Crates.io](https://img.shields.io/crates/v/kronos-compute.svg)](https://crates.io/crates/kronos-compute)
[![Documentation](https://docs.rs/kronos-compute/badge.svg)](https://docs.rs/kronos-compute)
[![Windows CI](https://github.com/LynnColeArt/kronos-compute/actions/workflows/windows.yml/badge.svg)](https://github.com/LynnColeArt/kronos-compute/actions/workflows/windows.yml)
[![License](https://img.shields.io/crates/l/kronos-compute.svg)](https://github.com/LynnColeArt/kronos-compute)

## Documentation Status

[implemented] This README uses explicit tags:
- `[measured]` — reproducible measurement with hardware/method metadata.
- `[implemented]` — completed and integrated.
- `[experimental]` — implemented but not production-proven.
- `[planned]` — not yet implemented.

Kronos production posture:

- [implemented] Kronos is the active intended compute backend strategy (with active recovery validation).
- [experimental] Runtime path depends on Vulkan ICD discovery and runtime availability.
- [planned] Full multi-platform parity and complete documentation of all vendor edge-cases.

### Production capability snapshot

- [implemented] Rust API for compute context/pipeline dispatch.
- [experimental] Pure-Rust driver implementation narrative is not complete without validated external Vulkan interoperability assumptions.
- [implemented] Backend selection and telemetry hooks are part of the runtime contract.

## Overview

[implemented] Kronos is a Rust-first implementation of compute-oriented Vulkan APIs, with reduced surface area (compute-only), while still integrating via Vulkan ICD loading and validated driver compatibility.

[implemented] This means:
- [implemented] No graphics path overhead.
- [implemented] Build-time and runtime startup can fail fast with explicit diagnostics when Vulkan dependencies are not compatible.
- [planned] "Works on any system" behavior remains aspirational until full validation is complete.

Kronos Compute is a streamlined Vulkan implementation that removes graphics functionality to focus on compute-focused runtime paths. The pure Rust implementation provides:

- [experimental] Descriptor lifecycle design emphasizes reduced descriptor churn in validated scenarios.
- [experimental] Barrier scheduling targets reduced transition overhead under stable workloads.
- [experimental] CPU-side execution and allocator behavior are hardware/ICD dependent.
- [planned] Zero-allocation steady state is the target profile, currently workload and backend dependent.
- [implemented] Structure layout compactness via core type definitions.

## 🎯 Key Features

### 1. **Safe Unified API** 🆕

- [implemented] Primary safe API entrypoint for context/pipeline creation and dispatch.
- [implemented] RAII and builder-style ownership.
- [implemented] Type-safe handle abstraction.

### 2. **Advanced Optimizations**

#### Persistent Descriptors
- [experimental] Set0 and push constant model are being finalized.
- [experimental] Descriptor-update reduction depends on workload and pipeline shape.

#### Intelligent Barrier Policy
- [implemented] Barrier policy model and transition tracking are part of API behavior.
- [experimental] Measured barriers-vs-dispatch results are platform-sensitive.

#### Timeline Semaphore Batching
- [experimental] Timeline semaphore batching is implemented in the design.
- [experimental] CPU overhead gains vary by backend and ICD behavior.

#### Advanced Memory Allocator
- Three-pool system: DEVICE_LOCAL, HOST_VISIBLE|COHERENT, HOST_VISIBLE|CACHED
- Slab-based sub-allocation with 256MB slabs
- Power-of-2 block sizes for O(1) allocation/deallocation

### 3. **Type-Safe Implementation**
- Safe handles with phantom types
- Proper error handling with Result types
- Zero-cost abstractions
- Memory safety guarantees

### 4. **Pure Rust Implementation** (NEW in v0.2.3)
- [implemented] Compute API surface and safe Rust orchestration are implemented.
- [experimental] Runtime execution is still coupled to Vulkan ICD/runtime compatibility.
- [implemented] API compatibility layer for existing-style Vulkan calls is maintained.
- [experimental] Backend behaviors are actively being validated in cross-vendor contexts.

### 5. **Optimized Structures**
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
├── benches/                # Validation artifacts
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
    ├── benchmarks/         # Historical validation artifacts
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
```

## 📊 Validation notes (benchmarks deferred)

Benchmark execution is currently deferred while backend proofing stabilizes.

- [implemented] Runtime startup, dispatch, and synchronization behavior are validated by existing test coverage.
- [experimental] Perf-focused claims are intentionally withheld pending reproducible benchmark refresh.

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

## 📈 Performance and Validation

Benchmark-driven performance claims are intentionally deferred while backend proofing and release diagnostics are stabilized.

- [implemented] Runtime startup, dispatch, synchronization, and failure-path behavior are validated by current test and example coverage.
- [experimental] Throughput/latency/submit-cost comparisons are currently withheld.
- [planned] Quantitative benchmark claims will be reintroduced once Kronos-only production behavior is stable and benchmark evidence is rerun and tagged.

## 🔧 Configuration

Kronos can be configured via environment variables:

- `KRONOS_ICD_SEARCH_PATHS`: Custom Vulkan ICD search paths
- `VK_ICD_FILENAMES`: Standard Vulkan ICD override
- `RUST_LOG`: Logging level (info, debug, trace)

### ICD Discovery Logging
Enable detailed logs to debug ICD discovery and loading:

```bash
RUST_LOG=kronos_compute=info,kronos_compute::implementation::icd_loader=debug cargo run
```

Logs include:
- Search paths scanned
- Each discovered manifest JSON
- Each library load attempt (as-provided and manifest-relative)
- Errors per candidate and the selected ICD summary

### ICD Selection
You can enumerate available ICDs and select one explicitly when creating a context.

- Enumerate programmatically:

```rust
use kronos_compute::implementation::icd_loader;
let icds = icd_loader::available_icds();
for (i, icd) in icds.iter().enumerate() {
    println!("[{i}] {} ({}), api=0x{:x}",
        icd.library_path.display(),
        if icd.is_software { "software" } else { "hardware" },
        icd.api_version);
}
```

- Select via `ContextBuilder`:

```rust
use kronos_compute::api;
let ctx = api::ComputeContext::builder()
    .prefer_icd_index(0)               // or .prefer_icd_path("/path/to/libvulkan_*.so")
    .build()?;
println!("Using ICD: {:?}", ctx.icd_info());
```

- Example CLI:

```bash
cargo run --example icd_select -- list
cargo run --example icd_select -- index 0
cargo run --example icd_select -- path /usr/lib/x86_64-linux-gnu/libvulkan_radeon.so
```

### Aggregated Mode (Experimental)
Aggregated mode exposes physical devices from multiple ICDs in a single instance and routes calls to the correct ICD by handle provenance.

- Enable:
```bash
KRONOS_AGGREGATE_ICD=1 RUST_LOG=kronos_compute=info,kronos_compute::implementation::icd_loader=debug cargo run
```

- Behavior:
  - `vkCreateInstance` creates a meta-instance wrapping per‑ICD instances.
  - `vkEnumeratePhysicalDevices` returns a combined list across all ICDs.
  - `vkCreateDevice` routes by the physical device’s owning ICD.
  - Subsequent queue, pool, command buffer and all `vkCmd*` calls route by handle.

- Caveats:
  - Experimental: Intended for orchestration and testing; API surface remains Vulkan-compatible, but behavior is meta-loader-like.
  - Performance: Routing adds a small handle→ICD lookup; negligible vs GPU work.
  - Diagnostics: enable debug logs for provenance and routing visibility.

### Windows CI / Headless Testing
- Linking: on Windows, linking to `vulkan-1` is opt-in. Set `KRONOS_LINK_VULKAN=1` if the Vulkan runtime is installed. CI uses direct ICD loading by default.
- Unit tests: run on `windows-latest` via `.github/workflows/windows.yml` without a GPU.
- Optional ICD tests: provide a software ICD (e.g., SwiftShader) and set:
  - `VK_ICD_FILENAMES` to the SwiftShader JSON path
  - `KRONOS_ALLOW_UNTRUSTED_LIBS=1` (if path is outside trusted prefixes)
  - `KRONOS_RUN_ICD_TESTS=1` to enable ignored tests
  - (Optional) `KRONOS_AGGREGATE_ICD=1` to test aggregated enumeration

### Security Notes (ICD Loading)
- Paths from `VK_ICD_FILENAMES` and discovery directories are canonicalized and validated.
- Libraries must resolve to regular files under trusted prefixes (Linux defaults: `/usr/lib`, `/usr/lib64`, `/usr/local/lib`, `/lib`, `/lib64`, `/usr/lib/x86_64-linux-gnu`).
- For development on non-standard locations, set `KRONOS_ALLOW_UNTRUSTED_LIBS=1` to override the trust policy (not recommended for production).

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
// Traditional path may issue multiple descriptor updates per dispatch
vkUpdateDescriptorSets(device, 5, writes, 0, nullptr);
vkCmdBindDescriptorSets(cmd, COMPUTE, layout, 0, 1, &set, 0, nullptr);

// Kronos route focuses on descriptor-set stability with push constants
vkCmdPushConstants(cmd, layout, COMPUTE, 0, 128, &params);
vkCmdDispatch(cmd, x, y, z);
```

### Smart Barriers
Kronos tracks buffer usage patterns and inserts only the minimum required barriers:

```rust
// Traditional path may require multiple barrier transitions
vkCmdPipelineBarrier(cmd, TRANSFER, COMPUTE, ...);  // upload→compute
vkCmdPipelineBarrier(cmd, COMPUTE, COMPUTE, ...);   // compute→compute  
vkCmdPipelineBarrier(cmd, COMPUTE, TRANSFER, ...);  // compute→download

// Kronos applies minimized barrier scheduling where safe
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
  - [Troubleshooting](docs/TROUBLESHOOTING.md) - Common issues and ICD loader diagnostics
  
- **Quality Assurance**: Test results and validation reports
  - [QA Report](docs/qa/QA_REPORT.md) - Comprehensive validation for Sporkle integration
  - [Test Results](docs/qa/TEST_RESULTS.md) - Unit and integration test details
  

## 🤝 Contributing

Contributions are welcome! Areas of interest:

1. SPIR-V shader integration for validation cases
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

// Safe Rust wrapper
let buffer = device.create_buffer(&info)?;
```

All unsafe functions include comprehensive safety documentation.

## 📦 Features

- `implementation` - Enable Kronos optimizations and ICD forwarding
- `validation` - Enable additional safety checks (default)
- 
## 📝 Status

- [implemented] Core implementation scaffold complete.
- [experimental] Optimization integration is present but still requires cross-vendor proof under Kronos-first semantics.
- [implemented] ICD loader with Vulkan forwarding.
- [implemented] Validation harnesses and safety checks are present.
- [measured] Basic examples working.
- [implemented] Crates.io publication recorded (v0.1.0; confirm current package state before relying on it).
- [implemented] C header generation.
- [implemented] SPIR-V shader build scripts.
- [implemented] Safe unified API available.
- [measured] Compute correctness validated for baseline sample case.
- [planned] Safety documentation completeness by tag coverage is under implementation.
- [implemented] CI/CD pipeline with CI validation.
- [measured] Test suite baseline count is historical and should be revalidated in the current checkout.
- [experimental] Production testing and final readiness sign-off are deferred until cross-vendor evidence is current.

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
