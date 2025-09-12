#!/bin/bash

# Build script for Shell-T
# This script builds optimized release binaries for the current platform

set -e

echo "Building Shell-T release..."

# Build release version
cargo build --release

# Get the binary name based on platform
if [[ "$OSTYPE" == "linux-gnu"* ]]; then
    BINARY_NAME="shell-t-linux-x64"
    SOURCE="target/release/shell-t"
elif [[ "$OSTYPE" == "darwin"* ]]; then
    BINARY_NAME="shell-t-macos-x64"
    SOURCE="target/release/shell-t"
elif [[ "$OSTYPE" == "msys" ]] || [[ "$OSTYPE" == "win32" ]]; then
    BINARY_NAME="shell-t-windows-x64.exe"
    SOURCE="target/release/shell-t.exe"
else
    echo "Unsupported platform: $OSTYPE"
    exit 1
fi

# Copy binary to root directory with platform-specific name
cp "$SOURCE" "$BINARY_NAME"

echo "✅ Build complete!"
echo "📦 Binary created: $BINARY_NAME"
echo ""
echo "To create a GitHub release:"
echo "1. Commit your changes: git add . && git commit -m 'Release version x.x.x'"
echo "2. Create a tag: git tag v1.0.0"
echo "3. Push with tags: git push origin main --tags"
echo "4. GitHub Actions will automatically build and create the release"