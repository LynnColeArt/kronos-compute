# Development Environment Setup

This guide helps you set up a development environment for Kronos Compute.

## Prerequisites

### 1. Rust Toolchain
Kronos Compute requires Rust 1.70 or later.

```bash
# Install Rust via rustup
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# Update to latest stable
rustup update stable

# Verify version (should be 1.70+)
rustc --version
```

### 2. Vulkan SDK
The Vulkan SDK provides the ICD loader and validation layers.

#### Linux (Ubuntu/Debian)
```bash
wget -qO- https://packages.lunarg.com/lunarg-signing-key-pub.asc | sudo tee /etc/apt/trusted.gpg.d/lunarg.asc
sudo wget -qO /etc/apt/sources.list.d/lunarg-vulkan-jammy.list https://packages.lunarg.com/vulkan/lunarg-vulkan-jammy.list
sudo apt update
sudo apt install vulkan-sdk
```

#### Linux (Fedora)
```bash
sudo dnf install vulkan-devel vulkan-tools
```

#### Windows
Download and install from [LunarG](https://vulkan.lunarg.com/sdk/home#windows).

#### macOS
```bash
brew install vulkan-sdk molten-vk
```

### 3. GPU Drivers
Ensure you have up-to-date GPU drivers with Vulkan support:

- **NVIDIA**: Download from [nvidia.com](https://www.nvidia.com/drivers)
- **AMD**: Download from [amd.com](https://www.amd.com/support)
- **Intel**: Usually included with OS updates

### 4. Build Tools

#### Linux
```bash
sudo apt install build-essential pkg-config cmake
```

#### Windows
Install Visual Studio 2019+ with C++ workload.

#### macOS
```bash
xcode-select --install
```

### 5. SPIR-V Compiler (Optional)
For building shaders:

```bash
# Option 1: Install with Vulkan SDK (includes glslc)
# Option 2: Install standalone
cargo install shaderc
```

## Building Kronos Compute

### 1. Clone the Repository
```bash
git clone https://github.com/LynnColeArt/kronos-compute
cd kronos-compute
```

### 2. Build Shaders (if modifying)
```bash
./scripts/build_shaders.sh
```

### 3. Build the Project
```bash
# Debug build
cargo build --features implementation

# Release build with optimizations
cargo build --release --features implementation

# Build and run tests
cargo test --features implementation

# Build with all features
cargo build --all-features
```

## Development Workflow

### Running Examples
```bash
# Run the simple compute example
cargo run --example compute_simple --features implementation

# Run the unified API example
cargo run --example unified_api_simple --features implementation

# Run a specific binary
cargo run --bin demo --features implementation
```

### Testing
```bash
# Run all tests
cargo test --features implementation

# Run specific test module
cargo test --features implementation barrier_tests

# Run with debug output
RUST_LOG=kronos_compute=debug cargo test --features implementation

# Run benchmarks
cargo bench --features implementation
```

### Code Quality
```bash
# Format code
cargo fmt

# Run linter
cargo clippy --features implementation

# Check for security issues
cargo audit

# Generate documentation
cargo doc --features implementation --open
```

## Environment Variables

### Required for Development
```bash
# Enable debug logging
export RUST_LOG=kronos_compute=debug

# Custom ICD paths (if needed)
export KRONOS_ICD_SEARCH_PATHS=/path/to/icd/files
export VK_ICD_FILENAMES=/path/to/specific.json
```

### Optional Performance Tuning
```bash
# Increase timeline batch size
export KRONOS_BATCH_SIZE=64

# Customize memory pool size (bytes)
export KRONOS_SLAB_SIZE=536870912  # 512MB
```

## IDE Setup

### VS Code
Install recommended extensions:
- rust-analyzer
- CodeLLDB (for debugging)
- Even Better TOML

Settings (`.vscode/settings.json`):
```json
{
    "rust-analyzer.cargo.features": ["implementation"],
    "rust-analyzer.checkOnSave.command": "clippy"
}
```

### IntelliJ IDEA / CLion
- Install Rust plugin
- Enable "Use clippy instead of cargo check"
- Set cargo features: implementation

## Troubleshooting Development Issues

### 1. Vulkan Not Found
```bash
# Verify Vulkan installation
vulkaninfo

# Check library path
ldconfig -p | grep vulkan
```

### 2. Shader Compilation Fails
```bash
# Verify SPIR-V compiler
which glslc
glslc --version

# Use alternative compiler
which glslangValidator
```

### 3. Tests Fail with "No device found"
- Ensure GPU drivers are installed
- Check if running in VM/container (may need to pass through GPU)
- Try with software renderer: `export VK_ICD_FILENAMES=/usr/share/vulkan/icd.d/lvp_icd.x86_64.json`

### 4. Performance Issues During Development
```bash
# Use release mode for performance testing
cargo build --release --features implementation

# Profile with flamegraph
cargo install flamegraph
cargo flamegraph --features implementation --bin demo
```

## Contributing

1. Fork the repository
2. Create a feature branch: `git checkout -b feature/amazing-feature`
3. Make changes and test thoroughly
4. Run `cargo fmt` and `cargo clippy`
5. Commit with descriptive messages
6. Push and create a pull request

See [CONTRIBUTING.md](../CONTRIBUTING.md) for detailed guidelines.