# QA Plan: Multi‑ICD Enumeration and Selection

This document defines validation steps for the Multi‑ICD epic (Phase 3).

## Goals
- Verify ICD enumeration returns accurate metadata.
- Verify explicit ICD selection via builder hooks works.
- Verify process binds to the selected ICD and logs reflect it.

## Prerequisites
- Linux with Vulkan drivers installed.
- At least one hardware ICD (e.g., RADV) and optionally a software ICD (llvmpipe).
- Environment variable to enable ignored tests:
  - `KRONOS_RUN_ICD_TESTS=1`

## Steps

1) Enumerate ICDs
```bash
RUST_LOG=kronos_compute=info,kronos_compute::implementation::icd_loader=debug \
KRONOS_RUN_ICD_TESTS=1 \
cargo test --test icd_selection -- --ignored --nocapture enumerate_icds
```
Expect:
- Log of search paths and discovered manifests.
- Printed list of ICDs with `library_path`, `(hardware|software)`, and `api`.

2) Select first ICD and initialize
```bash
RUST_LOG=kronos_compute=info,kronos_compute::implementation::icd_loader=debug \
KRONOS_RUN_ICD_TESTS=1 \
cargo test --test icd_selection -- --ignored --nocapture select_first_icd_and_init
```
Expect:
- Successful initialization.
- Selected ICD info printed.
- Info log: "ComputeContext bound to ICD: … (hardware|software), api=…" when creating a context via the safe API.

3) Manual safe API check (optional)
```rust
use kronos_compute::api;
use kronos_compute::implementation::icd_loader;

let list = icd_loader::available_icds();
let idx = 0; // pick a hardware ICD if present
let ctx = api::ComputeContext::builder()
    .prefer_icd_index(idx)
    .build()?;
assert!(ctx.icd_info().is_some());
```

## Notes
- The current implementation binds a single ICD per process; switching ICDs requires a new process.
- Default policy prefers hardware ICDs; set `KRONOS_PREFER_HARDWARE=0` to include software in selection.

