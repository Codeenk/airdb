# 🗄️ AirDB

**The local-first, GitHub-backed database platform for schema management and API generation.**

[![CI](https://github.com/Codeenk/airdb/actions/workflows/ci.yml/badge.svg)](https://github.com/Codeenk/airdb/actions/workflows/ci.yml)
[![Release](https://img.shields.io/badge/release-v0.2.5-22d3ee)](https://github.com/Codeenk/airdb/releases)
[![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg)](LICENSE)
[![Platform](https://img.shields.io/badge/platform-Linux%20%7C%20macOS%20%7C%20Windows-lightgrey)](https://github.com/Codeenk/airdb/releases)

---

## ✨ Why AirDB?

Traditional databases lock you into cloud services. AirDB gives you **full ownership**:

- 🔒 **Your data stays local** - SQLite on your machine, no cloud required
- 📂 **Version control built-in** - GitHub syncs your schema like code
- 🔄 **Safe rollbacks** - Every change generates reversible migrations
- 👥 **Team collaboration** - Branch isolation, RBAC, and conflict resolution
- ⚡ **Instant REST API** - Auto-generated with OpenAPI + Swagger UI

---

## 🚀 Quick Start

### Installation

<details>
<summary><b>Linux</b></summary>

```bash
# AppImage (portable)
chmod +x airdb_0.2.5_amd64.AppImage
./airdb_0.2.5_amd64.AppImage

# Debian/Ubuntu
sudo dpkg -i airdb_0.2.5_amd64.deb
```
</details>

<details>
<summary><b>macOS</b></summary>

```bash
open AirDB_0.2.5_aarch64.dmg
# Drag to Applications
```
</details>

<details>
<summary><b>Windows</b></summary>

```powershell
.\AirDB_0.2.5_x64_en-US.msi
```
</details>

### Create Your First Project

```bash
# Initialize
airdb init my-project && cd my-project

# Create a table via migration
airdb migrate create add_users_table
# Edit: sql/migrations/001_add_users_table.sql

# Apply migration
airdb migrate push

# Start API server
airdb serve
# → http://localhost:54321/swagger-ui
```

---

## 🖥️ Features

### Visual SQL Editor (NEW in v0.2.5)

Edit tables, columns, indexes, and constraints visually. **Every change generates a migration** - impossible to break production with a click.

| Feature | Description |
|---------|-------------|
| 📋 Table Editor | Create/modify tables with full column options |
| 📇 Index Manager | Add/remove indexes with unique support |
| 🔗 Constraint Editor | Foreign keys and check constraints |
| 👁️ Migration Preview | Review SQL before applying |

### GitHub Sync

Your database schema versioned alongside your code:

```bash
airdb auth login              # OAuth device flow
airdb sync setup --create     # Initialize repo
airdb sync push -m "Add users table"
airdb sync pull               # Get team changes
```

### Self-Updating with Rollback

```bash
airdb update check    # Check for updates
airdb update apply    # Download & apply
# Problems? Automatic rollback on crash
```

### Team Features

- **Role-based access**: `admin`, `developer`, `readonly`
- **Branch isolation**: Work without conflicts
- **Smart merging**: Auto-resolve non-conflicting changes
- **Conflict resolution**: `--ours` / `--theirs` strategies

---

## 📚 Documentation

| Guide | Description |
|-------|-------------|
| **[Introduction](docs/introduction.md)** | What is AirDB and why use it |
| **[Installation](docs/installation.md)** | Platform-specific install guides |
| **[Quick Start](docs/quickstart.md)** | 5-minute getting started |
| **[CLI Reference](docs/cli-reference.md)** | All CLI commands |
| **[SQL Guide](docs/sql-guide.md)** | Working with SQL databases |
| **[NoSQL Guide](docs/nosql-guide.md)** | Working with document storage |
| **[Migrations](docs/migrations.md)** | Schema versioning explained |
| **[Updates & Rollback](docs/updates-and-rollback.md)** | Self-updater system |
| **[Team Workflows](docs/team-workflows.md)** | Collaboration patterns |
| **[Conflict Resolution](docs/conflict-resolution.md)** | Handling merge conflicts |
| **[Security](docs/security.md)** | Security model & best practices |
| **[FAQ](docs/faq.md)** | Common questions |

---

## 🏗️ Architecture

```
┌─────────────────────────────────────────────────┐
│           Desktop UI (Tauri + React)            │
│  • Visual Table Editor   • Update Settings      │
│  • NoSQL Browser         • GitHub Auth          │
└──────────────────┬──────────────────────────────┘
                   │ IPC
┌──────────────────▼──────────────────────────────┐
│              Rust Engine (Core)                 │
│  • Migration Runner    • Operation Locks        │
│  • GitHub Sync         • Self-Updater           │
│  • REST API Gen        • RBAC Enforcer          │
└──────────────────┬──────────────────────────────┘
                   │
         ┌─────────┴─────────┐
         ▼                   ▼
    SQLite DB           GitHub Repo
    (Local)             (Remote)
```

---

## 🛠️ CLI Commands

```bash
# Project
airdb init <name>              # Create project
airdb status                   # Project status
airdb info                     # Detailed health

# Migrations
airdb migrate create <name>    # Create migration
airdb migrate push             # Apply pending
airdb migrate status           # Check status

# GitHub Sync
airdb auth login/logout        # Authentication
airdb sync setup --create      # Init repo
airdb sync push/pull           # Sync changes
airdb sync conflicts           # List conflicts
airdb sync resolve <file>      # Resolve conflict

# API
airdb serve                    # Start REST API

# Team
airdb invite <user> --role <role>
airdb keys create <name>       # API key

# Updates
airdb update check/apply       # Self-update
```

---

## 🔐 Security

- **Tokens**: Stored in OS keyring (Keychain/DPAPI/Secret Service)
- **API Keys**: SHA-256 hashed
- **Updates**: Ed25519 signature verification
- **RBAC**: Enforced at engine level

See [Security Guide](docs/security.md) for deployment best practices.

---

## 🧪 Development

```bash
# Prerequisites: Node.js, Rust, Tauri CLI

# Frontend
npm install
npm run dev

# Backend
cd src-tauri
cargo build --release
cargo test

# Desktop App
npm run tauri build
```

---

## 📝 License

MIT License - see [LICENSE](LICENSE)

---

## 🙏 Built With

- [Tauri](https://tauri.app/) - Desktop framework
- [Rust](https://www.rust-lang.org/) - Core engine
- [React](https://react.dev/) - UI framework
- [SQLite](https://www.sqlite.org/) - Database
- [git2-rs](https://github.com/rust-lang/git2-rs) - Git operations

---

<p align="center">
  <strong>Made with ❤️ by the AirLabs Team</strong><br>
  <a href="https://github.com/Codeenk/airdb/issues">Report Bug</a> •
  <a href="https://github.com/Codeenk/airdb/discussions">Request Feature</a>
</p>
