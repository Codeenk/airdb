# AirDB

**Local-first, GitHub-backed database platform for schema management and API generation.**

[![CI](https://github.com/Codeenk/airdb/actions/workflows/ci.yml/badge.svg)](https://github.com/Codeenk/airdb/actions/workflows/ci.yml)
[![Release](https://github.com/Codeenk/airdb/releases/latest/badge.svg)](https://github.com/Codeenk/airdb/releases)
[![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg)](LICENSE)

---

## 🚀 Features

- **📦 Local-First SQLite** - Work offline, sync when ready
- **🔄 GitHub Sync** - Version control for your database schema
- **🔐 OAuth Authentication** - Secure device flow login
- **👥 Team Collaboration** - Invite collaborators with role-based access
- **⚔️ Conflict Resolution** - Smart merge conflict handling for schema changes
- **🔑 API Key Management** - Generate and rotate API keys
- **🖥️ Desktop UI** - Cross-platform Tauri app (Linux, macOS, Windows)
- **⚡ CLI** - Full-featured command-line interface
- **📡 Auto-Generated REST API** - OpenAPI spec + Swagger UI

---

## 📥 Installation

### Linux
```bash
# AppImage (portable)
chmod +x airdb_0.1.0_amd64.AppImage
./airdb_0.1.0_amd64.AppImage

# Debian/Ubuntu
sudo dpkg -i airdb_0.1.0_amd64.deb
```

### macOS
```bash
# Download and open the .dmg file
open AirDB_0.1.0_aarch64.dmg
```

### Windows
```powershell
# Run the MSI installer
.\AirDB_0.1.0_x64_en-US.msi
```

Download the latest release from [GitHub Releases](https://github.com/Codeenk/airdb/releases).

---

## ⚡ Quick Start

### 1. Initialize a Project
```bash
airdb init my-project
cd my-project
```

### 2. Create a Migration
```bash
airdb migrate create add_users_table
```

Edit the generated SQL file in `sql/migrations/`:
```sql
-- up
CREATE TABLE users (
  id INTEGER PRIMARY KEY,
  email TEXT NOT NULL UNIQUE,
  created_at DATETIME DEFAULT CURRENT_TIMESTAMP
);

-- down
DROP TABLE users;
```

### 3. Apply Migration
```bash
airdb migrate push
```

### 4. Authenticate with GitHub
```bash
airdb auth login
```

### 5. Sync to GitHub
```bash
airdb sync setup --create
airdb sync push -m "Add users table"
```

### 6. Start API Server
```bash
airdb serve
# API available at http://localhost:54321
# Swagger UI at http://localhost:54321/swagger-ui
```

---

## 📚 Documentation

- **[Getting Started Guide](docs/getting-started.md)** - Step-by-step tutorial
- **[Conflict Resolution](docs/conflict-resolution.md)** - Handling merge conflicts
- **[Security Best Practices](docs/security.md)** - Secure deployment guide

---

## 🛠️ CLI Commands

| Command | Description |
|---------|-------------|
| `airdb init <name>` | Initialize a new project |
| `airdb migrate create <name>` | Create a new migration |
| `airdb migrate push` | Apply pending migrations |
| `airdb auth login` | Authenticate with GitHub |
| `airdb auth logout` | Sign out |
| `airdb sync setup --create` | Initialize GitHub repository |
| `airdb sync push -m <msg>` | Push changes to GitHub |
| `airdb sync pull` | Pull changes from GitHub |
| `airdb sync conflicts` | List merge conflicts |
| `airdb sync resolve <file> --ours\|--theirs` | Resolve conflicts |
| `airdb invite <username> --role <role>` | Invite team member |
| `airdb keys create <name> --role <role>` | Generate API key |
| `airdb serve` | Start REST API server |

---

## 🏗️ Architecture

```
┌─────────────────────────────────────────────────┐
│              Desktop UI (Tauri)                 │
│         React + TypeScript + Vite               │
└──────────────────┬──────────────────────────────┘
                   │ IPC
┌──────────────────▼──────────────────────────────┐
│           Rust Engine (Core)                    │
│  • Migration Runner  • GitHub Sync              │
│  • SQLite WAL Mode   • OAuth Device Flow        │
│  • API Generator     • Keyring Storage          │
└──────────────────┬──────────────────────────────┘
                   │
         ┌─────────┴─────────┐
         ▼                   ▼
    SQLite DB          GitHub Repo
    (Local)            (Remote Sync)
```

---

## 🤝 Team Collaboration

AirDB supports team workflows:

1. **Invite collaborators:**
   ```bash
   airdb invite alice --role developer
   ```

2. **Role-based access:**
   - `admin` - Full repository access
   - `developer` - Push access
   - `readonly` - Pull-only access

3. **Conflict resolution:**
   ```bash
   airdb sync pull
   # If conflicts occur:
   airdb sync conflicts
   airdb sync resolve migration.sql --ours
   airdb sync push
   ```

---

## 🔐 Security

- **GitHub tokens** stored in OS keyring (Keychain/Credential Manager/Secret Service)
- **API keys** hashed with SHA-256
- **Role-based access control** for team members
- **OAuth device flow** - no password storage

See [Security Best Practices](docs/security.md) for deployment guidelines.

---

## 🧪 Development

### Build from Source
```bash
# Install dependencies
npm install

# Build CLI
cd src-tauri
cargo build --bin airdb-cli

# Build Desktop App
npm run tauri build
```

### Run Tests
```bash
cd src-tauri
cargo test
```

---

## 📝 License

MIT License - see [LICENSE](LICENSE) for details.

---

## 🙏 Acknowledgments

Built with:
- [Tauri](https://tauri.app/) - Desktop framework
- [Rust](https://www.rust-lang.org/) - Core engine
- [React](https://react.dev/) - UI framework
- [SQLite](https://www.sqlite.org/) - Database
- [git2-rs](https://github.com/rust-lang/git2-rs) - Git operations

---

**Made with ❤️ by the AirDB Team**
