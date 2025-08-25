# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Added
- Complete CI/CD pipeline with GitHub Actions
- Multi-platform testing (Linux, Windows, macOS)
- Code coverage reporting with tarpaulin
- Security auditing with cargo-audit
- Automated release workflow for crates.io
- Comprehensive safety documentation for all unsafe functions
- SAFETY_DOCUMENTATION_REPORT.md tracking all unsafe functions

### Fixed
- Compute correctness bug in compute_simple example (now 1024/1024 correct)
- Compilation errors in all binary examples (demo, test_minimal, test_optimizations)
- Import statements changed from `use kronos::*` to `use kronos_compute::*`

### Changed
- Updated EPIC2.md to reflect completed milestones

## [0.1.0] - 2024-12-29

### Added
- Initial release with unified safe API
- Core Kronos Compute implementation
- 4 key optimizations:
  - Persistent descriptors (zero updates per dispatch)
  - Smart barrier policy (≤0.5 barriers per dispatch)
  - Timeline semaphore batching (30-50% CPU reduction)
  - Pool allocator (zero allocations in steady state)
- Safe API components:
  - ComputeContext - main entry point
  - Buffer - safe memory management
  - Pipeline - shader abstraction
  - CommandBuilder - fluent dispatch API
- Examples demonstrating usage
- Comprehensive documentation
- FFI compatibility with existing Vulkan ICDs

### Known Issues
- Some examples may not work correctly without proper Vulkan drivers
- Timeline semaphore support requires Vulkan 1.2 or extensions
- Limited to compute operations only (no graphics)

[Unreleased]: https://github.com/LynnColeArt/kronos-compute/compare/v0.1.0...HEAD
[0.1.0]: https://github.com/LynnColeArt/kronos-compute/releases/tag/v0.1.0