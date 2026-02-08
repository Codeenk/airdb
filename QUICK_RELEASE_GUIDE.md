# 🚀 AirDB v0.2.1 - Quick Release Guide

## Ready to Release? Follow These Steps

### 1. Prerequisites Check
```bash
# Verify you have build tools
cargo --version    # Should show 1.70+
node --version     # Should show 18+
npm --version      # Should show 9+

# Navigate to project
cd /home/grim/Documents/Linux_projects/newtest/airdb
```

### 2. Final Code Verification
```bash
# Run tests
cd src-tauri
cargo test

# Check for warnings
cargo clippy

# Verify frontend builds
cd ..
npm install
npm run build
```

### 3. Build Release
```bash
# Run the automated release script
./scripts/release.sh

# This will:
# - Build all binaries (desktop, cli, bootstrap)
# - Create platform bundles
# - Generate checksums
# - Create installer scripts
```

### 4. Test Binaries
```bash
# Test CLI
cd release/0.2.1/linux  # or macos/windows
./airdb-cli --version
# Should output: airdb 0.2.1

# Test desktop app
./airdb-desktop
# Should launch GUI
```

### 5. Commit & Tag
```bash
cd /home/grim/Documents/Linux_projects/newtest/airdb

# Review changes
git status
git diff

# Use automated commit script
./scripts/commit_release.sh
# This will:
# - Stage all changes
# - Create detailed commit
# - Tag v0.2.1
# - Push to origin (if you confirm)
```

### 6. Create GitHub Release
```bash
# Option A: GitHub CLI (recommended)
gh release create v0.2.1 \
  --title "AirDB v0.2.1 - Productization Release" \
  --notes-file RELEASE_NOTES.md \
  release/0.2.1/**/*

# Option B: Manual
# 1. Go to https://github.com/yourusername/airdb/releases/new
# 2. Choose tag: v0.2.1
# 3. Copy content from RELEASE_NOTES.md
# 4. Upload all files from release/0.2.1/
```

### 7. Post-Release Testing
```bash
# Test installation on clean VM/system
wget https://github.com/yourusername/airdb/releases/download/v0.2.1/airdb-0.2.1-linux.tar.gz
tar -xzf airdb-0.2.1-linux.tar.gz
cd airdb-0.2.1-linux
./install.sh

# Verify PATH works
airdb --version

# Enable auto-start
airdb autostart enable

# Reboot and verify it starts
```

---

## 🎯 What's New in v0.2.1

### For Users
- **Auto-start on boot** - AirDB starts automatically when you log in
- **Global `airdb` command** - Works from any terminal, immediately after install
- **Beautiful CLI** - Colorized output, progress bars, clear status indicators
- **Visual table editor** - Create and modify tables without writing SQL
- **Update UI** - Check and apply updates from Settings panel

### For Developers
- New modules: `installer`, `cli/formatter`
- Enhanced auto-start with bootstrapper safety
- Comprehensive documentation
- Production-ready build scripts

---

## 📋 Quick Reference

### Key Files Changed
```
src-tauri/
  ├─ Cargo.toml (version 0.2.1)
  ├─ src/
  │   ├─ commands/autostart.rs (improved bootstrapper path)
  │   └─ engine/
  │       ├─ installer/mod.rs (NEW - PATH installation)
  │       └─ cli/formatter.rs (NEW - terminal formatting)
  
docs/
  └─ team-workflows.md (NEW)

package.json (version 0.2.1)
CHANGELOG.md (NEW)
RELEASE_NOTES.md (NEW)
RELEASE_CHECKLIST.md (NEW)
IMPLEMENTATION_SUMMARY.md (NEW)

scripts/
  ├─ release.sh (NEW)
  └─ commit_release.sh (NEW)
```

### Commands to Know
```bash
# Development
npm run dev              # Start dev server
npm run build            # Build frontend
cargo run --bin airdb-cli   # Run CLI in dev
cargo build --release    # Build release binaries

# Testing
cargo test              # Run Rust tests
cargo clippy            # Lint Rust code
npm run preview         # Preview built frontend

# Release
./scripts/release.sh    # Build release artifacts
./scripts/commit_release.sh  # Commit and tag

# Deployment
gh release create v0.2.1  # Create GitHub release
```

---

## ⚠️ Common Issues

### "cargo: command not found"
```bash
# Install Rust
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source $HOME/.cargo/env
```

### "npm: command not found"
```bash
# Install Node.js (using nvm)
curl -o- https://raw.githubusercontent.com/nvm-sh/nvm/v0.39.0/install.sh | bash
source ~/.bashrc
nvm install 20
```

### Build fails with "linker errors"
```bash
# Linux: Install build essentials
sudo apt install build-essential pkg-config libssl-dev

# macOS: Install Xcode Command Line Tools
xcode-select --install
```

### Frontend build fails
```bash
# Clear node_modules and reinstall
rm -rf node_modules package-lock.json
npm install
npm run build
```

---

## 📊 Success Criteria

Before announcing the release, verify:
- [ ] All binaries run on target platforms
- [ ] Auto-start works after reboot
- [ ] `airdb` command works from any directory
- [ ] Visual table editor creates migrations correctly
- [ ] Update UI shows current version
- [ ] Documentation is accessible and complete
- [ ] GitHub release has all artifacts
- [ ] Checksums match downloaded files

---

## 🎉 You're Done!

After successful release:
1. Announce on social media
2. Update README with v0.2.1 features
3. Monitor GitHub issues
4. Prepare for v0.2.2 (bug fixes) or v0.3.0 (new features)

**Congratulations on shipping v0.2.1! 🚀**
