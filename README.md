# Kronos Compute

Kronos is a compute-only Vulkan library with a Rust API and an ICD-aware
implementation. This repository contains an experimental prerelease, not a
qualified production compute backend. Runtime behavior depends on the Vulkan
loader, device, driver and workload.

The package manifest declares `0.2.3-rc3`; GitHub lists release
[`v0.2.3-rc3`](https://github.com/crucible-energy/kronos-compute/releases/tag/v0.2.3-rc3)
as its latest release, published on **2025-08-31**. This repository's
`Cargo.toml` still names `LynnColeArt/kronos-compute` as the package repository
and homepage. That registry/source ownership boundary needs review before a
future release; commits here do not imply a new crates.io publication.

## Start here on Linux or macOS

With Git, Rust and Cargo installed, clone the Crucible Energy repository and
compile the library:

```sh
git clone https://github.com/crucible-energy/kronos-compute.git
cd kronos-compute
cargo check --lib
```

To run the library unit tests:

```sh
cargo test --lib
```

These commands compile and test library code; they do not dispatch work to a
GPU. `Cargo.toml` declares Rust 1.70 as its minimum version. The dated
[development setup guide](docs/DEVELOPMENT_SETUP.md) records the tested macOS
toolchain and explicit Linux/GPU qualification limits; the declared minimum
version and Linux runtime behavior remain untested.

To inspect the safe API, read [Unified API](docs/UNIFIED_API.md) and
[`examples/unified_api_simple.rs`](examples/unified_api_simple.rs). Compile that
example with `cargo check --example unified_api_simple`. Running it requires a
compatible Vulkan runtime and dispatches real GPU work; hardware and
cross-vendor behavior are not qualified by the compile check.

## Current validation boundary

On macOS with Rust 1.98.1 on **2026-10-16**, `cargo check --lib`,
`cargo test --lib` (48 tests), and `cargo check --example unified_api_simple`
passed. The broader package paths remain blocked in this source snapshot:

- `cargo check --all-targets` fails while compiling the legacy
  `compute_workloads` benchmark, which imports the unresolved `kronos` crate.
- `cargo test --features implementation` fails to compile
  `tests/integration_tests.rs`, which accesses the private
  `DispatchBuilder.command_buffers` field.
- `cargo fmt --all -- --check` reports existing whitespace drift in
  `tests/unified_api_test.rs`.
- No GPU dispatch, Linux hardware run, cross-vendor qualification or benchmark
  was performed in this review. Performance results remain deferred; see the
  [benchmark notes](docs/benchmarks/BENCHMARK_RESULTS.md). The repository's
  [current issues](https://github.com/crucible-energy/kronos-compute/issues)
  are the active issue tracker; older `docs/KNOWN_ISSUES.md` entries need
  revalidation before use as current status.

These are observed local results, not a claim about every host. Check the
[current repository issues](https://github.com/crucible-energy/kronos-compute/issues)
and the [CI workflow](.github/workflows/ci.yml) before relying on a broader
build or runtime path.

## Repository map

| If you need to understand… | Start with… |
| --- | --- |
| The safe Rust API | [`src/api/`](src/api/), [Unified API](docs/UNIFIED_API.md) |
| Vulkan loading and runtime behavior | [`src/implementation/`](src/implementation/), [compatibility](docs/COMPATIBILITY.md), [troubleshooting](docs/TROUBLESHOOTING.md) |
| Platform setup notes | [development setup](docs/DEVELOPMENT_SETUP.md) |
| Historical design and validation records | [`docs/architecture/`](docs/architecture/), [`docs/qa/`](docs/qa/), [benchmark notes](docs/benchmarks/BENCHMARK_RESULTS.md) |
| Contribution and implementation policy | [AGENTS.md](AGENTS.md) |

The source tree and Cargo target list are authoritative; older project maps and
roadmap dates in the repository are historical unless their status is
reconfirmed. The crate remains Rust today. Repository guidance requires new
repo-owned backend, runtime, CLI, native tooling, adapter, evaluation and
automation implementation to use Zig; read [AGENTS.md](AGENTS.md) before
proposing code changes.

The project is dual-licensed under MIT or Apache-2.0; see
[`LICENSE-MIT`](LICENSE-MIT) and [`LICENSE-APACHE`](LICENSE-APACHE).
