#!/bin/bash
# AirDB v0.2.1 Production Release Script
# This script builds production-ready binaries for all platforms

set -e  # Exit on error

VERSION="0.2.1"
PROJECT_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
RELEASE_DIR="$PROJECT_ROOT/release/$VERSION"

echo "🚀 AirDB v$VERSION Production Release"
echo "======================================"

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Helper functions
log_info() {
    echo -e "${BLUE}ℹ${NC} $1"
}

log_success() {
    echo -e "${GREEN}✓${NC} $1"
}

log_warning() {
    echo -e "${YELLOW}⚠${NC} $1"
}

log_error() {
    echo -e "${RED}✗${NC} $1"
}

# Step 1: Verify version consistency
log_info "Verifying version consistency..."

VERSION_CARGO=$(grep '^version = ' "$PROJECT_ROOT/src-tauri/Cargo.toml" | head -1 | sed 's/.*"\(.*\)".*/\1/')
VERSION_PACKAGE=$(grep '"version":' "$PROJECT_ROOT/package.json" | sed 's/.*"\(.*\)".*/\1/')
VERSION_TAURI=$(grep '"version":' "$PROJECT_ROOT/src-tauri/tauri.conf.json" | sed 's/.*"\(.*\)".*/\1/')

if [ "$VERSION_CARGO" != "$VERSION" ] || [ "$VERSION_PACKAGE" != "$VERSION" ] || [ "$VERSION_TAURI" != "$VERSION" ]; then
    log_error "Version mismatch detected!"
    echo "  Cargo.toml: $VERSION_CARGO"
    echo "  package.json: $VERSION_PACKAGE"
    echo "  tauri.conf.json: $VERSION_TAURI"
    echo "  Expected: $VERSION"
    exit 1
fi

log_success "Version consistency verified"

# Step 2: Run tests
log_info "Running tests..."
cd "$PROJECT_ROOT/src-tauri"

if cargo test --quiet; then
    log_success "Tests passed"
else
    log_error "Tests failed"
    exit 1
fi

# Step 3: Check for errors
log_info "Checking for compilation errors..."
if cargo check --release --quiet; then
    log_success "No compilation errors"
else
    log_error "Compilation errors detected"
    exit 1
fi

# Step 4: Build frontend
log_info "Building frontend..."
cd "$PROJECT_ROOT"
if npm run build; then
    log_success "Frontend built"
else
    log_error "Frontend build failed"
    exit 1
fi

# Step 5: Build Rust binaries
log_info "Building Rust binaries (release mode)..."
cd "$PROJECT_ROOT/src-tauri"

# Build all binaries
log_info "  - airdb-desktop"
cargo build --release --bin airdb-desktop --quiet

log_info "  - airdb-cli"
cargo build --release --bin airdb-cli --quiet

log_info "  - airdb-bootstrap"
cargo build --release --bin airdb-bootstrap --quiet

log_success "All binaries built"

# Step 6: Create release directory structure
log_info "Creating release directory..."
mkdir -p "$RELEASE_DIR"/{linux,windows,macos}
mkdir -p "$RELEASE_DIR/checksums"

# Step 7: Copy binaries
log_info "Copying binaries..."

TARGET_DIR="$PROJECT_ROOT/src-tauri/target/release"

# Determine platform
PLATFORM=$(uname -s | tr '[:upper:]' '[:lower:]')

if [ "$PLATFORM" = "linux" ]; then
    cp "$TARGET_DIR/airdb-desktop" "$RELEASE_DIR/linux/"
    cp "$TARGET_DIR/airdb-cli" "$RELEASE_DIR/linux/"
    cp "$TARGET_DIR/airdb-bootstrap" "$RELEASE_DIR/linux/"
    
    # Strip binaries
    strip "$RELEASE_DIR/linux/airdb-desktop"
    strip "$RELEASE_DIR/linux/airdb-cli"
    strip "$RELEASE_DIR/linux/airdb-bootstrap"
    
    log_success "Linux binaries copied and stripped"
    
elif [ "$PLATFORM" = "darwin" ]; then
    cp "$TARGET_DIR/airdb-desktop" "$RELEASE_DIR/macos/"
    cp "$TARGET_DIR/airdb-cli" "$RELEASE_DIR/macos/"
    cp "$TARGET_DIR/airdb-bootstrap" "$RELEASE_DIR/macos/"
    
    # Strip binaries
    strip "$RELEASE_DIR/macos/airdb-desktop"
    strip "$RELEASE_DIR/macos/airdb-cli"
    strip "$RELEASE_DIR/macos/airdb-bootstrap"
    
    log_success "macOS binaries copied and stripped"
fi

# Step 8: Build Tauri bundle
log_info "Building Tauri bundle..."
cd "$PROJECT_ROOT"

if npm run tauri build -- --verbose; then
    log_success "Tauri bundle created"
    
    # Copy bundle to release dir
    BUNDLE_DIR="$PROJECT_ROOT/src-tauri/target/release/bundle"
    
    if [ -d "$BUNDLE_DIR/deb" ]; then
        cp "$BUNDLE_DIR/deb"/*.deb "$RELEASE_DIR/linux/" 2>/dev/null || true
    fi
    
    if [ -d "$BUNDLE_DIR/appimage" ]; then
        cp "$BUNDLE_DIR/appimage"/*.AppImage "$RELEASE_DIR/linux/" 2>/dev/null || true
    fi
    
    if [ -d "$BUNDLE_DIR/dmg" ]; then
        cp "$BUNDLE_DIR/dmg"/*.dmg "$RELEASE_DIR/macos/" 2>/dev/null || true
    fi
    
    if [ -d "$BUNDLE_DIR/msi" ]; then
        cp "$BUNDLE_DIR/msi"/*.msi "$RELEASE_DIR/windows/" 2>/dev/null || true
    fi
else
    log_warning "Tauri bundle creation failed (continuing anyway)"
fi

# Step 9: Generate checksums
log_info "Generating checksums..."
cd "$RELEASE_DIR"

for dir in linux macos windows; do
    if [ -d "$dir" ] && [ "$(ls -A $dir)" ]; then
        (cd "$dir" && sha256sum * > "../checksums/${dir}_sha256.txt" 2>/dev/null) || true
    fi
done

log_success "Checksums generated"

# Step 10: Create metadata file
log_info "Creating release metadata..."

cat > "$RELEASE_DIR/release.json" <<EOF
{
  "version": "$VERSION",
  "releaseDate": "$(date -u +%Y-%m-%dT%H:%M:%SZ)",
  "channel": "stable",
  "platform": "$PLATFORM",
  "changelog": "See CHANGELOG.md",
  "files": {
    "linux": $(find "$RELEASE_DIR/linux" -type f -exec basename {} \; 2>/dev/null | jq -R -s -c 'split("\n") | map(select(length > 0))' || echo '[]'),
    "macos": $(find "$RELEASE_DIR/macos" -type f -exec basename {} \; 2>/dev/null | jq -R -s -c 'split("\n") | map(select(length > 0))' || echo '[]'),
    "windows": $(find "$RELEASE_DIR/windows" -type f -exec basename {} \; 2>/dev/null | jq -R -s -c 'split("\n") | map(select(length > 0))' || echo '[]')
  }
}
EOF

log_success "Release metadata created"

# Step 11: Create installer scripts
log_info "Creating installer scripts..."

# Linux installer
cat > "$RELEASE_DIR/linux/install.sh" <<'EOF'
#!/bin/bash
# AirDB Linux Installer

set -e

VERSION="0.2.1"
INSTALL_DIR="$HOME/.local/share/airdb"
BIN_DIR="$HOME/.local/bin"

echo "🚀 Installing AirDB v$VERSION"

# Create directories
mkdir -p "$INSTALL_DIR/current"
mkdir -p "$BIN_DIR"

# Copy binaries
cp airdb-desktop "$INSTALL_DIR/current/"
cp airdb-cli "$INSTALL_DIR/current/"
cp airdb-bootstrap "$INSTALL_DIR/current/"

chmod +x "$INSTALL_DIR/current/airdb-desktop"
chmod +x "$INSTALL_DIR/current/airdb-cli"
chmod +x "$INSTALL_DIR/current/airdb-bootstrap"

# Create symlinks
ln -sf "$INSTALL_DIR/current/airdb-bootstrap" "$BIN_DIR/airdb"
ln -sf "$INSTALL_DIR/current/airdb-bootstrap" "$BIN_DIR/airdb-bootstrap"

echo "✓ AirDB installed to $INSTALL_DIR"
echo ""
echo "⚠️  Make sure $BIN_DIR is in your PATH:"
echo "  export PATH=\"\$HOME/.local/bin:\$PATH\""
echo ""
echo "Run 'airdb --version' to verify installation"
EOF

chmod +x "$RELEASE_DIR/linux/install.sh"

log_success "Installer scripts created"

# Step 12: Summary
echo ""
echo "======================================"
log_success "Release v$VERSION ready!"
echo "======================================"
echo ""
echo "📦 Release artifacts:"
echo "  Location: $RELEASE_DIR"
echo ""

if [ -d "$RELEASE_DIR/linux" ] && [ "$(ls -A $RELEASE_DIR/linux)" ]; then
    echo "  Linux:"
    ls -lh "$RELEASE_DIR/linux" | tail -n +2 | awk '{print "    - " $9 " (" $5 ")"}'
fi

if [ -d "$RELEASE_DIR/macos" ] && [ "$(ls -A $RELEASE_DIR/macos)" ]; then
    echo "  macOS:"
    ls -lh "$RELEASE_DIR/macos" | tail -n +2 | awk '{print "    - " $9 " (" $5 ")"}'
fi

if [ -d "$RELEASE_DIR/windows" ] && [ "$(ls -A $RELEASE_DIR/windows)" ]; then
    echo "  Windows:"
    ls -lh "$RELEASE_DIR/windows" | tail -n +2 | awk '{print "    - " $9 " (" $5 ")"}'
fi

echo ""
echo "📋 Next steps:"
echo "  1. Test binaries: cd $RELEASE_DIR/$PLATFORM && ./airdb-cli --version"
echo "  2. Review checksums: cat $RELEASE_DIR/checksums/*"
echo "  3. Create GitHub release: gh release create v$VERSION"
echo "  4. Upload artifacts: gh release upload v$VERSION $RELEASE_DIR/**/*"
echo ""

log_info "Done!"
