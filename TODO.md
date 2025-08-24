# Kronos TODO List

## 🚨 Critical (Production Blockers)

### 1. Fix panic-prone unwrap() calls
**Impact**: Can crash the entire application in production  
**Locations**:
- `icd_loader.rs`: CString creation, mutex locks, path operations
- `instance.rs`, `device.rs`, `mod.rs`: Mutex locks
**Solution**: Replace with proper error propagation using `?`

### 2. Add safety documentation to unsafe code
**Impact**: Critical for preventing undefined behavior  
**Locations**:
- `icd_loader.rs`: Large unsafe blocks in load_icd()
- `sys/mod.rs`: unsafe impl Send/Sync
- All unsafe extern "C" functions
**Solution**: Add `// SAFETY:` comments explaining invariants

### 3. Handle missing video codec headers
**Impact**: Could cause compilation failures  
**Action**: Verify complete removal of video codec references

## ⚠️ Important (Should Have)

### 4. Replace debug prints with proper logging
**Impact**: Debug output in production is unprofessional  
**Locations**:
- `icd_loader.rs:453`: println!("Loaded ICD: {:?}", lib_path)
- `icd_loader.rs:458`: eprintln!("Failed to load ICD...")
- `mod.rs:45`: eprintln!("Warning: Failed to load Vulkan ICD...")
**Solution**: Use `log` crate with appropriate levels

### 5. Create proper error types instead of String
**Impact**: Better error handling and recovery  
**Locations**:
- `icd_loader.rs`: Functions returning Result<T, String>
- `mod.rs`: initialize_kronos returns Result<(), String>
**Solution**: Create error enum with `thiserror`

### 6. Make ICD paths platform-dependent or configurable
**Impact**: Currently Linux-only, blocks Windows/macOS  
**Current**:
```rust
const ICD_SEARCH_PATHS: &[&str] = &[
    "/usr/share/vulkan/icd.d",
    "/usr/local/share/vulkan/icd.d",
    "/etc/vulkan/icd.d",
];
```
**Solution**: Use platform-specific paths or environment variables

## 📝 Nice to Have (Code Quality)

### 7. Document compatibility guarantees and test suite
- What Vulkan compute features are supported
- How to run the test suite
- Performance characteristics

### 8. Replace magic numbers with named constants
**Examples**:
- Device name buffer: 256
- UUID size: 16
- Error code: -1000069000
- Bit shifts: << 22, << 12

### 9. Move tests from source files to tests directory
**Files**: lib.rs, sys/mod.rs, core/flags.rs, core/structs.rs

### 10. Replace manual JSON parsing with proper parser
**Location**: icd_loader.rs JSON manifest parsing  
**Solution**: Use `serde_json` or `json`

## ✅ Completed Tasks

- [x] Create full Rust port (not just FFI bindings)
- [x] Implement ICD loader with driver discovery
- [x] Add function forwarding to real drivers
- [x] Remove all graphics-only functionality
- [x] Fix thread-safety issues
- [x] Complete descriptor management functions
- [x] Implement synchronization primitives
- [x] Remove mock implementation from production code
- [x] Fix unit test compilation errors

## Future Enhancements

- [ ] Vulkan 1.1+ compute features (subgroups, etc.)
- [ ] Async/await for fence waiting
- [ ] Windows and macOS ICD loading
- [ ] Performance benchmarks against raw Vulkan
- [ ] Compute workload examples