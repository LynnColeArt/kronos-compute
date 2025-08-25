#!/bin/bash
# Script to prepare a release of Kronos Compute

set -e

VERSION="${1:-}"

if [ -z "$VERSION" ]; then
    echo "Usage: $0 <version>"
    echo "Example: $0 0.1.0"
    exit 1
fi

echo "Preparing release v$VERSION..."
echo "=============================="

# Check we're on a clean working directory
if [ -n "$(git status --porcelain)" ]; then
    echo "Error: Working directory not clean. Please commit or stash changes."
    exit 1
fi

# Run pre-release checks
echo "Running tests..."
cargo test --all-features --quiet || { echo "Tests failed!"; exit 1; }

echo "Running clippy..."
cargo clippy --all-features -- -D warnings || { echo "Clippy failed!"; exit 1; }

echo "Checking formatting..."
cargo fmt -- --check || { echo "Code not formatted!"; exit 1; }

echo "Building examples..."
cargo build --examples --release --quiet || { echo "Examples failed to build!"; exit 1; }

echo "Generating C header..."
if command -v cbindgen &> /dev/null; then
    cbindgen --config cbindgen.toml --crate kronos-compute --output kronos.h
else
    echo "Warning: cbindgen not installed, skipping header generation"
fi

# Update version in files
echo "Updating version to $VERSION..."
sed -i "s/^version = \".*\"/version = \"$VERSION\"/" Cargo.toml
sed -i "s/kronos-compute = \".*\"/kronos-compute = \"$VERSION\"/" README.md
sed -i "s/Version: .*/Version: $VERSION/" kronos.pc

# Update Cargo.lock
cargo update -p kronos-compute

# Create release commit
echo "Creating release commit..."
git add -A
git commit -m "chore: release v$VERSION

- Update version in Cargo.toml
- Update installation instructions in README.md
- Update pkg-config version
- Regenerate Cargo.lock"

# Create tag
echo "Creating tag v$VERSION..."
git tag -a "v$VERSION" -m "Release v$VERSION

See CHANGELOG.md for details."

echo ""
echo "Release v$VERSION prepared successfully!"
echo ""
echo "Next steps:"
echo "1. Review the changes: git show"
echo "2. Push to GitHub: git push origin main && git push origin v$VERSION"
echo "3. Publish to crates.io: cargo publish"
echo "4. Create GitHub release at: https://github.com/LynnColeArt/kronos-compute/releases/new"