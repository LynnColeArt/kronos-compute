# Changelog

All notable changes to Kronos Compute will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.1.0] - 2025-08-24

### Added
- Initial release of Kronos Compute
- Core Vulkan compute API implementation
- ICD (Installable Client Driver) forwarding for Vulkan compatibility
- Mini's 4 performance optimizations:
  - Persistent descriptors (zero updates per dispatch)
  - Smart barrier policy (≤0.5 barriers per dispatch)
  - Timeline semaphore batching (30-50% CPU overhead reduction)
  - 3-pool memory allocator (zero allocations in steady state)
- AMD GPU optimization support
- Comprehensive test suite (59 tests)
- Benchmark suite comparing against standard Vulkan
- Multiple working examples:
  - `compute_optimized` - Demonstrates all 4 optimizations
  - `compute_test` - Basic compute functionality
  - `descriptor_test` - Descriptor management
  - `simple_test` - Minimal example
  - `sync_test` - Synchronization primitives
  - `test_thread_safety` - Multi-threaded usage
- C header generation via cbindgen
- SPIR-V shader build scripts
- pkg-config support
- GitHub Actions CI/CD pipelines
- Dual MIT/Apache-2.0 licensing

### Performance Improvements
- 100% reduction in descriptor updates per dispatch
- 83% reduction in pipeline barriers per dispatch
- 30-50% reduction in CPU submit overhead
- 100% reduction in memory allocations after warm-up
- 13.9% reduction in average structure size

### Documentation
- Comprehensive README with usage examples
- API documentation
- Performance benchmarks and metrics
- Roadmap for future releases
- Citation information for academic use

[Unreleased]: https://github.com/LynnColeArt/kronos-compute/compare/v0.1.0...HEAD
[0.1.0]: https://github.com/LynnColeArt/kronos-compute/releases/tag/v0.1.0