#!/bin/bash
# AirDB Uninstallation Script

set -e

RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m'

echo -e "${YELLOW}AirDB Uninstaller${NC}"
echo ""

# Determine paths based on privileges
if [[ $EUID -eq 0 ]]; then
    INSTALL_DIR="/opt/airdb"
    BIN_DIR="/usr/local/bin"
    DESKTOP_DIR="/usr/share/applications"
else
    INSTALL_DIR="$HOME/.local/share/airdb"
    BIN_DIR="$HOME/.local/bin"
    DESKTOP_DIR="$HOME/.local/share/applications"
fi

echo "This will remove AirDB from:"
echo "  - $INSTALL_DIR"
echo "  - $BIN_DIR/airdb*"
echo "  - $DESKTOP_DIR/airdb.desktop"
echo ""

read -p "Continue? [y/N] " -n 1 -r
echo ""

if [[ ! $REPLY =~ ^[Yy]$ ]]; then
    echo "Aborted."
    exit 0
fi

echo ""
echo "Removing symlinks..."
rm -f "$BIN_DIR/airdb" 2>/dev/null || true
rm -f "$BIN_DIR/airdb-desktop" 2>/dev/null || true
rm -f "$BIN_DIR/airdb-bootstrap" 2>/dev/null || true

echo "Removing desktop entry..."
rm -f "$DESKTOP_DIR/airdb.desktop" 2>/dev/null || true

echo "Removing installation directory..."
rm -rf "$INSTALL_DIR" 2>/dev/null || true

echo ""
echo -e "${GREEN}✓ AirDB uninstalled successfully${NC}"
