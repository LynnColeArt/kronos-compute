# Safe API Change Request

## Problem Statement

The Kronos Compute safe API (`api::ComputeContext`) is not functioning correctly. While the low-level FFI implementation works for multi-GPU support, the safe API has critical issues preventing applications from using it.

### Current Issues

1. **Device Enumeration Failure (Single-ICD Mode)**
   - When `KRONOS_AGGREGATE_ICD` is not set, the safe API finds 0 physical devices
   - The low-level implementation successfully finds devices, but the safe API does not
   - Example: Intel Haswell ICD is selected but reports 0 devices

2. **Device Creation Failure (Aggregated Mode)**
   - When `KRONOS_AGGREGATE_ICD=1` is set, devices are found correctly
   - The 7900 XTX is properly selected as the discrete GPU
   - However, `vkCreateDevice` fails with `ErrorFeatureNotPresent`
   - No features are being requested (using default/all disabled)

3. **ICD Selection Crashes**
   - Selecting an ICD by path causes a segmentation fault
   - Selecting by index works but still results in DeviceNotFound
   - The crash only occurs with path-based selection

### Root Causes

1. **Potential ICD Function Loading Issues**
   - Instance-level functions may not be loaded correctly for the safe API
   - The device enumeration functions might not be properly initialized

2. **Feature Requirements Mismatch**
   - Even with no features requested, the driver reports `ErrorFeatureNotPresent`
   - This suggests either:
     - The device creation parameters are malformed
     - Required extensions are missing
     - The ICD routing is incorrect

3. **Memory Safety Issues**
   - The path-based ICD selection crash indicates memory corruption
   - String handling or lifetime issues in the ICD preference system

## Proposed Solution

### Phase 1: Fix Device Enumeration (Single-ICD Mode)
1. **Verify ICD function loading**
   - Ensure instance functions are loaded after instance creation
   - Add validation that vkEnumeratePhysicalDevices is properly loaded
   - Compare safe API flow with working low-level examples

2. **Debug enumeration path**
   - Add logging to track ICD handle propagation
   - Verify the instance handle is valid and correctly passed
   - Check if the ICD loader state is consistent

### Phase 2: Fix Device Creation (Aggregated Mode)
1. **Investigate feature requirements**
   - Query and log available device features before creation
   - Try creating device with NULL features pointer (instead of default struct)
   - Check if any implicit extensions are required

2. **Validate device creation parameters**
   - Ensure queue family indices are valid
   - Verify queue creation info is properly formed
   - Test with minimal device creation (no extensions, no features)

### Phase 3: Fix ICD Selection Crashes
1. **Fix path-based selection**
   - Review string lifetime management in preference system
   - Ensure PathBuf is properly handled through FFI boundary
   - Add bounds checking and validation

2. **Improve error handling**
   - Replace crashes with proper error returns
   - Add validation for ICD availability before selection
   - Ensure thread safety in ICD preference setting

## Implementation Tasks

### Immediate Tasks (v0.2.0-rc12)
- [ ] Add instance function validation logging
- [ ] Fix device enumeration in single-ICD mode
- [ ] Test with NULL features pointer
- [ ] Add comprehensive error logging for device creation

### Short-term Tasks (v0.2.0-rc13)
- [ ] Fix ErrorFeatureNotPresent in aggregated mode
- [ ] Resolve path-based ICD selection crash
- [ ] Add integration tests for safe API
- [ ] Document working configurations

### Long-term Tasks (v0.2.0)
- [ ] Full safe API test coverage
- [ ] Performance benchmarks comparing safe vs FFI API
- [ ] Examples demonstrating all safe API features
- [ ] Production-ready error handling

## Success Criteria

1. **Basic Functionality**
   - `ComputeContext::new()` successfully creates a context
   - Devices are enumerated in both single-ICD and aggregated modes
   - No crashes or segfaults during normal operation

2. **Multi-GPU Support**
   - Safe API can discover and use multiple GPUs
   - Buffer creation and compute dispatch work correctly
   - Performance matches low-level FFI implementation

3. **Developer Experience**
   - Clear error messages for all failure cases
   - Comprehensive logging for debugging
   - Examples work out of the box
   - Documentation covers common issues

## Testing Strategy

1. **Unit Tests**
   - Test each component of the safe API in isolation
   - Mock ICD responses for predictable testing
   - Verify error handling paths

2. **Integration Tests**
   - Test with real ICDs on various hardware
   - Verify multi-GPU scenarios
   - Test all ICD selection methods

3. **Example Programs**
   - Update unified_api_simple.rs to work correctly
   - Create multi-GPU example using safe API
   - Add debugging example with comprehensive logging

## Timeline

- **Week 1**: Fix device enumeration and basic functionality
- **Week 2**: Resolve aggregated mode issues and crashes
- **Week 3**: Complete testing and documentation
- **Week 4**: Release v0.2.0 with working safe API

## Notes

The safe API is critical for Kronos Compute adoption as it provides:
- Memory safety guarantees
- Ergonomic Rust interface
- Automatic resource management
- Simplified error handling

Fixing these issues will make Kronos Compute accessible to Rust developers who want GPU compute without dealing with unsafe FFI code.