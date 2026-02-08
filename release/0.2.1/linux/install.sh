#!/bin/bash
# AirDB Linux Installer v0.2.1

set -e

VERSION="0.2.1"
INSTALL_DIR="$HOME/.local/share/airdb"
VERSION_DIR="$INSTALL_DIR/versions/$VERSION"
BIN_DIR="$HOME/.local/bin"

echo "🚀 Installing AirDB v$VERSION"

# Create directories
mkdir -p "$VERSION_DIR"
mkdir -p "$BIN_DIR"

# Copy binaries to version directory
cp airdb-desktop "$VERSION_DIR/"
cp airdb-cli "$VERSION_DIR/"
cp airdb-bootstrap "$VERSION_DIR/"

chmod +x "$VERSION_DIR/airdb-desktop"
chmod +x "$VERSION_DIR/airdb-cli"
chmod +x "$VERSION_DIR/airdb-bootstrap"

# Create or update "current" symlink
ln -sfn "versions/$VERSION" "$INSTALL_DIR/current"

# Create symlinks in bin directory
# airdb -> CLI (for command-line usage)
# airdb-desktop -> bootstrapper (for launching GUI)
ln -sf "$VERSION_DIR/airdb-cli" "$BIN_DIR/airdb"
ln -sf "$VERSION_DIR/airdb-bootstrap" "$BIN_DIR/airdb-desktop"
ln -sf "$VERSION_DIR/airdb-bootstrap" "$BIN_DIR/airdb-bootstrap"

echo "✅ AirDB installed to $INSTALL_DIR"
echo ""

# Check if in PATH
if echo "$PATH" | grep -q "$BIN_DIR"; then
    echo "✅ $BIN_DIR is in your PATH"
    echo ""
    echo "Run 'airdb --version' to verify installation"
else
    echo "⚠️  $BIN_DIR is not in your PATH."
    echo ""
    echo "Add this line to your ~/.bashrc or ~/.zshrc:"
    echo "  export PATH=\"\$HOME/.local/bin:\$PATH\""
    echo ""
    echo "Then run: source ~/.bashrc"
    echo "Or open a new terminal"
fi

echo ""
echo "🎉 Installation complete!"
