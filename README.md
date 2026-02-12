# 🗄️ AirDB

**The local-first, GitHub-backed database platform for modern developers**

[![CI](https://github.com/Codeenk/airdb/actions/workflows/ci.yml/badge.svg)](https://github.com/Codeenk/airdb/actions/workflows/ci.yml)
[![Release](https://img.shields.io/github/v/release/Codeenk/airdb?color=22d3ee&label=latest%20release)](https://github.com/Codeenk/airdb/releases/latest)
[![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg)](LICENSE)
[![Platform](https://img.shields.io/badge/platform-Linux%20%7C%20macOS%20%7C%20Windows-lightgrey)](https://github.com/Codeenk/airdb/releases)

---

## 🎯 What is AirDB?

AirDB is a **database development platform** that combines the simplicity of SQLite with professional-grade tooling. Design databases visually, version schema like code with Git, and ship production-ready APIs — all in one beautiful desktop application.

**One tool. Zero configuration. Full ownership.**

### Why AirDB?

| Feature | AirDB | Traditional Tools |
|---------|-------|------------------|
| **Local-First** | ✅ SQLite on your machine, zero latency | ❌ Cloud-dependent, slow queries |
| **Git-Backed Schema** | ✅ Every change versioned in your repo | ❌ Manual migration scripts |
| **Visual Schema Designer** | ✅ Drag-and-drop tables, ER diagrams | ❌ Raw SQL or clunky GUIs |
| **Auto REST API** | ✅ Instant production endpoints | ❌ Write boilerplate by hand |
| **Migration Safety** | ✅ Reversible migrations, rollback support | ❌ One-way migration pain |
| **Team Collaboration** | ✅ Branch isolation, conflict resolution | ❌ Database schema hell |
| **Audit Trail** | ✅ Built-in change tracking | ❌ Custom logging required |

---

## 🚀 What's New in v0.8.0

This is our biggest release yet — a complete UX overhaul with production-grade features.

### 🎨 Redesigned Sidebar Experience
- **Auto-collapse icons-only mode** — maximizes screen space
- **Hover to expand** — smooth overlay with labels
- **Pin mode** (`Ctrl+B`) — keep sidebar open when needed
- **Visible active indicators** — cyan accent bars
- **Optimized icon sizing** — 20px with perfect centering

### 📊 Visual Migration Manager
Complete migration timeline with:
- **Applied vs Pending** visual separation
- **Inline SQL preview** — see every migration's code
- **Create migrations** directly from the UI
- **Schema snapshots** — instant database state capture
- **Stats dashboard** — applied/pending/total counts

### 📈 Enhanced Dashboard
New at-a-glance insights:
- **Project banner** with DB type, size, and port
- **Quick stats** — tables, pending migrations, total rows, DB size
- **Quick actions** — one-click navigation to all tools
- **Table overview** — row counts sorted by size
- **Schema health** — migration warnings and checks
- **Recent activity** — audit log entries with timestamps

### 🌐 Integrated API Server
- **Start/Stop API** from Settings — no CLI needed
- **Port configuration** — custom ports with conflict detection
- **Status indicators** — running state with visual badges
- **Auto-generated Swagger UI** — instant API documentation

### 🔍 Audit & Observability
Comprehensive change tracking:
- **Audit log** for all data operations (insert, update, delete)
- **Activity feed** in Dashboard — see what changed and when
- **Health dashboard** — schema metrics and warnings
- **Resource usage** — database size, row counts, table stats

### ⌨️ Productivity Features
**Command Palette** (`Ctrl+K` or `Ctrl+P`):
- VS Code-style quick actions
- Search commands with keyboard navigation
- Grouped by Navigate and Actions

**Keyboard Shortcuts**:
- `Ctrl+1-7` — Navigate pages instantly
- `Ctrl+,` — Open Settings
- `Ctrl+B` — Toggle sidebar pin
- `Escape` — Close modals/palette

**Loading States**:
- Skeleton loaders instead of spinners
- Shimmer animations for tables and stats
- Progressive content reveal
- Optimized perceived performance

**Notification Center**:
- Bell icon in topbar with badge count
- Slide-out notification drawer
- Auto-populated from pending migrations
- Dismiss individual or clear all

---

## 📥 Installation

### Linux / macOS

```bash
# Download and extract
curl -L https://github.com/Codeenk/airdb/releases/latest/download/airdb-0.8.0-linux-x64.tar.gz | tar -xz
cd airdb-0.8.0-linux-x64

# Install (user mode) or use sudo for system-wide
./install.sh

# Verify
airdb --version
```

### Windows

1. Download [`airdb-0.8.0-windows-x64.zip`](https://github.com/Codeenk/airdb/releases/latest/download/airdb-0.8.0-windows-x64.zip)
2. Extract to a folder (e.g., `C:\Program Files\AirDB`)
3. Add the `bin\` folder to your system PATH
4. Run `airdb --version` to verify

See [Installation Guide](docs/installation.md) for detailed instructions.

---

##🏃 Quick Start

### 1. Launch the Desktop App

```bash
# Start AirDB
airdb-desktop

# Or from CLI
airdb init my-project
cd my-project
airdb serve
```

### 2. Create Your Schema Visually

1. Click **Tables** in the sidebar
2. Click **+ New Table**
3. Add columns, set types, define constraints
4. AirDB **auto-generates a migration** for you

### 3. Apply & Sync to GitHub

```bash
# Apply migrations locally
airdb migrate push

# Sync schema to GitHub (first time)
airdb sync setup --create
airdb sync push -m "Initial schema"
```

### 4. Your API is Ready

The API server starts automatically on `:54321`:

```bash
# All your tables automatically get endpoints
GET    /api/users
POST   /api/users
GET    /api/users/:id
PATCH  /api/users/:id
DELETE /api/users/:id

# Interactive docs
http://localhost:54321/swagger-ui
```

---

## 🎨 Desktop App Features

### Visual Schema Designer
- Drag-and-drop table creation
- Foreign key relationships with visual connectors
- Index manager with performance hints
- Constraint editor (CHECK, UNIQUE, DEFAULT)
- ER diagram with React Flow

### Data Browser
- DataGrid with inline editing
- Row inspector with JSON view
- Bulk operations (insert, update, delete)
- Export to CSV/JSON/SQL
- Advanced filtering and sorting

### SQL Editor
- CodeMirror 6 with autocomplete
- Multiple tabs for queries
- Results grid with pagination
- EXPLAIN visualizer
- Saved queries library

### Migration Dashboard
- Timeline view of all migrations
- Pending vs Applied visual separation
- Inline SQL preview
- Create migrations with custom names
- Generate schema snapshots
- Rollback support

### NoSQL Browser
- JSON document storage
- Collection management
- Query by ID or filter
- Syntax-highlighted preview
- Import/export collections

### Settings Hub
8 specialized tabs:
1. **General** — Project name, description, auto-start
2. **Database** — Connection settings, engine selection
3. **API Server** — Port config, Start/Stop controls
4. **GitHub** — Sync settings, branch config
5. **Security** — API keys, RBAC, authentication
6. **Migrations** — Auto-apply, conflict resolution
7. **Appearance** — Theme, sidebar, editor preferences
8. **Updates** — Version check, auto-update, rollback

---

## 🛠️ CLI Reference

| Command | Description |
|---------|-------------|
| `airdb init <name>` | Create new project |
| `airdb status` | Show project status, pending migrations |
| `airdb migrate create <name>` | Generate timestamped migration file |
| `airdb migrate push` | Apply all pending migrations |
| `airdb migrate rollback [count]` | Rollback N migrations (default: 1) |
| `airdb migrate list` | List all migrations with status |
| `airdb sync setup --create` | Initialize GitHub remote |
| `airdb sync push -m "message"` | Sync changes to GitHub |
| `airdb sync pull` | Pull team changes and auto-merge |
| `airdb serve [--port 54321]` | Start REST API server |
| `airdb auth login` | GitHub device flow authentication |
| `airdb update check` | Check for new AirDB releases |
| `airdb update apply` | Download and install updates |
| `airdb update rollback` | Revert to previous version |

---

## 🏗️ Architecture

```
┌─────────────────────────────────────────────────┐
│           Desktop App (Tauri + React)           │
│  ┌──────────┐  ┌──────────┐  ┌──────────────┐  │
│  │ Visual   │  │ SQL      │  │ Migration    │  │
│  │ Designer │  │ Editor   │  │ Manager      │  │
│  └──────────┘  └──────────┘  └──────────────┘  │
└────────────────────┬────────────────────────────┘
                     │ IPC Commands
         ┌───────────▼────────────┐
         │   Rust Engine Core     │
         │  ┌──────────────────┐  │
         │  │ DatabaseAdapter  │  │ ◄─── Trait-based multi-DB
         │  │ (SQLite/PG/...)  │  │
         │  └──────────────────┘  │
         │  ┌──────────────────┐  │
         │  │ Migration Runner │  │ ◄─── Safe schema evolution
         │  └──────────────────┘  │
         │  ┌──────────────────┐  │
         │  │ Axum REST API    │  │ ◄─── Auto-generated endpoints
         │  └──────────────────┘  │
         │  ┌──────────────────┐  │
         │  │ Audit Logger     │  │ ◄─── Change tracking
         │  └──────────────────┘  │
         └───────────┬────────────┘
                     │
       ┌─────────────┼─────────────┐
       ▼             ▼             ▼
  ┏━━━━━━━┓   ┏━━━━━━━━━┓   ┏━━━━━━━━━┓
  ┃ SQLite┃   ┃  GitHub ┃   ┃  Logs   ┃
  ┗━━━━━━━┛   ┗━━━━━━━━━┛   ┗━━━━━━━━━┛
```

### Tech Stack

**Frontend**:
- React 19.1.0 + TypeScript 5.8.3
- Vite 7.0.4 for blazing-fast builds
- CodeMirror 6 for SQL editing
- React Flow for ER diagrams
- Lucide React for icons
- Custom Void Cyan dark theme

**Backend**:
- Rust (edition 2021)
- Tauri 2.10.2 for desktop shell
- rusqlite 0.32 with connection pooling
- Axum 0.8 for REST API
- git2 0.19 for GitHub sync
- tokio 1 (full async runtime)
- clap 4 for CLI parsing

---

## 📚 Documentation

| Category | Guide | Description |
|----------|-------|-------------|
| **Getting Started** | [**Introduction**](docs/introduction.md) | High-level overview and concepts |
| | [**Quick Start**](docs/quickstart.md) | 5-minute hands-on tutorial |
| | [**Installation**](docs/installation.md) | Platform-specific setup guides |
| **Core Concepts** | [**SQL Guide**](docs/sql-guide.md) | Tables, relations, indexes |
| | [**NoSQL Guide**](docs/nosql-guide.md) | JSON document storage |
| | [**Migrations**](docs/migrations.md) | Schema versioning deep-dive |
| **Operations** | [**CLI Reference**](docs/cli-reference.md) | Complete command documentation |
| | [**Updates & Rollback**](docs/updates-and-rollback.md) | Version management |
| | [**Security**](docs/security.md) | API keys, RBAC, best practices |
| **Collaboration** | [**Team Workflows**](docs/team-workflows.md) | Branching, merging, sync |
| | [**Conflict Resolution**](docs/conflict-resolution.md) | Handling schema conflicts |
| **Support** | [**FAQ**](docs/faq.md) | Common questions answered |

---

## 🌟 Use Cases

### Solo Developers
- **Rapid prototyping** with visual schema designer
- **Instant APIs** without writing boilerplate
- **Git-backed backups** — never lose schema evolution history

### Startups & Small Teams
- **Local-first development** — fast iteration, no cloud costs
- **Branch isolation** — each feature gets its own schema branch
- **Painless migrations** — reversible, tested, version-controlled

### Education
- **Visual schema learning** — see relationships in ER diagrams
- **Safe experimentation** — rollback any change instantly
- **Complete audit trail** — review every database operation

### Production Apps
- **Type-safe migrations** — auto-generated, never manually edited
- **Zero-downtime deploys** — test migrations on staging branches
- **Built-in monitoring** — audit logs, health checks, resource metrics

---

## 🤝 Contributing

We welcome contributions! Check out our [Contributing Guide](CONTRIBUTING.md) and [Code of Conduct](CODE_OF_CONDUCT.md).

### Development Setup

```bash
# Clone the repo
git clone https://github.com/Codeenk/airdb.git
cd airdb

# Install dependencies
npm install
cargo build

# Dev mode (hot reload)
npm run tauri dev

# Production build
npm run tauri build
```

---

## 🗺️ Roadmap

- [ ] **PostgreSQL Adapter** — multi-database engine support
- [ ] **MySQL Adapter** — expand database compatibility
- [ ] **AI Query Assistant** — natural language to SQL
- [ ] **GraphQL API** — in addition to REST
- [ ] **Real-time Sync** — WebSocket-based collaboration
- [ ] **Schema Templates** — starter templates for common use cases
- [ ] **Advanced RBAC** — role-based access control for APIs
- [ ] **Analytics Dashboard** — query performance insights

---

## 📝 License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.

---

## 💬 Community & Support

- **Issues**: [GitHub Issues](https://github.com/Codeenk/airdb/issues)
- **Discussions**: [GitHub Discussions](https://github.com/Codeenk/airdb/discussions)
- **Discord**: [Join our community](https://discord.gg/airdb) *(coming soon)*
- **Docs**: [docs.airdb.dev](https://docs.airdb.dev) *(coming soon)*

---

<p align="center">
  <strong>Built with ❤️ by developers, for developers</strong><br>
  <sub>AirDB is open-source and always will be.</sub>
</p>

<p align="center">
  <a href="https://github.com/Codeenk/airdb/stargazers">⭐ Star us on GitHub</a> •
  <a href="https://github.com/Codeenk/airdb/releases">📦 Download Latest</a> •
  <a href="https://github.com/Codeenk/airdb/discussions">💬 Join Discussion</a>
</p>
