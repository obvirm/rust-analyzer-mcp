#!/bin/bash
set -e

echo "Installing rust-analyzer-mcp..."

# Check if cargo is installed
if ! command -v cargo &> /dev/null; then
    echo "Error: Rust/Cargo is not installed"
    echo "Install from: https://rustup.rs/"
    exit 1
fi

# Build the project
echo "Building..."
cargo build --release

# Install binary
BIN_DIR="$HOME/.local/bin"
mkdir -p "$BIN_DIR"
cp target/release/rust-analyzer-mcp "$BIN_DIR/"
chmod +x "$BIN_DIR/rust-analyzer-mcp"

echo "Installation complete!"
echo "Add $BIN_DIR to your PATH if not already added"
echo ""
echo "Usage:"
echo "  rust-analyzer-mcp --help"
echo "  rust-analyzer-mcp --project-root /path/to/rust/project"