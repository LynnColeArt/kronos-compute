# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.1.6-rc1] - 2025-08-29

### Fixed
- ICD loader now properly discovers all available Vulkan drivers instead of stopping at the first one
- Hardware drivers (AMD, NVIDIA, Intel) are now prioritized over software renderers (llvmpipe)
- No longer need to manually set VK_ICD_FILENAMES to use hardware GPUs
- Fixed issue where only llvmpipe was being detected even when hardware drivers were installed

### Added
- Improved ICD loader logging showing all discovered drivers and their types (hardware/software)
- Clear indication when falling back to software rendering

## [0.1.5-rc3] - 2025-08-29

### Fixed
- GPU selection now properly prioritizes hardware devices over software renderers (llvmpipe)
- Added device name and type logging for debugging GPU selection

## [0.1.5-rc2] - 2025-08-29

### Fixed
- Added `implementation` feature to default features so the Vulkan driver implementation is included by default
- Fixed package publishing to include working implementation out of the box

## [0.1.5-rc1] - 2025-08-25

### Added
- Complete CI/CD pipeline with GitHub Actions
- Multi-platform testing (Linux, Windows, macOS)
- Code coverage reporting with tarpaulin
- Security auditing with cargo-audit
- Automated release workflow for crates.io
- Comprehensive safety documentation for all unsafe functions
- SAFETY_DOCUMENTATION_REPORT.md tracking all unsafe functions
- Test suite expansion from 31 to 46 tests
- Unit tests for implementation modules
- Integration tests for unified API
- TROUBLESHOOTING.md guide for common issues
- KNOWN_ISSUES.md tracking current and resolved issues
- DEVELOPMENT_SETUP.md for contributor onboarding
- rust-version = "1.70" in Cargo.toml for MSRV

### Fixed
- Compute correctness bug in compute_simple example (now 1024/1024 correct)
- Compilation errors in all binary examples (demo, test_minimal, test_optimizations)
- Import statements changed from `use kronos::*` to `use kronos_compute::*`
- Private field access issues in tests (added public methods to BatchBuilder)

### Changed
- Updated EPIC2.md to reflect completed milestones
- README.md updated with accurate test counts and status
- Added build prerequisites and development setup link to README

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

[Unreleased]: https://github.com/LynnColeArt/kronos-compute/compare/v0.1.6-rc1...HEAD
[0.1.6-rc1]: https://github.com/LynnColeArt/kronos-compute/compare/v0.1.5-rc3...v0.1.6-rc1
[0.1.5-rc3]: https://github.com/LynnColeArt/kronos-compute/compare/v0.1.5-rc2...v0.1.5-rc3
[0.1.5-rc2]: https://github.com/LynnColeArt/kronos-compute/compare/v0.1.5-rc1...v0.1.5-rc2
[0.1.5-rc1]: https://github.com/LynnColeArt/kronos-compute/compare/v0.1.0...v0.1.5-rc1
[0.1.0]: https://github.com/LynnColeArt/kronos-compute/releases/tag/v0.1.0