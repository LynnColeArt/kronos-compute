# README orientation and evidence refresh

Owner: Sam (product owner); implementation by Codex under the organization README mission.
Branch: `docs/readme-orientation`.
Baseline: `a66faa55c459615fc11de7a293f3e9d3cde78265`.
Scope: `README.md`, `docs/DEVELOPMENT_SETUP.md`, and this independently owned log.

History: [ICD loader implementation record](2026-10-13-windows-icd-loader.md)
and the [repository guidance](../../AGENTS.md).

## User Experience Findings

- The 476-line README repeated API, design, validation and historical roadmap
  material while omitting a short starting path for Linux or macOS users.
- It linked cloning, CI and issues to a personal GitHub repository, linked to a
  missing `CONTRIBUTING.md`, and showed a `kronos/` tree that is not this
  repository's source layout.
- The page presented a 2025 release candidate and roadmap as current without
  dates, and did not distinguish library unit-test success from full test,
  benchmark or GPU-runtime qualification.
- Its linked setup guide cloned from the old personal-repository URL, put
  Windows ahead of macOS, and recommended full test commands that fail in the
  current source tree.

## Engineering Decisions

- Replace the duplicate catalog with a concise path to the actual crate,
  representative safe-API example, current docs, policy and source layout.
- Date the latest repository release claim and describe the Cargo package URL
  mismatch without changing packaging metadata or implying a new publication.
- Put Linux and macOS first. Report only local macOS evidence; do not imply
  Linux hardware or cross-vendor qualification.
- Replace copied OS package instructions with current official LunarG Linux and
  macOS SDK references; the macOS guide notes MoltenVK's Metal translation
  boundary. Point source cloning at the organization repo and align setup tests
  with the successful library-only path.
- Preserve the observed compile blockers for the full target/test commands and
  the deferred benchmark boundary. Keep this as documentation work; do not
  broaden the Rust implementation under the organization Zig policy.

## Known Limitations

- On macOS with Rust 1.98.1, `cargo check --lib`, `cargo test --lib` (48 passed),
  and `cargo check --example unified_api_simple` passed. The declared Rust 1.70
  minimum was not tested.
- `cargo check --all-targets` fails in the legacy benchmark target on unresolved
  `kronos` imports. `cargo test --features implementation` fails to compile an
  integration test that accesses private `DispatchBuilder.command_buffers`.
- `cargo fmt --all -- --check` reports existing whitespace drift in
  `tests/unified_api_test.rs`.
- The official SDK pages are the source for current installer requirements;
  no local Linux runtime or GPU qualification was performed.
- No GPU dispatch, Linux hardware run, Windows run or performance benchmark was
  performed. `Cargo.toml` still identifies the package repository as
  `LynnColeArt/kronos-compute`; source and registry ownership/release alignment
  remains for the product owner to resolve.
