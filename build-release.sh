#!/bin/bash

# Build script for Shell-T
# This script builds optimized release binaries for the current platform
# For comprehensive cross-platform builds, use GitHub Actions (see .github/workflows/release.yml)

set -e

echo "Building Shell-T release..."
echo "Note: For full cross-platform builds (macOS, Linux, Windows with ARM64 support),"
echo "use GitHub Actions by creating a release tag."
echo ""

# Build release version
cargo build --release

# Get the binary name based on platform
if [[ "$OSTYPE" == "linux-gnu"* ]]; then
    BINARY_NAME="shell-t-linux-x64-gnu"
    SOURCE="target/release/shell-t"
elif [[ "$OSTYPE" == "darwin"* ]]; then
    # Check if we're on Apple Silicon
    if [[ $(uname -m) == "arm64" ]]; then
        BINARY_NAME="shell-t-macos-arm64"
    else
        BINARY_NAME="shell-t-macos-x64"
    fi
    SOURCE="target/release/shell-t"
elif [[ "$OSTYPE" == "msys" ]] || [[ "$OSTYPE" == "win32" ]]; then
    BINARY_NAME="shell-t-windows-x64-msvc.exe"
    SOURCE="target/release/shell-t.exe"
else
    echo "Unsupported platform: $OSTYPE"
    exit 1
fi

# Copy binary to root directory with platform-specific name
cp "$SOURCE" "$BINARY_NAME"

echo "✅ Local build complete!"
echo "📦 Binary created: $BINARY_NAME"
echo ""
echo "🚀 For comprehensive releases with all platforms:"
echo "1. Commit your changes: git add . && git commit -m 'Release version x.x.x'"
echo "2. Create a tag: git tag v1.0.0"
echo "3. Push with tags: git push origin main --tags"
echo "4. GitHub Actions will automatically build for:"
echo "   • macOS (Intel + Apple Silicon)"
echo "   • Linux (x86_64 + ARM64, GNU + musl)"
echo "   • Windows (x86_64 + ARM64, MSVC + GNU)"
echo "   • And create a GitHub release with all binaries"