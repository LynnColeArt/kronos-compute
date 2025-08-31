# Kronos Compute v0.2.0-rc2 Release Notes

## Overview

This release fixes a critical issue with multi-ICD enumeration in aggregated mode, addressing the bug reported by sporkle claude where only llvmpipe was being discovered.

## What's Fixed

### Multi-ICD Enumeration
- **Root Cause**: The `initialize_icd_loader()` function was only storing the best selected ICD in `ALL_ICDS`, not all discovered ICDs
- **Fix**: Now stores all discovered ICDs in `ALL_ICDS` before selecting the best one
- **Impact**: Aggregated mode (`KRONOS_AGGREGATE_ICD=1`) now properly discovers all available hardware ICDs

### Technical Details
```rust
// Before: Only stored the selected ICD
*ALL_ICDS.lock()? = vec![Arc::new(best_icd.clone())];

// After: Stores all discovered ICDs
let mut all_icds_vec = Vec::new();
for (icd, _, _) in &loaded_icds {
    all_icds_vec.push(Arc::new(icd.clone()));
}
*ALL_ICDS.lock()? = all_icds_vec;
```

## Testing Results

- ✅ Successfully discovers and stores all 6 hardware ICDs
- ✅ Filters out software renderers (llvmpipe) when `KRONOS_PREFER_HARDWARE=1` (default)
- ✅ Aggregated mode creates meta-instances correctly
- ✅ All 48 unit tests pass

## Usage

To use aggregated mode:
```bash
export KRONOS_AGGREGATE_ICD=1
./your_application
```

## Links

- GitHub Release: https://github.com/LynnColeArt/kronos-compute/releases/tag/v0.2.0-rc2
- Crates.io: https://crates.io/crates/kronos-compute/0.2.0-rc2
- Diff: https://github.com/LynnColeArt/kronos-compute/compare/v0.2.0-rc1...v0.2.0-rc2

## Next Steps

This fix enables proper multi-GPU support in Kronos Compute. Users with multiple GPUs should now be able to enumerate all devices when using aggregated mode.