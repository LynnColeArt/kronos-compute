# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.2.0-rc7] - 2025-08-31

### Added
- Entry point logging to verify our vkCreateBuffer is being called
- Initialization logging to confirm Kronos implementation is active
- Immediate logging at function entry to diagnose routing issues

### Technical Details
The enhanced logging in rc6 wasn't showing up, suggesting either:
1. Our vkCreateBuffer implementation isn't being called at all
2. The application might be linking directly to system Vulkan
3. In aggregated mode, calls might be bypassing our implementation

Added logging at the very start of vkCreateBuffer and initialize_kronos to verify
whether our implementation is being used.

## [0.2.0-rc6] - 2025-08-31

### Added
- Enhanced debug logging for buffer creation to diagnose function loading issues
- Logging now shows whether create_buffer function was successfully loaded
- Detailed error messages when ICD doesn't have required functions

### Technical Details
Added comprehensive logging to track:
1. Whether device functions are loaded successfully
2. Whether create_buffer specifically is present in the ICD
3. Which path (device-specific or fallback) is being used
4. Why ErrorInitializationFailed might be returned

This will help diagnose why buffer creation fails even when device creation succeeds.

## [0.2.0-rc5] - 2025-08-31

### Fixed
- Buffer creation now works correctly in aggregated mode
- Device-ICD mapping properly handles fallback to single ICD when device lookup fails
- Added debug logging to trace device registration and buffer creation paths

### Technical Details
In aggregated mode, when `icd_for_device()` fails to find a device in the registry,
it now correctly falls back to the selected "best" ICD stored in `ICD_LOADER`.
This ensures buffer creation and other device operations work correctly even
if the device-ICD mapping is not found.

## [0.2.0-rc4] - 2025-08-31

### Fixed
- Aggregated mode now properly loads instance-level functions after creating instances
- Physical device enumeration now works correctly in aggregated mode
- Multi-GPU discovery now works without VK_ICD_FILENAMES workaround

### Technical Details
After creating instances with each ICD in aggregated mode, we now load the instance-level
functions (like vkEnumeratePhysicalDevices) for each ICD. This was the missing piece that
prevented device enumeration from working.

## [0.2.0-rc3] - 2025-08-31

### Fixed
- Build errors due to Vulkan linking - now only links to system Vulkan when `KRONOS_LINK_VULKAN=1` is set
- "undefined version" errors that prevented building on Linux systems

## [0.2.0-rc2] - 2025-08-31

### Fixed
- Multi-ICD enumeration now properly stores all discovered ICDs for aggregated mode
- `ALL_ICDS` is populated with all hardware ICDs during initialization
- `discover_and_load_all_icds()` now uses already loaded ICDs instead of re-discovering

## [0.2.0-rc1] - 2025-08-31

### Added
- Windows loader support using `libloading`; Windows trust policy with opt-in override.
- Aggregated ICD mode (experimental) behind `KRONOS_AGGREGATE_ICD=1`:
  - Aggregated `vkEnumeratePhysicalDevices` across ICDs
  - Per-handle routing for device/queue/command pool/cmd buffer and all `vkCmd*` calls
  - Descriptor/buffer/memory/shader/pipeline creation routed by device mapping
- New tests (ignored by default): safe API ICD selection, dispatch sanity, aggregated E2E, aggregated stress

### Changed
- Safer loader lifetime (Arc instead of unsafe 'static); replace-on-write update helpers
- Path canonicalization + trust policy for ICD discovery and loading

### Docs
- README: Windows CI/headless testing; ICD selection; Aggregated mode usage

## [0.1.6-rc3] - 2025-08-29

### Fixed
- AMD RADV not discovered unless `VK_ICD_FILENAMES` set: loader now resolves `library_path` both as-provided (honors dynamic linker search) and relative to the manifest directory. Ensures `/usr/share/vulkan/icd.d/radeon_icd.x86_64.json` works out-of-the-box on Ubuntu/Mesa.
- Incorrect `VkStructureType` constants for shader/pipeline objects corrected to match Vulkan spec (`ShaderModuleCreateInfo=15`, `ComputePipelineCreateInfo=28`, `PipelineCacheCreateInfo=17`).
- Potential double-destroy of Vulkan resources when cloning `ComputeContext`: Drop now only destroys on last clone.
- Invalid device-proc fallback: `load_device_functions` no longer uses a NULL instance fallback and now requires `vkGetDeviceProcAddr`.

### Improved
- ICD discovery logging: now logs all search paths, each discovered JSON, and each library load attempt, including errors per candidate.
- Global ICD function load failures are now propagated instead of silently ignored.

## [0.1.6-rc2] - 2025-08-29

### Changed
- VK_ICD_FILENAMES environment variable is now treated as an override priority, not exclusive
- ICD loader always discovers all available drivers for intelligent fallback
- If VK_ICD_FILENAMES points to non-existent files, Kronos will fall back to discovered drivers

### Improved
- Better handling of user preferences while maintaining automatic driver discovery
- Environment variable ICDs are prioritized but don't prevent fallback options

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

[Unreleased]: https://github.com/LynnColeArt/kronos-compute/compare/v0.2.0-rc7...HEAD
[0.2.0-rc7]: https://github.com/LynnColeArt/kronos-compute/compare/v0.2.0-rc6...v0.2.0-rc7
[0.2.0-rc6]: https://github.com/LynnColeArt/kronos-compute/compare/v0.2.0-rc5...v0.2.0-rc6
[0.2.0-rc5]: https://github.com/LynnColeArt/kronos-compute/compare/v0.2.0-rc4...v0.2.0-rc5
[0.2.0-rc4]: https://github.com/LynnColeArt/kronos-compute/compare/v0.2.0-rc3...v0.2.0-rc4
[0.2.0-rc3]: https://github.com/LynnColeArt/kronos-compute/compare/v0.2.0-rc2...v0.2.0-rc3
[0.2.0-rc2]: https://github.com/LynnColeArt/kronos-compute/compare/v0.2.0-rc1...v0.2.0-rc2
[0.2.0-rc1]: https://github.com/LynnColeArt/kronos-compute/compare/v0.1.6-rc3...v0.2.0-rc1
[0.1.6-rc3]: https://github.com/LynnColeArt/kronos-compute/compare/v0.1.6-rc2...v0.1.6-rc3
[0.1.6-rc2]: https://github.com/LynnColeArt/kronos-compute/compare/v0.1.6-rc1...v0.1.6-rc2
[0.1.6-rc1]: https://github.com/LynnColeArt/kronos-compute/compare/v0.1.5-rc3...v0.1.6-rc1
[0.1.5-rc3]: https://github.com/LynnColeArt/kronos-compute/compare/v0.1.5-rc2...v0.1.5-rc3
[0.1.5-rc2]: https://github.com/LynnColeArt/kronos-compute/compare/v0.1.5-rc1...v0.1.5-rc2
[0.1.5-rc1]: https://github.com/LynnColeArt/kronos-compute/compare/v0.1.0...v0.1.5-rc1
[0.1.0]: https://github.com/LynnColeArt/kronos-compute/releases/tag/v0.1.0
