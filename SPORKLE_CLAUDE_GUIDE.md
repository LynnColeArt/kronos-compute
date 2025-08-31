# Kronos Compute Multi-GPU Guide for Sporkle Claude

## Quick Start

To enable multi-GPU support in Kronos Compute v0.2.0-rc10:

```bash
export KRONOS_AGGREGATE_ICD=1
./your_application
```

## What's New in v0.2.0-rc13 🎯

- **v0.2.0-rc13**: Safe API FULLY WORKING! Fixed Arc<LoadedICD> lifetime issue, works in single & aggregated modes!
- **v0.2.0-rc12**: Safe API progress! Fixed ErrorFeatureNotPresent, but hangs in aggregated mode
- **v0.2.0-rc11**: Safe API crash fixed! Added comprehensive logging, but device enumeration issues remain
- **v0.2.0-rc10**: DIAGNOSTIC - Shows "KRONOS vkCreateBuffer called (v0.2.0-rc10)" if using correct implementation
- **v0.2.0-rc9**: API now explicitly uses implementation functions (but external apps might still use system Vulkan)
- **v0.2.0-rc8**: API layer logging revealed calls were bypassing implementation 
- **v0.2.0-rc7**: Entry point logging confirmed implementation wasn't being called
- **v0.2.0-rc6**: Enhanced debug logging! Shows why create_buffer might fail
- **v0.2.0-rc5**: Buffer creation attempted fix - handles device-ICD mapping in aggregated mode
- **v0.2.0-rc4**: Multi-GPU WORKS! Fixed instance-level function loading in aggregated mode
- **v0.2.0-rc3**: Fixed build errors! Now compiles without Vulkan linking issues
- **v0.2.0-rc2**: Fixed multi-ICD enumeration to discover all GPUs
- You can now discover both AMD GPUs (integrated + 7900 XTX) without the `VK_ICD_FILENAMES` workaround!

## Environment Variables

### Required for Multi-GPU
- `KRONOS_AGGREGATE_ICD=1` - Enables aggregated mode to enumerate devices from all ICDs

### Optional Configuration
- `KRONOS_PREFER_HARDWARE=1` (default) - Filters out software renderers like llvmpipe
- `KRONOS_PREFER_HARDWARE=0` - Include software renderers in enumeration
- `RUST_LOG=kronos_compute::implementation::icd_loader=info` - Enable debug logging

## Testing Multi-GPU Discovery

### 1. List Available ICDs
```bash
cargo run --example icd_select -- list
```

Expected output should show all hardware ICDs:
```
Found 6 ICD(s):
[0] /usr/lib/x86_64-linux-gnu/libvulkan_radeon.so (hardware)
[1] /usr/lib/x86_64-linux-gnu/libvulkan_intel.so (hardware)
...
```

### 2. Test Aggregated Mode
```bash
KRONOS_AGGREGATE_ICD=1 cargo run --example icd_select -- list
```

This should show the same ICDs are available in aggregated mode.

### 3. Enumerate Physical Devices
With aggregated mode enabled, your application should now see all GPUs:

```rust
// Your conv2d application should now enumerate both:
// - AMD Radeon Graphics (integrated)
// - AMD Radeon RX 7900 XTX (discrete)
```

## How It Works

In aggregated mode, Kronos Compute:
1. Discovers all available Vulkan ICDs on the system
2. Creates instances with each ICD
3. Aggregates physical devices from all ICDs into a single enumeration
4. Routes Vulkan calls to the appropriate ICD based on handle provenance

## Troubleshooting

### Buffer Creation Errors?

**CRITICAL**: Do NOT link your application to system Vulkan when using Kronos!

Kronos provides its own Vulkan implementation. If you link to both:
- The dynamic linker might use system Vulkan's vkCreateBuffer instead of Kronos's
- This breaks multi-GPU support because system Vulkan doesn't understand Kronos's aggregated mode

To diagnose:
- Enable logging: `RUST_LOG=kronos_compute=info`
- Look for: "KRONOS vkCreateBuffer called (v0.2.0-rc10)"
- If you don't see this, you're using system Vulkan by mistake

How to fix:
- Remove `-lvulkan` or `vulkan-1.lib` from your linker flags
- Only link to `kronos_compute`
- Kronos provides all the Vulkan functions you need

### Build Errors?
If you see "undefined version" errors when building:
- This was fixed in v0.2.0-rc3
- The build no longer links to system Vulkan by default
- If you need to link to system Vulkan for some reason, set `KRONOS_LINK_VULKAN=1`

### Still Only Seeing llvmpipe?
1. Ensure you're using v0.2.0-rc4 or later (fixes device enumeration)
2. Check that `KRONOS_AGGREGATE_ICD=1` is set
3. Enable logging: `RUST_LOG=kronos_compute::implementation::icd_loader=info`

### No Devices Found?
- The fix stores all ICDs correctly, but physical device enumeration depends on the ICDs actually returning devices
- Check that your Vulkan drivers are properly installed
- Try with `VK_ICD_FILENAMES` as a comparison to ensure drivers work

### Performance Considerations
- Aggregated mode has minimal overhead for device enumeration
- Once a device is selected, all subsequent calls go directly to that device's ICD
- No performance impact on compute operations

## Example Usage

```rust
use kronos_compute::api::ComputeContext;

// Enable aggregated mode before creating context
std::env::set_var("KRONOS_AGGREGATE_ICD", "1");

// Create context - will enumerate all GPUs
let ctx = ComputeContext::builder()
    .app_name("Multi-GPU Conv2D")
    .prefer_discrete_gpu()  // Automatically select 7900 XTX
    .build()?;

// Or manually select by index
let ctx = ComputeContext::builder()
    .app_name("Multi-GPU Conv2D")
    .prefer_device_index(1)  // Select second GPU (likely 7900 XTX)
    .build()?;
```

## Verification

To verify the fix is working:

1. Count ICDs stored:
   ```bash
   KRONOS_AGGREGATE_ICD=1 RUST_LOG=kronos_compute=debug cargo test aggregated_mode_test -- --nocapture
   ```
   
   Should show: `ALL_ICDS contains 6 ICDs` (or similar based on your system)

2. The workaround should no longer be needed:
   ```bash
   # This should work WITHOUT setting VK_ICD_FILENAMES
   KRONOS_AGGREGATE_ICD=1 ./your_conv2d_app
   ```

## Feedback

If you encounter any issues with multi-GPU enumeration in v0.2.0-rc2, please report them with:
- Output of `cargo run --example icd_select -- list`
- Your GPU configuration
- Any error messages with `RUST_LOG=kronos_compute=debug` enabled