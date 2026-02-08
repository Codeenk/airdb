# AirDB v0.2.1 - Production Release Checklist

## ✅ Pre-Release Verification

### Version Updates
- [x] Cargo.toml: `0.2.1`
- [x] package.json: `0.2.1`
- [x] tauri.conf.json: `0.2.1`
- [x] CHANGELOG.md created with full v0.2.1 details
- [x] RELEASE_NOTES.md created for GitHub release

### Code Quality
- [ ] Run `cargo test` - All tests pass
- [ ] Run `cargo check --release` - No compilation errors
- [ ] Run `cargo clippy` - No warnings
- [ ] Frontend builds successfully (`npm run build`)
- [ ] Desktop app launches without errors

### Feature Completeness

#### 1. Auto-Start at Boot ✅
- [x] AutostartManager module (`src-tauri/src/engine/autostart/mod.rs`)
- [x] Windows implementation (Task Scheduler)
- [x] Linux implementation (XDG .desktop)
- [x] macOS implementation (LaunchAgent)
- [x] Bootstrapper path resolution fixed
- [x] Tauri commands exported (`enable_autostart`, `disable_autostart`, `get_autostart_status`)
- [ ] **TEST**: Toggle auto-start in UI on Windows/Linux/macOS
- [ ] **TEST**: Reboot and verify bootstrapper launches

#### 2. CLI PATH Availability ✅
- [x] Installer module created (`src-tauri/src/engine/installer/mod.rs`)
- [x] Windows PATH installation (User PATH via registry)
- [x] Linux symlink to `~/.local/bin`
- [x] macOS intelligent fallback
- [x] Dependencies added (`which`, `winreg`)
- [ ] **TEST**: After install, run `airdb --version` from any directory
- [ ] **TEST**: Version switch doesn't break PATH

#### 3. Enhanced Terminal UX ✅
- [x] CliFormatter module (`src-tauri/src/engine/cli/formatter.rs`)
- [x] Colorized output with `colored` crate
- [x] Status symbols (✓ ✗ ⚠ ℹ)
- [x] Progress bars
- [x] Enhanced `cmd_status` function
- [x] JSON output mode support
- [ ] **TEST**: Run `airdb status` - verify colors and formatting
- [ ] **TEST**: Run `airdb update check` - verify output
- [ ] **TEST**: Run with `--format json` - verify JSON output

#### 4. Visual SQL Table Editor ✅
- [x] TableEditor component (`src/components/TableEditor.tsx`)
- [x] Schema editor commands (`src-tauri/src/commands/schema_editor.rs`)
- [x] Migration preview generation
- [x] Up/Down SQL generation
- [x] Add column, remove column, edit column
- [x] Create table flow
- [ ] **TEST**: Create new table via UI
- [ ] **TEST**: Add column to existing table
- [ ] **TEST**: Preview migration before applying
- [ ] **TEST**: Verify migration file created in sql/migrations/

#### 5. Updater UI ✅
- [x] UpdateSettings component (`src/components/UpdateSettings.tsx`)
- [x] UpdateBanner component (`src/components/UpdateBanner.tsx`)
- [x] Update status display
- [x] Channel selector
- [x] Progress indicators
- [x] Lock-aware update blocking
- [ ] **TEST**: Open Settings → Updates
- [ ] **TEST**: Click "Check for Updates"
- [ ] **TEST**: Verify banner appears when update available
- [ ] **TEST**: Test during migration (should show locked state)

### Documentation ✅
- [x] team-workflows.md created
- [x] All required docs exist:
  - [x] introduction.md
  - [x] installation.md
  - [x] quickstart.md
  - [x] cli-reference.md
  - [x] sql-guide.md
  - [x] nosql-guide.md
  - [x] migrations.md
  - [x] updates-and-rollback.md
  - [x] team-workflows.md
  - [x] security.md
  - [x] faq.md

### Build & Release
- [ ] Run `scripts/release.sh` successfully
- [ ] Linux binaries built and stripped
- [ ] macOS binaries built and stripped (if on macOS)
- [ ] Windows binaries built (if on Windows)
- [ ] Tauri bundles created (.deb, .AppImage, .dmg, .msi)
- [ ] SHA256 checksums generated
- [ ] Release metadata (`release.json`) created
- [ ] Install script tested (`linux/install.sh`)

### Quality Assurance

#### Functional Testing
- [ ] **Fresh Install**: Install on clean system, verify PATH works
- [ ] **Auto-Start**: Enable, reboot, verify starts automatically
- [ ] **CLI Commands**: Test all major commands
  - [ ] `airdb init`
  - [ ] `airdb migrate create`
  - [ ] `airdb migrate push`
  - [ ] `airdb serve`
  - [ ] `airdb status`
  - [ ] `airdb update check`
  - [ ] `airdb project info`
- [ ] **Visual Editor**: Create table, add columns, preview migration
- [ ] **Update Flow**: Mock update, apply, verify restart works
- [ ] **Rollback**: Test update rollback scenario

#### Cross-Platform Testing
- [ ] **Linux**: Ubuntu 22.04, Fedora 39
- [ ] **macOS**: 13.0 (Ventura), 14.0 (Sonoma)
- [ ] **Windows**: Windows 10, Windows 11

#### Performance
- [ ] App launches in < 2 seconds
- [ ] UI remains responsive during operations
- [ ] Migration generation is instant
- [ ] Update check completes in < 5 seconds

### Security Review
- [ ] No hardcoded credentials
- [ ] No sensitive data in logs
- [ ] Update signatures verified
- [ ] Auto-start runs with minimal privileges
- [ ] PATH installation doesn't require admin/root

---

## 📦 Release Steps

### 1. Pre-Release
```bash
# Ensure clean working directory
git status

# Run full test suite
cd src-tauri
cargo test
cargo clippy

# Build frontend
cd ..
npm run build

# Run release script
./scripts/release.sh
```

### 2. GitHub Release
```bash
# Create and push tag
git tag -a v0.2.1 -m "v0.2.1 - Productization Release"
git push origin v0.2.1

# Create GitHub release
gh release create v0.2.1 \
  --title "AirDB v0.2.1 - Productization Release" \
  --notes-file RELEASE_NOTES.md \
  release/0.2.1/**/*
```

### 3. Post-Release
- [ ] Announce on social media
- [ ] Update website/landing page
- [ ] Notify beta testers
- [ ] Monitor issue tracker for reports
- [ ] Update documentation site

---

## 🐛 Known Issues (Document if any)

None currently identified in v0.2.1.

---

## 📊 Success Metrics

Track these after release:
- Download count by platform
- Installation success rate
- Update adoption rate (0.1.0 → 0.2.1)
- Issue reports vs. downloads
- User feedback sentiment

---

## 🚀 Next Steps (v0.3.0)

Future enhancements not in v0.2.1:
- [ ] Cloud sync (optional GitHub alternative)
- [ ] Data browser UI (like phpMyAdmin)
- [ ] Query builder UI
- [ ] Performance monitoring dashboard
- [ ] Multi-tenant support
- [ ] Plugin system

---

## Notes

- All critical productization features implemented
- Focus on stability and user experience
- Ready for public beta announcement
- Monitor feedback closely for v0.2.2 bug fix release if needed
