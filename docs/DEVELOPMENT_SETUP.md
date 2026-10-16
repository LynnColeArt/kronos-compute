# Development setup

Last checked: 2026-10-16. This guide prioritizes Linux and macOS. The source
checks below were run on macOS; no Linux runtime or GPU dispatch was tested.

## Check the Rust library

The package manifest declares Rust 1.70 as its minimum. This review used Rust
1.98.1; the declared minimum version was not tested. In a checkout, run:

```sh
cargo check --lib
cargo test --lib
cargo check --example unified_api_simple
```

On macOS, these commands passed during this review. The library tests do not
dispatch GPU work, and checking the example compiles it without running it.

The broader commands are currently blocked in this source snapshot:

- `cargo check --all-targets` fails in `benches/compute_workloads.rs` because
  the benchmark imports an unresolved `kronos` crate.
- `cargo test --features implementation` fails to compile
  `tests/integration_tests.rs` because it accesses private
  `DispatchBuilder.command_buffers`.
- `cargo fmt --all -- --check` reports whitespace drift in
  `tests/unified_api_test.rs`.

These are dated local observations, not a claim about every host. Check the
[current repository issues](https://github.com/crucible-energy/kronos-compute/issues)
and [CI workflow](../.github/workflows/ci.yml) for later changes.

## Configure a Vulkan runtime

Compilation of the Rust library does not prove that a Vulkan device is
available. Running compute work needs a compatible Vulkan loader, an installed
device driver (ICD), and a device supported by that driver.

- **Linux:** follow [LunarG's current Linux SDK guide](https://vulkan.lunarg.com/doc/view/latest/linux/getting_started.html)
  for supported distributions and installation steps. A Vulkan ICD is needed
  to run Vulkan applications; the SDK alone does not establish device support.
- **macOS:** follow [LunarG's current macOS SDK guide](https://vulkan.lunarg.com/doc/view/latest/mac/getting_started.html).
  macOS Vulkan execution uses MoltenVK, which translates Vulkan calls to
  Apple's Metal API; see the [Khronos Vulkan development guide](https://docs.vulkan.org/tutorial/latest/02_Development_environment.html#macos).

Package versions and device support vary by OS, driver and hardware. The links
above are the installation authorities; no OS-wide or cross-vendor support is
claimed here.

## Run the safe API example

After configuring a compatible Vulkan runtime and device, the example can be
run with:

```sh
cargo run --example unified_api_simple
```

It creates a compute context, dispatches a shader and reads results. This review
only compiled the example; it did not execute GPU work. Start with
[Unified API](UNIFIED_API.md), then see [troubleshooting](TROUBLESHOOTING.md)
and [compatibility notes](COMPATIBILITY.md) for implementation boundaries.

## Repository policy

The crate remains Rust today. Read [AGENTS.md](../AGENTS.md) before changing
implementation code: new repo-owned backend, runtime, CLI, native tooling,
adapter, evaluation and automation work is required to use Zig unless Sam
explicitly approves an exception for the exact change. There is no separate
`CONTRIBUTING.md`; the repository guidance is the contribution policy.
