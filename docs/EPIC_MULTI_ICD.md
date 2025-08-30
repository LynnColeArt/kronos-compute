# EPIC: Multi‑ICD Selection and Orchestration

Owner: Core
Status: Draft
Target: v0.2.x (phased)

## Summary
Enable discovering multiple Vulkan ICDs, exposing them to the user, and allowing explicit selection at context creation time. Support running multiple `ComputeContext`s concurrently, each bound to a different ICD. Keep behavior in‑spec by keeping objects routed through the ICD that created them. Optionally, consider a follow‑on to aggregate enumeration across ICDs with full per‑handle dispatch (bigger refactor).

## Goals
- List all discovered ICDs with useful metadata (path, type, API version).
- Allow selecting an ICD by index or path via `api::ContextBuilder`.
- Bind `ComputeContext` to a specific ICD; co‑existence of multiple contexts.
- Keep default policy “prefer hardware”; configurable via env var.
- Provide clear diagnostics and documentation.

## Non‑Goals (for this EPIC)
- Full “combined instance” that aggregates devices from multiple ICDs and routes by handle across all entrypoints. This is a separate follow‑on due to complexity and risk.

## Checklists

### Phase 0 – Foundations (Done)
- [x] Robustly resolve ICD `library_path` (as‑provided and manifest‑relative).
- [x] Improve discovery diagnostics (paths, manifests, candidates, per‑candidate errors).
- [x] Prefer hardware ICDs by default; `KRONOS_PREFER_HARDWARE=0` to include software.
- [x] Fix sType constants used by shader/pipeline structures.
- [x] Safer `ComputeContext` Drop (destroy only on last clone).

### Phase 1 – ICD Enumeration API
- [x] Add `IcdInfo` struct: `{ library_path, manifest_path, api_version, is_software }`.
- [x] Add `implementation::icd_loader::available_icds() -> Vec<IcdInfo>`.
- [ ] Unit test: simulate discovery list and verify classification/logging.
- [x] Docs: README + Troubleshooting sections referencing enumeration API.

### Phase 2 – Builder Selection + Binding
- [x] Extend `api::ContextBuilder`:
  - [x] `prefer_icd_index(usize)`
  - [x] `prefer_icd_path<P: AsRef<Path>>(P)`
- [x] Plumb selection into context creation, binding to the chosen ICD.
- [x] Expose `ComputeContext::icd_info() -> IcdInfo` for verification.
- [ ] Integration tests: create contexts for two ICDs (e.g., RADV + llvmpipe) in one process.
- [x] Logging: “Selected ICD for context: <path>, type=<hardware|software>”.

### Phase 3 – QA Coverage
- [x] Linux: add ignored smoke tests for enumeration and selection; QA plan doc.
- [ ] Add dispatch sanity tests via safe API with selected ICD.
- [ ] Windows basic smoke: ensure discovery doesn’t regress (stub for `LoadLibrary` path TBD).
- [ ] Stress: create/destroy multiple contexts across ICDs; submit/idle/teardown cycles.

### Phase 4 – (Optional) Aggregated Enumeration Across ICDs
- [ ] Design: handle provenance routing across all entrypoints (map `Vk*` → owning ICD).
- [ ] Aggregate `vkEnumeratePhysicalDevices` across ICDs; return combined list.
- [ ] Route instance/device/queue/command calls by originating ICD.
- [ ] Concurrency & safety review; perf assessment.
- [ ] Large test pass across mixed ICD sets.

### Phase 5 – Documentation & Examples
- [ ] README: “Selecting an ICD” with `available_icds()` + builder usage.
- [ ] Troubleshooting: choosing hardware vs software, logs to expect.
- [ ] Example: select RADV by index or path; run a tiny dispatch.
- [ ] CHANGELOG updates.

## API Sketch

```rust
// Query ICDs
let icds = kronos_compute::implementation::icd_loader::available_icds();
for (i, icd) in icds.iter().enumerate() {
    println!("[{i}] {:?} hw? {} api 0x{:x}", icd.library_path, !icd.is_software, icd.api_version);
}

// Select by index
let ctx = api::ComputeContext::builder()
    .prefer_icd_index(0)
    .build()?;

// Or select by path
let ctx = api::ComputeContext::builder()
    .prefer_icd_path("/usr/lib/x86_64-linux-gnu/libvulkan_radeon.so")
    .build()?;

// Inspect selection
let chosen = ctx.icd_info();
println!("Using ICD: {}", chosen.library_path.display());
```

## Acceptance Criteria
- `available_icds()` returns accurate list (hardware/software, api_version, paths).
- Builder can select ICD by index or path; selection is respected.
- Multiple contexts against different ICDs operate correctly in one process.
- Logs clearly show selected ICD per context.
- Documentation updated; example runnable.

## Risks & Mitigations
- Windows loader parity: separate task to add `LoadLibrary`/`GetProcAddress`.
- Mis‑routing calls across ICDs: enforce “bound context” per ICD, keep routing localized.
- Aggregated enumeration complexity: punt to Phase 4 with clear design gate.

### Phase 3.5 – Loader Lifetime Safety (Follow‑up)
- [x] Replace `get_icd()` unsafe lifetime cast with safe borrowing (return `Arc<LoadedICD>`).
- [x] Update mutation sites to use replace-on-write helpers (`update_instance_functions`, `update_device_functions`).
- [x] Remove unsafe `'static` cast; document lifetime model and guarantees.
- [ ] Add unit tests covering concurrent read access and mutation points.

## Telemetry & Logging
- Keep `info` summary and `debug` per‑candidate logs.
- On context creation, log selected ICD and whether software/hardware.

## Timeline (Indicative)
- P1 + P2: 2–4 days of focused work + review.
- P3: 1–2 days across supported platforms.
- P5: 0.5–1 day docs/examples.
- P4: separate mini‑epic (1–2 weeks) if green‑lit.
