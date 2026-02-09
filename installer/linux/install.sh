#!/bin/bash
# AirDB Installation Script
# This script installs AirDB GUI and CLI tools

set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

print_banner() {
    echo -e "${BLUE}"
    echo "╔══════════════════════════════════════════╗"
    echo "║          AirDB Installer v0.2.6          ║"
    echo "║   Local-First, GitHub-Backed Database    ║"
    echo "╚══════════════════════════════════════════╝"
    echo -e "${NC}"
}

print_success() { echo -e "${GREEN}✓ $1${NC}"; }
print_warning() { echo -e "${YELLOW}⚠ $1${NC}"; }
print_error() { echo -e "${RED}✗ $1${NC}"; }
print_info() { echo -e "${BLUE}→ $1${NC}"; }

# Get the directory where this script is located
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"

# Check if binaries exist
check_binaries() {
    local missing=0
    for bin in airdb airdb-desktop airdb-bootstrap; do
        if [[ ! -f "$SCRIPT_DIR/bin/$bin" ]]; then
            print_error "Missing binary: bin/$bin"
            missing=1
        fi
    done
    if [[ $missing -eq 1 ]]; then
        print_error "Please ensure all binaries are present in the bin/ directory"
        exit 1
    fi
}

# Determine installation paths based on privileges
set_install_paths() {
    if [[ $EUID -eq 0 ]]; then
        # Running as root - system-wide installation
        INSTALL_DIR="/opt/airdb"
        BIN_DIR="/usr/local/bin"
        DESKTOP_DIR="/usr/share/applications"
        INSTALL_MODE="system"
        print_info "Installing system-wide (root)"
    else
        # Running as user - user installation
        INSTALL_DIR="$HOME/.local/share/airdb"
        BIN_DIR="$HOME/.local/bin"
        DESKTOP_DIR="$HOME/.local/share/applications"
        INSTALL_MODE="user"
        print_info "Installing for current user"
    fi
}

# Create necessary directories
create_directories() {
    print_info "Creating directories..."
    mkdir -p "$INSTALL_DIR/bin"
    mkdir -p "$INSTALL_DIR/versions/current"
    mkdir -p "$BIN_DIR"
    mkdir -p "$DESKTOP_DIR"
    print_success "Directories created"
}

# Copy binaries
install_binaries() {
    print_info "Installing binaries..."
    
    # Copy all binaries to install directory
    cp "$SCRIPT_DIR/bin/airdb" "$INSTALL_DIR/bin/"
    cp "$SCRIPT_DIR/bin/airdb-desktop" "$INSTALL_DIR/bin/"
    cp "$SCRIPT_DIR/bin/airdb-bootstrap" "$INSTALL_DIR/bin/"
    
    # Set executable permissions
    chmod +x "$INSTALL_DIR/bin/airdb"
    chmod +x "$INSTALL_DIR/bin/airdb-desktop"
    chmod +x "$INSTALL_DIR/bin/airdb-bootstrap"
    
    # Also copy to versions/current for the updater
    cp "$SCRIPT_DIR/bin/airdb" "$INSTALL_DIR/versions/current/airdb-cli"
    cp "$SCRIPT_DIR/bin/airdb-desktop" "$INSTALL_DIR/versions/current/"
    cp "$SCRIPT_DIR/bin/airdb-bootstrap" "$INSTALL_DIR/versions/current/"
    chmod +x "$INSTALL_DIR/versions/current/"*
    
    print_success "Binaries installed to $INSTALL_DIR/bin/"
}

# Create symlinks in PATH
create_symlinks() {
    print_info "Creating symlinks in $BIN_DIR..."
    
    # Remove existing symlinks if they exist
    rm -f "$BIN_DIR/airdb" 2>/dev/null || true
    rm -f "$BIN_DIR/airdb-desktop" 2>/dev/null || true
    rm -f "$BIN_DIR/airdb-bootstrap" 2>/dev/null || true
    
    # Create new symlinks
    ln -sf "$INSTALL_DIR/bin/airdb" "$BIN_DIR/airdb"
    ln -sf "$INSTALL_DIR/bin/airdb-desktop" "$BIN_DIR/airdb-desktop"
    ln -sf "$INSTALL_DIR/bin/airdb-bootstrap" "$BIN_DIR/airdb-bootstrap"
    
    print_success "Symlinks created"
}

# Install desktop entry
install_desktop_entry() {
    print_info "Installing desktop entry..."
    
    cat > "$DESKTOP_DIR/airdb.desktop" << EOF
[Desktop Entry]
Name=AirDB
Comment=Local-First, GitHub-Backed Database Platform
Exec=$INSTALL_DIR/bin/airdb-desktop
Icon=database
Terminal=false
Type=Application
Categories=Development;Database;
Keywords=database;sql;nosql;git;
StartupWMClass=airdb
EOF
    
    chmod 644 "$DESKTOP_DIR/airdb.desktop"
    
    # Update desktop database if available
    if command -v update-desktop-database &> /dev/null; then
        update-desktop-database "$DESKTOP_DIR" 2>/dev/null || true
    fi
    
    print_success "Desktop entry installed"
}

# Create initial state file
create_state_file() {
    if [[ ! -f "$INSTALL_DIR/state.json" ]]; then
        print_info "Creating initial state file..."
        cat > "$INSTALL_DIR/state.json" << EOF
{
  "current_version": "0.2.6",
  "last_good_version": "0.2.6",
  "pending_version": null,
  "update_channel": "stable",
  "last_check": null,
  "status": "idle",
  "failed_boot_count": 0,
  "max_failed_boots": 3
}
EOF
        print_success "State file created"
    fi
}

# Check if PATH includes bin directory
check_path() {
    if [[ ":$PATH:" != *":$BIN_DIR:"* ]]; then
        print_warning "$BIN_DIR is not in your PATH"
        echo ""
        if [[ "$INSTALL_MODE" == "user" ]]; then
            echo -e "Add this line to your ${YELLOW}~/.bashrc${NC} or ${YELLOW}~/.zshrc${NC}:"
            echo -e "  ${GREEN}export PATH=\"\$HOME/.local/bin:\$PATH\"${NC}"
            echo ""
            echo "Then run: source ~/.bashrc (or restart your terminal)"
        fi
    else
        print_success "$BIN_DIR is already in PATH"
    fi
}

# Create uninstall script in install directory
create_uninstall_script() {
    cat > "$INSTALL_DIR/uninstall.sh" << 'UNINSTALL'
#!/bin/bash
set -e

INSTALL_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"

if [[ $EUID -eq 0 ]]; then
    BIN_DIR="/usr/local/bin"
    DESKTOP_DIR="/usr/share/applications"
else
    BIN_DIR="$HOME/.local/bin"
    DESKTOP_DIR="$HOME/.local/share/applications"
fi

echo "Uninstalling AirDB..."

rm -f "$BIN_DIR/airdb" "$BIN_DIR/airdb-desktop" "$BIN_DIR/airdb-bootstrap"
rm -f "$DESKTOP_DIR/airdb.desktop"
rm -rf "$INSTALL_DIR"

echo "✓ AirDB uninstalled successfully"
UNINSTALL
    chmod +x "$INSTALL_DIR/uninstall.sh"
}

# Main installation
main() {
    print_banner
    
    check_binaries
    set_install_paths
    create_directories
    install_binaries
    create_symlinks
    install_desktop_entry
    create_state_file
    create_uninstall_script
    
    echo ""
    echo -e "${GREEN}╔══════════════════════════════════════════╗${NC}"
    echo -e "${GREEN}║       Installation Complete! 🎉          ║${NC}"
    echo -e "${GREEN}╚══════════════════════════════════════════╝${NC}"
    echo ""
    echo "Installed to: $INSTALL_DIR"
    echo ""
    
    check_path
    
    echo ""
    echo "Quick Start:"
    echo -e "  ${BLUE}airdb --version${NC}     Check CLI version"
    echo -e "  ${BLUE}airdb-desktop${NC}       Launch GUI application"
    echo -e "  ${BLUE}airdb init mydb${NC}     Create a new database"
    echo ""
    echo "To uninstall:"
    echo -e "  ${YELLOW}$INSTALL_DIR/uninstall.sh${NC}"
    echo ""
}

main "$@"
