# Known Issues

This document tracks known issues and workarounds for Kronos Compute.

## Current Issues

### 1. Example Compilation Warnings
**Issue**: Some examples generate warnings about unused variables and unsafe blocks.
**Workaround**: Run `cargo fix --examples` to apply automatic fixes.
**Status**: Low priority - doesn't affect functionality.

### 2. Benchmark SPIR-V Shaders Missing
**Issue**: Benchmarks require SPIR-V shaders that aren't included in the repository.
**Workaround**: Build shaders manually or use the provided `saxpy.spv` for testing.
**Status**: Planned for v0.2.0 - add shader build scripts.

### 3. Timeline Semaphore Fallback
**Issue**: Some older GPUs/drivers don't support timeline semaphores (Vulkan 1.2).
**Workaround**: The implementation automatically falls back to regular semaphores.
**Impact**: Reduced performance (no timeline batching optimization).

### 4. Limited Platform Testing
**Issue**: Primary development and testing on Linux. Windows/macOS may have issues.
**Workaround**: Report platform-specific issues on GitHub.
**Status**: CI/CD now tests all platforms, but real hardware testing needed.

## Resolved Issues

### 1. ~~Compute Correctness Bug~~ ✅
**Fixed in**: 2025-08-25
**Issue**: compute_simple example produced incorrect results (0/1024 correct).
**Resolution**: Fixed shader loading and buffer descriptor ranges.

### 2. ~~Binary Import Errors~~ ✅
**Fixed in**: 2025-08-25
**Issue**: Binary examples failed to compile with "use of undeclared crate".
**Resolution**: Updated imports from `kronos` to `kronos_compute`.

### 3. ~~Missing Safety Documentation~~ ✅
**Fixed in**: 2025-08-25
**Issue**: 68% of unsafe functions lacked safety documentation.
**Resolution**: Added comprehensive safety docs to all 29 unsafe functions.

## Reporting Issues

When reporting new issues, please include:

1. **Environment**:
   - OS and version
   - GPU model and driver version
   - Rust version (`rustc --version`)
   - Vulkan SDK version

2. **Reproduction Steps**:
   - Minimal code example
   - Exact commands run
   - Expected vs actual behavior

3. **Error Messages**:
   - Full error output
   - Debug logs (`RUST_LOG=kronos_compute=debug`)

Submit issues at: https://github.com/LynnColeArt/kronos-compute/issues