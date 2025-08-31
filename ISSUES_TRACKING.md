# Kronos Compute - Known Issues and Tracking

## Current Version: v0.2.0-rc13

### Critical Issues

#### 1. Memory Corruption with AMD Driver
**Status**: ✅ Fixed in v0.2.0-rc13  
**Priority**: High  
**First Observed**: v0.2.0-rc13  

**Symptoms**:
- When using AMD driver explicitly via path selection
- Device enumeration succeeds (returns 2 devices)
- Program crashes with "double free or corruption (fasttop)"

**Test Case**:
```bash
cargo run --example test_amd_path
```

**Output**:
```
Loaded ICD: /usr/lib/x86_64-linux-gnu/libvulkan_radeon.so
✓ Instance created
First enumeration: result=Success, count=2
Second enumeration: result=Success, got 2 devices
Device 0: AMD Radeon Graphics (RADV RAPHAEL_MENDOCINO)
Device 1: AMD Radeon RX 7900 XTX (RADV NAVI31)
double free or corruption (fasttop)
```

**Root Cause**:
- Missing Drop implementation for LoadedICD struct
- Library handles were being implicitly freed causing double-free

**Resolution**:
- Removed Drop implementation - library handles are intentionally leaked
- This is standard practice for dynamically loaded Vulkan ICDs
- Function pointers from the library may still be in use elsewhere

---

#### 2. Safe API Crashes in Path Selection Mode
**Status**: ✅ Fixed in v0.2.0-rc13  
**Priority**: High  
**First Observed**: v0.2.0-rc11  

**Symptoms**:
- ComputeContext::builder().prefer_icd_path() causes segfault
- ICD function pointers were null when accessed

**Root Cause**:
- get_icd() was returning a preferred ICD from ALL_ICDS that hadn't loaded instance functions
- Instance functions are only loaded after vkCreateInstance succeeds
- The preferred ICD had null function pointers for enumerate_physical_devices, etc.

**Resolution**:
- Modified get_icd() to always return the main ICD from ICD_LOADER
- This ensures we get the ICD with properly loaded instance/device functions
- Preferences are applied during initialization in initialize_icd_loader()

**Test Case**:
```bash
cargo run --example test_safe_api_amd
# Output: ✓ ComputeContext created successfully with AMD driver!
```

---

#### 3. Safe API Hangs in Aggregated Mode
**Status**: ✅ Fixed in v0.2.0-rc13  
**Priority**: High  
**First Observed**: v0.2.0-rc12  

**Symptoms**:
- With KRONOS_AGGREGATE_ICD=1, safe API hangs during initialization
- Appears to hang after finding devices but before creating command pool

**Root Cause**:
- Multiple issues including Arc<LoadedICD> lifetime and memory corruption
- Fixed by proper ICD lifetime management

**Test Case**:
```bash
KRONOS_AGGREGATE_ICD=1 cargo run --example test_safe_api_simple
# Now outputs: ✓ ComputeContext created successfully!
```

---

#### 4. ICD Preference Selection Limitations
**Status**: 🟡 Partially Fixed  
**Priority**: High  
**First Observed**: v0.2.0-rc13  

**Symptoms**:
- In single-ICD mode: Can only use the ICD selected during first initialization
- Index mismatch: available_icds() includes software renderers, but init filters them
- Switching ICDs after instance creation causes ErrorInitializationFailed

**Root Cause**:
- Vulkan instances are tied to the ICD that created them
- Cannot use instance from one ICD with a different ICD
- ICD preference must be set BEFORE any initialization

**Partial Fix**:
- Preferences now work dynamically in aggregated mode
- Preferences work if set before first initialization

**Limitations**:
- Single-ICD mode: Must set preference before ANY Kronos calls
- Index confusion: available_icds() shows different order than init uses
- Example: AMD is index 3 in available_icds() but index 2 during init

**Workaround**:
```bash
# For guaranteed AMD selection
export KRONOS_AGGREGATE_ICD=1  # Use aggregated mode
# OR
export VK_ICD_FILENAMES=/usr/lib/x86_64-linux-gnu/libvulkan_radeon.so
```

---

### Medium Priority Issues

#### 5. Intel Haswell Returns 0 Devices
**Status**: 🟡 Active  
**Priority**: Medium  
**First Observed**: v0.2.0-rc13  

**Symptoms**:
- Intel Haswell ICD loads successfully
- vkEnumeratePhysicalDevices returns 0 devices
- This might be expected if no Intel GPU is present

**Affected ICDs**:
- /usr/lib/x86_64-linux-gnu/libvulkan_intel_hasvk.so
- /usr/lib/x86_64-linux-gnu/libvulkan_nouveau.so
- /usr/lib/x86_64-linux-gnu/libvulkan_virtio.so
- /usr/lib/x86_64-linux-gnu/libvulkan_gfxstream.so

---

#### 6. Descriptor Pool Creation Disabled
**Status**: 🟡 Active  
**Priority**: Medium  
**First Observed**: v0.2.0-rc12  

**Symptoms**:
- Descriptor pool creation commented out in safe API
- Was causing ErrorInitializationFailed

**Location**: src/api/context.rs:123-127
```rust
// TODO: Fix descriptor pool creation - temporarily skip
log::info!("[SAFE API] Skipping descriptor pool creation temporarily");
let descriptor_pool = VkDescriptorPool::NULL;
// let descriptor_pool = Self::create_descriptor_pool(device)?;
```

---

### Fixed Issues

#### ✅ Safe API Hangs in Aggregated Mode
**Status**: Fixed in v0.2.0-rc13  
**Resolution**: Fixed Arc<LoadedICD> lifetime and memory corruption issues

#### ✅ Memory Corruption with AMD Driver
**Status**: Fixed in v0.2.0-rc13  
**Resolution**: Removed Drop implementation for LoadedICD - library handles are intentionally leaked

#### ✅ Arc<LoadedICD> Lifetime Issue
**Status**: Fixed in v0.2.0-rc13  
**Resolution**: Added DEVICE_ICDS static registry to maintain Arc references

#### ✅ VkQueueFamilyProperties Default Trait
**Status**: Fixed in v0.2.0-rc13  
**Resolution**: Manually construct struct with all fields initialized

#### ✅ Device Creation ErrorFeatureNotPresent
**Status**: Fixed in v0.2.0-rc12  
**Resolution**: Use NULL features pointer instead of default struct

---

## Testing Matrix

| ICD Driver | Low-Level API | Safe API (Single) | Safe API (Aggregated) |
|------------|---------------|-------------------|----------------------|
| Intel Haswell | ❌ 0 devices | ❌ DeviceNotFound | ✅ Works (selects AMD) |
| Intel Iris | ❓ Not tested | ❓ Not tested | ❓ Not tested |
| AMD Radeon | ✅ 2 devices | ✅ Works | ✅ Works |
| NVIDIA Nouveau | ❌ 0 devices | ❌ DeviceNotFound | ✅ Works (selects AMD) |
| LLVMpipe | ❓ Not tested | ❓ Not tested | ❓ Not tested |

---

## Next Steps

1. **Investigate Memory Corruption**
   - Add valgrind testing
   - Review string handling in device property extraction
   - Check for double-free in ICD cleanup

2. **Debug Safe API Path Selection**
   - Review PathBuf handling in preference setting
   - Check for use-after-free in string conversions

3. **Fix Aggregated Mode Hang**
   - Add timeout debugging
   - Trace exact hang location
   - Review multi-ICD instance creation

4. **Re-enable Descriptor Pool**
   - Debug why vkCreateDescriptorPool fails
   - Check if device functions are properly loaded

---

## Debugging Commands

```bash
# Valgrind memory check
valgrind --leak-check=full --track-origins=yes cargo run --example test_amd_path

# GDB with symbols
cargo build --example test_safe_api_amd
gdb target/debug/examples/test_safe_api_amd

# Trace-level logging
RUST_LOG=kronos_compute=trace cargo run --example test_safe_api_simple

# Aggregated mode with timeout
timeout 5 bash -c "KRONOS_AGGREGATE_ICD=1 cargo run --example test_safe_api_simple"
```