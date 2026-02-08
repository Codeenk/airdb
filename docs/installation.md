# Installation

## Quick Install

### Linux

```bash
# Download and run installer
curl -fsSL https://airdb.dev/install.sh | bash
```

### macOS

```bash
# Download and run installer
curl -fsSL https://airdb.dev/install.sh | bash

# Or with Homebrew
brew install airdb/tap/airdb
```

### Windows

Download the installer from [GitHub Releases](https://github.com/airdb/airdb/releases).

Or with Scoop:
```powershell
scoop bucket add airdb https://github.com/airdb/scoop-bucket
scoop install airdb
```

## Verify Installation

```bash
airdb --version
# AirDB 0.1.0

airdb auth status
# Not logged in
```

## GitHub Authentication

AirDB uses GitHub for sync and collaboration:

```bash
airdb auth login
# Browser opens for GitHub OAuth...
# ✓ Logged in as username
```

## System Requirements

| Platform | Minimum |
|----------|---------|
| Linux | Ubuntu 20.04+, glibc 2.31+ |
| macOS | 11 Big Sur+ |
| Windows | Windows 10+ |
| Storage | 100MB free |
| RAM | 256MB |

## CLI PATH Setup

The installer automatically adds `airdb` to your PATH:

- **Linux/macOS**: Symlink at `/usr/local/bin/airdb`
- **Windows**: Added to User PATH

If `airdb` is not found after install, restart your terminal or run:

```bash
# Linux/macOS
source ~/.bashrc  # or ~/.zshrc

# Windows (PowerShell)
$env:Path = [System.Environment]::GetEnvironmentVariable("Path","User")
```

## Auto-Start (Optional)

Enable AirDB to start on system boot:

```bash
# Enable
airdb settings set autostart true

# Disable
airdb settings set autostart false

# Check status
airdb settings get autostart
```

## Next Steps

- [Quick Start Guide](quickstart.md) - Create your first project
- [CLI Reference](cli-reference.md) - All available commands
