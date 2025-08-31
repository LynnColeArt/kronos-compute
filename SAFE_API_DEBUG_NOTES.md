# Safe API Debug Notes

## Investigation Summary (v0.2.0-rc11)

### Original Issue
- Safe API crashes with corrupted stack when calling `ComputeContext::new()`
- Reported by sporkle claude while testing multi-GPU support

### Debugging Process

1. **Initial Crash Analysis**
   - Added comprehensive logging throughout `ComputeContext::new_with_config()`
   - Fixed device name string handling (conversion from `[i8]` to `[u8]`)
   - Crash disappeared when logging was added (timing-sensitive issue)

2. **Device Enumeration Issues**
   ```
   Single-ICD Mode:
   - Selected ICD: Intel Haswell (/usr/lib/x86_64-linux-gnu/libvulkan_intel_hasvk.so)
   - Physical devices found: 0
   - Result: DeviceNotFound error
   
   Aggregated Mode (KRONOS_AGGREGATE_ICD=1):
   - All 6 hardware ICDs loaded
   - Physical devices found: 2
   - Selected device: AMD Radeon RX 7900 XTX (RADV NAVI31)
   - Result: ErrorFeatureNotPresent when creating device
   ```

3. **ICD Selection Testing**
   ```bash
   # List ICDs - WORKS
   cargo run --example icd_select -- list
   
   # Select by index - DeviceNotFound
   cargo run --example icd_select -- index 3
   
   # Select by path - SEGFAULT
   cargo run --example icd_select -- path /usr/lib/x86_64-linux-gnu/libvulkan_radeon.so
   ```

### Key Findings

1. **The safe API has different behavior than low-level API**
   - Low-level examples work fine
   - Safe API fails at device enumeration or creation
   - Suggests issue with how safe API initializes/uses the implementation

2. **Aggregated mode partially works**
   - Successfully creates meta-instance
   - Properly enumerates devices from multiple ICDs
   - Fails at device creation with feature error

3. **Memory safety issues remain**
   - Path-based ICD selection causes segfault
   - Original crash was timing-sensitive
   - String handling needs review

### Code Locations

1. **Safe API Context Creation**
   - `/src/api/context.rs:51` - `new_with_config()`
   - `/src/api/context.rs:138` - `create_instance()`
   - `/src/api/context.rs:185` - `find_compute_device()`
   - `/src/api/context.rs:271` - `create_device()`

2. **ICD Preference Setting**
   - `/src/implementation/icd_loader.rs` - `set_preferred_icd_path()`
   - Potential issue with PathBuf lifetime through FFI

3. **Device Creation Parameters**
   - Using default `VkPhysicalDeviceFeatures` (all disabled)
   - No extensions requested
   - Single queue with compute support

### Next Steps

1. **Compare with working low-level example**
   - Side-by-side comparison of instance/device creation
   - Verify function pointer loading sequence
   - Check for missing initialization steps

2. **Test minimal device creation**
   - Try NULL features pointer instead of default struct
   - Remove all optional parameters
   - Test with different ICDs

3. **Fix memory safety issues**
   - Review all string handling in safe API
   - Check for use-after-free in ICD preference system
   - Add proper lifetime management

### Useful Commands for Testing

```bash
# Test with debug logging
RUST_LOG=kronos_compute=debug cargo run --example test_safe_api_crash

# Test with aggregated mode
KRONOS_AGGREGATE_ICD=1 RUST_LOG=kronos_compute=info cargo run --example test_safe_api_crash

# Test with specific ICD preference
KRONOS_PREFER_HARDWARE=0 cargo run --example test_safe_api_crash  # Include software renderers

# Run under debugger
RUST_LOG=kronos_compute=debug rust-gdb target/release/examples/test_safe_api_crash
```

### Hypothesis

The safe API might be:
1. Not properly initializing the ICD loader state
2. Missing critical function pointer loads after instance creation
3. Using incorrect handles or routing in aggregated mode
4. Having lifetime issues with string parameters

The fact that adding logging "fixed" the crash suggests a race condition or uninitialized memory access that gets masked by the slower execution with logging.