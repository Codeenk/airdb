# AirDB v0.2.1 - Productization Release 🚀

## What's New

AirDB v0.2.1 is the **Productization Release** — making AirDB production-ready for public beta with essential features every database tool needs.

### 🌟 Highlights

#### 🔄 Auto-Start on Boot
- **Windows**: Task Scheduler integration (no admin required)
- **Linux**: XDG autostart with `.desktop` file
- **macOS**: LaunchAgent for seamless startup
- Always starts via bootstrapper for safe version switching
- Toggle from Settings with visual status indicators

#### 🛤️ System-Wide CLI Access
- `airdb` command works everywhere after install
- Automatic PATH setup (no manual configuration needed)
- Version-switching safe (PATH points to bootstrapper)
- Respects platform conventions (no root/admin required)

#### 🎨 Pro-Grade Terminal UX
- Beautiful colorized output with status symbols (✓ ✗ ⚠ ℹ)
- Progress bars for long operations
- `--format json` for scripting/automation
- Enhanced commands: `status`, `update check`, `project info`

#### 📊 Visual SQL Table Editor
- **Supabase-level experience** in a local-first tool
- Create/modify tables visually
- Every UI edit generates migrations (safety first!)
- Preview SQL before applying
- Automatic rollback SQL generation
- **Impossible to break production via UI click**

#### 🔄 Full Updater UI
- **Settings → Updates** panel with full control
- Update channel selector (Stable/Beta/Nightly)
- One-click checking and applying
- Real-time progress indicators
- Non-intrusive update banner
- Automatic rollback on failure

---

## Installation

### Linux

```bash
# Download and install
wget https://github.com/yourusername/airdb/releases/download/v0.2.1/airdb-0.2.1-linux.tar.gz
tar -xzf airdb-0.2.1-linux.tar.gz
cd airdb-0.2.1-linux
./install.sh

# Verify
airdb --version
```

### macOS

```bash
# Download DMG
wget https://github.com/yourusername/airdb/releases/download/v0.2.1/AirDB_0.2.1.dmg

# Or use Homebrew (coming soon)
# brew install airdb
```

### Windows

```powershell
# Download and run installer
# https://github.com/yourusername/airdb/releases/download/v0.2.1/AirDB_0.2.1.msi
```

---

## Quick Start

```bash
# Initialize a project
airdb init --name myproject

# Create and run migrations
airdb migrate create add_users_table
# Edit sql/migrations/001_add_users_table.sql
airdb migrate push

# Start API server
airdb serve

# Check project status
airdb status
```

---

## Upgrade from v0.1.0

No breaking changes! Just update:

```bash
airdb update check
airdb update apply
# Restart AirDB
```

**Try the new features:**
1. ✅ Enable auto-start: Settings → General → Start on boot
2. 📊 Visual table editor: Dashboard → Tables
3. 🔄 Update UI: Settings → Updates
4. 🎨 Colorized CLI: `airdb status`

---

## Full Changelog

### ✨ Added
- Cross-platform auto-start (Windows Task Scheduler, Linux XDG, macOS LaunchAgent)
- Automatic CLI PATH installation (no manual setup)
- Colorized terminal output with progress indicators
- Visual SQL table editor with migration preview
- Complete updater UI in Settings
- Global update banner (non-intrusive)
- JSON output mode (`--format json`) for automation
- Enhanced CLI commands with beautiful formatting

### 🔧 Changed
- Bootstrapper path resolution improved
- Update state management between CLI and UI
- Migration preview with better SQL formatting
- More actionable error messages

### 📚 Documentation
- New: [Team Workflows](https://github.com/yourusername/airdb/blob/main/docs/team-workflows.md)
- Updated: All docs reflect v0.2.1 features

### 🐛 Fixed
- Auto-start now correctly uses bootstrapper
- CLI PATH respects platform conventions
- Update checks don't block during migrations
- TableEditor SQL escaping

### 🔐 Security
- User-level PATH installation (no admin/root)
- Auto-start with minimal privileges
- Update signature verification

---

## Documentation

📖 [Full Documentation](https://github.com/yourusername/airdb/tree/main/docs)

- [Introduction](docs/introduction.md)
- [Installation Guide](docs/installation.md)
- [Quick Start](docs/quickstart.md)
- [CLI Reference](docs/cli-reference.md)
- [Migrations](docs/migrations.md)
- [Team Workflows](docs/team-workflows.md) ⭐ NEW
- [Updates & Rollback](docs/updates-and-rollback.md)
- [FAQ](docs/faq.md)

---

## System Requirements

- **Linux**: Ubuntu 20.04+, Fedora 35+, or equivalent (glibc 2.31+)
- **macOS**: 11.0 (Big Sur) or later
- **Windows**: Windows 10 (1809+) or Windows 11

---

## Checksums

See [checksums/](checksums/) directory for SHA256 hashes of all binaries.

---

## Support

- 🐛 [Report a bug](https://github.com/yourusername/airdb/issues/new?template=bug_report.md)
- 💡 [Request a feature](https://github.com/yourusername/airdb/issues/new?template=feature_request.md)
- 💬 [Join Discord](https://discord.gg/airdb) (coming soon)
- 📧 Email: support@airdb.dev

---

## Contributors

Thank you to everyone who contributed to this release! 🎉

---

**Full Changelog**: https://github.com/yourusername/airdb/compare/v0.1.0...v0.2.1
