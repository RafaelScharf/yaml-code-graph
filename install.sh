#!/usr/bin/env bash
set -e

# YCG CLI Installation Script
# This script builds and installs the ycg_cli binary to your system

INSTALL_DIR="/usr/local/bin"
BINARY_NAME="ycg"
SOURCE_BINARY="./target/release/ycg_cli"

echo "🚀 Installing YCG CLI..."

# Check if cargo is available
if ! command -v cargo &> /dev/null; then
    echo "❌ Error: cargo not found"
    echo "Please install Rust from https://rustup.rs/"
    exit 1
fi

# Build the latest version
echo "🔨 Building latest version with cargo..."
cargo build --release --bin ycg_cli

# Verify binary was created
if [ ! -f "$SOURCE_BINARY" ]; then
    echo "❌ Error: Build failed - binary not found at $SOURCE_BINARY"
    exit 1
fi

# Create install directory if it doesn't exist
mkdir -p "$INSTALL_DIR"

# Copy binary
echo "📦 Copying binary to $INSTALL_DIR/$BINARY_NAME"
cp "$SOURCE_BINARY" "$INSTALL_DIR/$BINARY_NAME"
chmod +x "$INSTALL_DIR/$BINARY_NAME"

# Check if install dir is in PATH
if [[ ":$PATH:" != *":$INSTALL_DIR:"* ]]; then
    echo ""
    echo "⚠️  Warning: $INSTALL_DIR is not in your PATH"
    echo ""
    echo "Add this line to your shell configuration file:"
    echo "  ~/.zshrc (for zsh) or ~/.bashrc (for bash)"
    echo ""
    echo "  export PATH=\"\$HOME/.local/bin:\$PATH\""
    echo ""
    echo "Then run: source ~/.zshrc (or source ~/.bashrc)"
else
    echo "✅ Installation directory is already in PATH"
fi

echo ""
echo "✨ Installation complete!"
echo ""
echo "Usage:"
echo "  ycg -i index.scip -o graph.yaml"
echo "  ycg -i index.scip -o graph.yaml --compact"
echo "  ycg --help"
echo ""
