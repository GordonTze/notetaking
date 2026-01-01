# Build & Deployment Guide

## Prerequisites

### Required Software
- **Rust**: Version 1.70 or higher
- **Cargo**: Comes with Rust
- **Git**: For version control (optional)

### System Requirements
- **OS**: Linux, macOS, or Windows
- **RAM**: 2GB minimum for compilation
- **Disk Space**: 500MB for build artifacts

## Installing Rust

### Linux/macOS
```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source $HOME/.cargo/env
```

### Windows
Download and run: https://rustup.rs/

### Verify Installation
```bash
rustc --version
cargo --version
```

## Building the Application

### Development Build
```bash
cd notetaking-app
cargo build
```

Binary location: `target/debug/notetaking-app`

### Release Build (Recommended)
```bash
cargo build --release
```

Binary location: `target/release/notetaking-app`

**Release build is 10-100x faster and much smaller!**

### Build with Maximum Optimization
Add to `Cargo.toml`:
```toml
[profile.release]
opt-level = 3           # Maximum optimization
lto = true             # Link-time optimization
codegen-units = 1      # Better optimization
strip = true           # Remove debug symbols
panic = 'abort'        # Smaller binary
```

Then build:
```bash
cargo build --release
```

## Running the Application

### From Source
```bash
cargo run --release
```

### From Binary
```bash
./target/release/notetaking-app
```

### Make Binary Executable (Linux/macOS)
```bash
chmod +x target/release/notetaking-app
```

## Testing

### Run All Tests
```bash
cargo test
```

### Run Specific Test
```bash
cargo test test_name
```

### Run with Output
```bash
cargo test -- --nocapture
```

## Platform-Specific Builds

### Linux

#### Ubuntu/Debian Dependencies
```bash
sudo apt-get install build-essential pkg-config libssl-dev
```

#### Arch Linux Dependencies
```bash
sudo pacman -S base-devel openssl
```

#### Build
```bash
cargo build --release
```

### macOS

#### Install Xcode Command Line Tools
```bash
xcode-select --install
```

#### Build
```bash
cargo build --release
```

#### Create macOS App Bundle (Optional)
```bash
mkdir -p NoteTakingApp.app/Contents/MacOS
cp target/release/notetaking-app NoteTakingApp.app/Contents/MacOS/
```

### Windows

#### Build
```bash
cargo build --release
```

#### Result
- Binary: `target/release/notetaking-app.exe`
- Run directly or create shortcut

## Cross-Compilation

### Linux to Windows
```bash
# Install target
rustup target add x86_64-pc-windows-gnu

# Install mingw
sudo apt-get install mingw-w64

# Build
cargo build --release --target x86_64-pc-windows-gnu
```

### Linux to macOS
Requires macOS SDK (more complex, use CI/CD)

## Optimizing Binary Size

### 1. Strip Debug Symbols
```bash
strip target/release/notetaking-app
```

### 2. Use UPX Compression
```bash
# Install UPX
sudo apt-get install upx

# Compress
upx --best --lzma target/release/notetaking-app
```

### 3. Optimize Cargo.toml
```toml
[profile.release]
opt-level = 'z'     # Optimize for size
lto = true
codegen-units = 1
strip = true
panic = 'abort'
```

## Continuous Integration

### GitHub Actions Example

`.github/workflows/build.yml`:
```yaml
name: Build

on: [push, pull_request]

jobs:
  build:
    runs-on: ${{ matrix.os }}
    strategy:
      matrix:
        os: [ubuntu-latest, macos-latest, windows-latest]
    
    steps:
    - uses: actions/checkout@v2
    
    - name: Install Rust
      uses: actions-rs/toolchain@v1
      with:
        toolchain: stable
        override: true
    
    - name: Build
      run: cargo build --release
    
    - name: Test
      run: cargo test --release
    
    - name: Upload Binary
      uses: actions/upload-artifact@v2
      with:
        name: notetaking-app-${{ matrix.os }}
        path: target/release/notetaking-app*
```

## Deployment Options

### Option 1: Single Binary Distribution

**Advantages**:
- Simple
- No installation required
- Cross-platform

**Steps**:
1. Build release binary
2. Distribute binary file
3. Users run directly

### Option 2: Package Managers

#### Homebrew (macOS)
Create formula:
```ruby
class NotetakingApp < Formula
  desc "Fast notetaking application"
  homepage "https://github.com/yourusername/notetaking-app"
  url "https://github.com/yourusername/notetaking-app/archive/v1.0.0.tar.gz"
  
  def install
    system "cargo", "build", "--release"
    bin.install "target/release/notetaking-app"
  end
end
```

#### Cargo Install
Publish to crates.io:
```bash
cargo publish
```

Users install:
```bash
cargo install notetaking-app
```

#### Linux Package (.deb)

Install `cargo-deb`:
```bash
cargo install cargo-deb
```

Add to `Cargo.toml`:
```toml
[package.metadata.deb]
maintainer = "Your Name <email@example.com>"
copyright = "2025, Your Name"
extended-description = "A fast notetaking application"
depends = "$auto"
section = "utility"
priority = "optional"
assets = [
    ["target/release/notetaking-app", "usr/bin/", "755"],
    ["README.md", "usr/share/doc/notetaking-app/", "644"],
]
```

Build:
```bash
cargo deb
```

### Option 3: Container Distribution

#### Dockerfile
```dockerfile
FROM rust:1.75 as builder
WORKDIR /app
COPY . .
RUN cargo build --release

FROM debian:bookworm-slim
COPY --from=builder /app/target/release/notetaking-app /usr/local/bin/
RUN apt-get update && apt-get install -y libssl3 ca-certificates
CMD ["notetaking-app"]
```

Build and run:
```bash
docker build -t notetaking-app .
docker run -v $(pwd)/notes_data:/app/notes_data notetaking-app
```

## Version Management

### Semantic Versioning
Update version in `Cargo.toml`:
```toml
[package]
version = "1.2.3"  # MAJOR.MINOR.PATCH
```

### Git Tags
```bash
git tag -a v1.0.0 -m "Release version 1.0.0"
git push origin v1.0.0
```

## Release Checklist

- [ ] Update version in Cargo.toml
- [ ] Update CHANGELOG.md
- [ ] Run all tests: `cargo test`
- [ ] Build release: `cargo build --release`
- [ ] Test binary manually
- [ ] Create git tag
- [ ] Build for all platforms
- [ ] Create release notes
- [ ] Upload binaries to GitHub releases
- [ ] Announce release

## Troubleshooting Build Issues

### Linker Errors on Linux
```bash
sudo apt-get install build-essential
```

### OpenSSL Errors
```bash
# Linux
sudo apt-get install pkg-config libssl-dev

# macOS
brew install openssl
```

### Permission Denied
```bash
chmod +x target/release/notetaking-app
```

### Out of Memory During Build
```bash
# Reduce parallel jobs
cargo build --release -j 1
```

### Slow Compilation
```bash
# Use faster linker (Linux)
sudo apt-get install lld clang
export RUSTFLAGS="-C link-arg=-fuse-ld=lld"
cargo build --release
```

## Performance Benchmarking

### Compile Time
```bash
cargo clean
time cargo build --release
```

### Binary Size
```bash
ls -lh target/release/notetaking-app
```

### Runtime Profiling
```bash
# Install cargo-flamegraph
cargo install flamegraph

# Profile
sudo cargo flamegraph
```

## Distribution Checklist

### For Users
- [ ] Binary executable
- [ ] README.md
- [ ] QUICKSTART.md
- [ ] LICENSE file
- [ ] Example notes (optional)

### For Developers
- [ ] Source code
- [ ] Cargo.toml
- [ ] Build instructions
- [ ] ARCHITECTURE.md
- [ ] Contributing guidelines

## Update Process

### For Users
1. Download new binary
2. Replace old binary
3. Restart application
4. Data automatically compatible

### For Developers
1. Pull latest code
2. Run `cargo update`
3. Rebuild: `cargo build --release`

## Maintenance

### Keep Dependencies Updated
```bash
# Check outdated dependencies
cargo outdated

# Update dependencies
cargo update

# Update Rust
rustup update
```

### Security Audits
```bash
# Install cargo-audit
cargo install cargo-audit

# Run audit
cargo audit
```

## Support

### Build Issues
1. Check Rust version: `rustc --version`
2. Update Rust: `rustup update`
3. Clean build: `cargo clean && cargo build --release`
4. Check system dependencies

### Runtime Issues
1. Check binary permissions
2. Verify `notes_data` directory exists
3. Check disk space
4. Review error messages in terminal

## Additional Resources

- Rust Book: https://doc.rust-lang.org/book/
- Cargo Guide: https://doc.rust-lang.org/cargo/
- Cross-compilation: https://rust-lang.github.io/rustup/cross-compilation.html
- egui documentation: https://docs.rs/egui/

## Quick Reference

### Essential Commands
```bash
# Build
cargo build --release

# Run
cargo run --release

# Test
cargo test

# Clean
cargo clean

# Update dependencies
cargo update

# Check for errors
cargo check
```

### File Locations
- Source: `src/`
- Binary: `target/release/notetaking-app`
- Data: `notes_data/`
- Backup: `notes_data_cloud_sync/`

This guide covers everything you need to build, deploy, and maintain the notetaking application!
