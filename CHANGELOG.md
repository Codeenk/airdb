# Changelog

All notable changes to AirDB will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.2.5] - 2026-02-09

### 🎉 Productization Complete

This release marks full productization layer completion for public beta.

### ✨ Added

#### Visual SQL Table Editor - Fully Integrated
- **Tauri commands registered** - All schema editor commands now work:
  - `get_tables` - List all user tables
  - `get_table_schema` - Get column definitions
  - `get_table_indexes` - Get index information
  - `generate_table_migration` - Create migration from UI edits
  - `apply_generated_migration` - Apply changes safely
- **ConstraintEditor component** - Foreign keys and check constraints
- **IndexManager integration** - Inline index management
- **Full TableEditor** - Column, index, and constraint editing in one view

#### Autostart Commands Registered
- `get_autostart_status`, `enable_autostart`, `disable_autostart` now available

### 📚 Documentation

- **Revamped README** - Modern design with all 12 documentation links
- **Complete doc structure** - Introduction, Installation, Quick Start, CLI Reference, SQL/NoSQL guides, Migrations, Updates, Team Workflows, Security, FAQ

### 🔧 Changed

- Version bump from 0.2.1 to 0.2.5
- README now includes Visual SQL Editor feature highlight
- Architecture diagram updated with new components

---

## [0.2.1] - 2026-02-08

### 🎉 Productization Release

This release focuses on production-grade features for public beta readiness.

### ✨ Added

#### Auto-Start at Boot
- **Cross-platform auto-start** - Automatically launch AirDB on system boot
  - Windows: Task Scheduler integration (no admin required)
  - Linux: XDG autostart `.desktop` file support
  - macOS: LaunchAgent integration
- **Bootstrapper-based** - Always starts via bootstrapper for version switching safety
- **UI Controls** - Toggle auto-start from Settings with status indicators
- **Rollback-aware** - Defers startup if update/migration locks exist

#### CLI PATH Availability
- **System-wide `airdb` command** - Works immediately after install
  - Windows: Installer adds to User PATH automatically
  - Linux: Symlinks to `~/.local/bin` (respects XDG standards)
  - macOS: Intelligent fallback to `/usr/local/bin` or `~/.local/bin`
- **Version-switching safe** - PATH points to bootstrapper, never breaks on updates
- **No manual configuration** - Installer handles everything

#### Enhanced Terminal UX
- **Colorized output** - Clear, professional CLI formatting
- **Status indicators** - ✓ ✗ ⚠ ℹ symbols for quick scanning
- **Progress bars** - Visual feedback for long operations
- **JSON mode** - `--format json` for automation/scripting
- **Formatted commands**:
  - `airdb status` - Enhanced project overview
  - `airdb update check` - Beautiful update notifications
  - `airdb update apply` - Clear progress indicators
  - `airdb project info` - Detailed health dashboard

#### Visual SQL Table Editor (Supabase-Level)
- **Full visual editing** - Create/modify tables, columns, indexes, constraints
- **Migration generation** - UI edits always generate migrations
- **Preview before apply** - Review SQL diff before committing
- **Rollback SQL** - Every change includes down migration
- **Safety first** - Impossible to break production via UI click

#### Updater UI
- **Settings → Updates panel** - Full update management interface
  - Current version display
  - Update channel selector (Stable/Beta/Nightly)
  - Last check timestamp
  - One-click update checking
  - Apply update button with progress
  - Changelog viewer
- **Global update banner** - Non-intrusive notifications
- **State indicators**:
  - Checking for updates...
  - Downloading (with progress %)
  - Ready to apply (restart required)
  - Rolled back (with explanation)
- **Lock-aware** - Respects migration/backup/serve operations

### 🔧 Changed

- **Bootstrapper path resolution** - Auto-start now correctly locates bootstrapper
- **Update state management** - Improved coordination between CLI and UI
- **Migration preview** - Enhanced SQL formatting and diff display
- **Error messages** - More actionable and user-friendly

### 📚 Documentation

- Added comprehensive [Team Workflows](docs/team-workflows.md) guide
  - Branch isolation explained
  - Merge safety procedures
  - Conflict resolution workflows
  - RBAC examples
  - Best practices & anti-patterns
- Updated [Introduction](docs/introduction.md) with v0.2.1 features
- Enhanced [Updates & Rollback](docs/updates-and-rollback.md) guide

### 🐛 Fixed

- Auto-start now uses bootstrapper instead of main binary
- CLI PATH installation respects platform conventions
- Update checks no longer block UI during migration operations
- TableEditor migration preview properly escapes SQL

### 🔐 Security

- PATH installation uses user-level permissions (no admin/root required)
- Auto-start tasks run with minimal privileges
- Update verification includes signature checks

### 🏗️ Technical

- Added `which` dependency for PATH detection
- Added `colored` dependency for CLI formatting
- Added `winreg` dependency (Windows registry for PATH)
- New `engine/installer` module for PATH management
- New `engine/cli/formatter` module for consistent output
- Improved error handling in autostart module

---

## [0.1.0] - 2026-01-15

### 🎉 Initial Release

- Core database engine (SQLite + NoSQL)
- Migration system with versioning
- GitHub sync integration
- Basic CLI (`init`, `migrate`, `serve`)
- Desktop app with Tauri
- RBAC and team collaboration
- Self-updater with rollback
- Conflict resolution
- Audit logging
- Health dashboard

---

## Upgrade Guide

### From 0.1.0 to 0.2.1

No breaking changes. Just update:

```bash
airdb update check
airdb update apply
# Restart AirDB
```

**New features you'll want to try:**

1. Enable auto-start: Settings → General → Start on boot
2. Try the visual table editor: Dashboard → Tables
3. Check the new update UI: Settings → Updates
4. Use colorized CLI: `airdb status`, `airdb update check`

---

## Links

- [GitHub Repository](https://github.com/yourusername/airdb)
- [Documentation](docs/)
- [Issue Tracker](https://github.com/yourusername/airdb/issues)
- [Releases](https://github.com/yourusername/airdb/releases)
