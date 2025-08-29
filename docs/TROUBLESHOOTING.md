# Troubleshooting Guide

This guide helps resolve common issues with Kronos Compute.

## Table of Contents
- [Build Issues](#build-issues)
- [Runtime Errors](#runtime-errors)
- [Performance Issues](#performance-issues)
- [Platform-Specific Issues](#platform-specific-issues)

## Build Issues

### Error: "failed to select a version for the requirement `rayon = ^1.8`"
**Solution**: The project requires Rust 1.70+. Earlier versions may have dependency conflicts. Update Rust:
```bash
rustup update stable
```

### Error: "use of unstable library feature"
**Solution**: Some dependencies may require newer Rust features. Ensure you're using at least Rust 1.70:
```bash
rustc --version
```

### Error: "cannot find -lvulkan"
**Solution**: Install the Vulkan SDK for your platform:
- **Ubuntu/Debian**: `sudo apt install vulkan-sdk`
- **Fedora**: `sudo dnf install vulkan-devel`
- **Windows**: Download from [LunarG](https://vulkan.lunarg.com/sdk/home)
- **macOS**: `brew install vulkan-sdk`

## Runtime Errors

### Error: "Failed to initialize Kronos: NoManifestsFound"
**Cause**: No Vulkan drivers found.

**Solutions**:
1. Install GPU drivers with Vulkan support
2. Set the ICD path manually:
   ```bash
   export VK_ICD_FILENAMES=/path/to/icd.json
   ```

### Error: "No compute-capable device found"
**Cause**: No GPU with compute support detected.

**Solutions**:
1. Ensure your GPU supports Vulkan compute
2. Update GPU drivers
3. Check if GPU is disabled in BIOS
4. For laptops, ensure discrete GPU is active

### Error: "VkResult::ErrorOutOfHostMemory"
**Cause**: System is out of memory.

**Solutions**:
1. Close other applications
2. Reduce buffer sizes in your code
3. Enable pool allocator (should be automatic)

### Error: "Timeline semaphores not supported"
**Cause**: GPU/driver doesn't support Vulkan 1.2 or timeline semaphore extension.

**Solutions**:
1. Update GPU drivers
2. The code will fall back to regular semaphores automatically
3. Performance may be reduced without timeline batching

## Performance Issues

### Issue: Lower than expected performance
**Diagnostics**:
1. Check if optimizations are enabled:
   ```bash
   cargo build --release --features implementation
   ```
2. Verify GPU is being used (not CPU fallback)
3. Check for thermal throttling

**Solutions**:
1. Ensure release build with optimizations
2. Profile with `cargo bench`
3. Check GPU utilization with system tools
4. Verify barrier policy is working (check logs)

### Issue: High CPU usage
**Cause**: May indicate excessive synchronization.

**Solutions**:
1. Enable timeline batching (automatic with implementation feature)
2. Batch more operations together
3. Use larger workgroup sizes

## Platform-Specific Issues

### Linux

#### Issue: "Failed to load ICD"
**Causes**:
1. ICD manifest found but library resolution failed (common when `library_path` is relative and not in default linker paths)
2. No manifests found in standard paths

**Diagnostics**:
```bash
RUST_LOG=kronos_compute=info,kronos_compute::implementation::icd_loader=debug <your app>
```
Look for:
- "ICD search paths" list
- "Discovered ICD candidate" entries for each JSON
- "Attempting to load ICD library" entries per candidate
- Any per-candidate error messages

**Solutions**:
1. Ensure RADV/NVIDIA/Intel ICD JSONs exist (e.g., `/usr/share/vulkan/icd.d/radeon_icd.x86_64.json`)
2. If needed, set Vulkan ICD override:
```bash
export VK_ICD_FILENAMES=/usr/share/vulkan/icd.d/nvidia_icd.json
# Or for AMD:
export VK_ICD_FILENAMES=/usr/share/vulkan/icd.d/radeon_icd.x86_64.json
```
3. If the JSON uses a relative `library_path`, ensure the corresponding `.so` is in the system library search path (e.g., `/usr/lib/x86_64-linux-gnu/`). Kronos now tries both the as-provided name and manifest-relative path.

#### Issue: Permission denied accessing GPU
**Solution**: Add user to video/render groups:
```bash
sudo usermod -a -G video,render $USER
# Log out and back in
```

### Windows

#### Issue: "VulkanSDK not found"
**Solution**: Set environment variables:
```powershell
$env:VULKAN_SDK = "C:\VulkanSDK\1.3.268.0"
$env:Path += ";$env:VULKAN_SDK\Bin"
```

#### Issue: Missing DLLs
**Solution**: Install Visual C++ Redistributables

### macOS

#### Issue: "MoltenVK not found"
**Solution**: Install MoltenVK:
```bash
brew install molten-vk
```

#### Issue: Security warnings
**Solution**: Allow unsigned binaries in Security settings

## Debug Tips

### Enable Logging
```bash
RUST_LOG=kronos_compute=debug cargo run
```

### Validation Layers
Enable Vulkan validation for debugging:
```rust
let ctx = ComputeContext::builder()
    .enable_validation()
    .build()?;
```

### Check ICD Loading
```bash
RUST_LOG=kronos_compute=info,kronos_compute::implementation::icd_loader=debug cargo run
```

## Getting Help

If these solutions don't resolve your issue:

1. Check [existing issues](https://github.com/LynnColeArt/kronos-compute/issues)
2. Enable debug logging and capture output
3. Include system information:
   - OS and version
   - GPU model and driver version
   - Rust version
   - Vulkan SDK version
4. Create a [minimal reproducible example](https://stackoverflow.com/help/minimal-reproducible-example)
5. Open a new issue with all information

## Common Workarounds

### Disable Specific Optimizations
If you suspect an optimization is causing issues:

```rust
// Disable timeline batching
kronos::implementation::timeline_batching::set_batch_size(1)?;

// Use conservative barriers
let tracker = BarrierTracker::new(GpuVendor::Other);
```

### Force CPU Readback
For debugging compute results:
```rust
let results: Vec<f32> = buffer.read()?;
println!("First 10 results: {:?}", &results[..10]);
```
