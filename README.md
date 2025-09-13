# Shell-T

[![Rust](https://img.shields.io/badge/Rust-1.70%2B-orange)](https://www.rust-lang.org/)
[![License](https://img.shields.io/badge/License-MIT-blue.svg)](LICENSE)

A high-performance terminal shell built in Rust with multi-language script execution support.

## Features

- **Multi-Language Support**: Execute Python, Ruby, and JavaScript files directly
- **Command Pipelines**: Chain commands with `|` operator
- **I/O Redirection**: Input/output redirection with `<`, `>`, `>>`
- **Background Jobs**: Run commands asynchronously with `&`
- **Built-in Commands**: `cd`, `pwd`, `exit`, `help`
- **Colored Interface**: Enhanced terminal experience

## Installation

### Option 1: Download Pre-built Binary (Recommended)

Download the latest release from [GitHub Releases](https://github.com/sonofnos/shell-t/releases):

#### macOS

- **Intel (x86_64)**: `shell-t-macos-x64`
- **Apple Silicon (ARM64)**: `shell-t-macos-arm64`

#### Linux

- **GNU (x86_64)**: `shell-t-linux-x64-gnu`
- **GNU (ARM64)**: `shell-t-linux-arm64-gnu`
- **musl (x86_64)**: `shell-t-linux-x64-musl` *(static linking)*
- **musl (ARM64)**: `shell-t-linux-arm64-musl` *(static linking)*

#### Windows

- **MSVC (x86_64)**: `shell-t-windows-x64-msvc.exe`
- **MSVC (ARM64)**: `shell-t-windows-arm64-msvc.exe`
- **GNU (x86_64)**: `shell-t-windows-x64-gnu.exe`

```bash
# Make executable and move to PATH (macOS/Linux)
chmod +x shell-t-macos-x64  # or shell-t-linux-x64-gnu, etc.
sudo mv shell-t-macos-x64 /usr/local/bin/shell-t

# Windows: Just run the .exe file
shell-t-windows-x64-msvc.exe
```

### Option 2: Build from Source

```bash
# Prerequisites: Rust 1.70+
git clone <repository-url>
cd shell-t
cargo build --release
```

## Releases

### Creating a Release

To create a downloadable release:

1. **Local Build** (optional):

   ```bash
   ./build-release.sh
   ```

2. **GitHub Release**:

   ```bash
   git add .
   git commit -m "Release version x.x.x"
   git tag v1.0.0
   git push origin main --tags
   ```

3. **Automated Build**: GitHub Actions will automatically:
   - Build binaries for macOS, Linux, and Windows
   - Create a GitHub release
   - Upload binaries as release assets

### Release Assets

Each release includes binaries for multiple platforms and architectures:

**macOS:**
- `shell-t-macos-x64` - Intel (x86_64)
- `shell-t-macos-arm64` - Apple Silicon (ARM64)

**Linux:**
- `shell-t-linux-x64-gnu` - GNU libc (x86_64)
- `shell-t-linux-arm64-gnu` - GNU libc (ARM64)
- `shell-t-linux-x64-musl` - musl libc (x86_64, static)
- `shell-t-linux-arm64-musl` - musl libc (ARM64, static)

**Windows:**
- `shell-t-windows-x64-msvc.exe` - MSVC (x86_64)
- `shell-t-windows-arm64-msvc.exe` - MSVC (ARM64)
- `shell-t-windows-x64-gnu.exe` - GNU (x86_64)

## Usage

```bash
# Start the shell
./target/release/shell-t

# Basic commands
pwd                    # Print working directory
cd <directory>         # Change directory
ls                     # List files
exit                   # Exit shell

# Multi-language execution
./script.py           # Run Python script
./script.rb           # Run Ruby script
./script.js           # Run JavaScript file


# Pipelines and redirection
ls | grep txt         # Pipeline
echo "hello" > file.txt  # Output redirection
sort < input.txt      # Input redirection

# Background jobs
sleep 10 &            # Run in background
```

## License

MIT License
