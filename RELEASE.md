# Release Process for Kronos Compute

## Pre-release Checklist

### Code Quality
- [ ] All tests pass (`cargo test --all-features`)
- [ ] No clippy warnings (`cargo clippy --all-features -- -D warnings`)
- [ ] Code is formatted (`cargo fmt`)
- [ ] Examples compile (`cargo build --examples --release`)
- [ ] Benchmarks run (`cargo bench`)

### Documentation
- [ ] README.md is up to date
- [ ] CHANGELOG.md has release notes
- [ ] API documentation builds (`cargo doc --no-deps`)
- [ ] TODO.md reflects completed work

### Artifacts
- [ ] C header is up to date (`cbindgen --config cbindgen.toml --crate kronos-compute --output kronos.h`)
- [ ] Shaders build successfully (`scripts/build_shaders.sh`)
- [ ] pkg-config file is correct

### Version Bumps
- [ ] Cargo.toml version updated
- [ ] README.md installation shows new version
- [ ] kronos.pc has correct version

## Release Steps

1. **Prepare Release Branch**
   ```bash
   git checkout -b release/v0.1.0
   ```

2. **Update Version**
   - Edit `Cargo.toml` version field
   - Update version in `README.md`
   - Update version in `kronos.pc`

3. **Create Changelog Entry**
   ```bash
   echo "## v0.1.0 - $(date +%Y-%m-%d)" >> CHANGELOG.md
   ```

4. **Commit Changes**
   ```bash
   git add -A
   git commit -m "chore: prepare v0.1.0 release"
   ```

5. **Create Tag**
   ```bash
   git tag -a v0.1.0 -m "Release v0.1.0 - Initial Release

   Features:
   - Core Vulkan compute API implementation
   - ICD forwarding for Vulkan compatibility
   - Mini's 4 performance optimizations
   - AMD GPU optimization support
   - Comprehensive test suite
   - C header generation
   - SPIR-V shader build scripts"
   ```

6. **Push to GitHub**
   ```bash
   git push origin release/v0.1.0
   git push origin v0.1.0
   ```

7. **Publish to crates.io**
   ```bash
   cargo publish
   ```

8. **Create GitHub Release**
   - Go to https://github.com/LynnColeArt/kronos-compute/releases
   - Click "Draft a new release"
   - Select the v0.1.0 tag
   - Use tag message as release notes
   - Attach pre-built binaries if available

## Post-release

1. **Merge Release Branch**
   ```bash
   git checkout main
   git merge release/v0.1.0
   git push origin main
   ```

2. **Update develop**
   ```bash
   git checkout develop
   git merge main
   git push origin develop
   ```

3. **Announce Release**
   - Update project website
   - Post to relevant forums/communities
   - Tweet about the release

## Version Numbering

We follow Semantic Versioning (SemVer):
- MAJOR.MINOR.PATCH
- MAJOR: Incompatible API changes
- MINOR: New functionality, backwards compatible
- PATCH: Bug fixes, backwards compatible