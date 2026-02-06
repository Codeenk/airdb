# AirDB

**Local-first, GitHub-backed database platform for developers.**

AirDB combines the simplicity of SQLite with the collaboration power of GitHub. Develop locally with a real database, push your schema and data changes to GitHub to collaborate with your team, and auto-generate instant REST APIs.

## 🚀 Features

- **Local-First**: Built on SQLite. Fast, zero-latency local development.
- **Git-Backed**: Schema migrations and access policies are versioned in Git.
- **Collaboration**: Sync changes via GitHub. Handle merge conflicts with ease.
- **Instant API**: Auto-generated REST API for your database.
- **Access Control**: Role-based access control (RBAC) with JSON policies.
- **Cross-Platform**: CLI and Desktop app for Windows, macOS, and Linux.

## 📦 Installation

### From Source

Ensure you have Rust and Cargo installed.

```bash
git clone https://github.com/Codeenk/airdb
cd airdb
cargo install --path src-tauri
```

### Binary Releases

Download the latest release for your platform from the [Releases](https://github.com/Codeenk/airdb/releases) page.

## 🛠️ Usage

### 1. Authentication

Authenticate with your GitHub account to enable sync features.

```bash
airdb auth login
```

### 2. Create a Project

Initialize a new AirDB project in your workspace.

```bash
airdb init --name my-project
cd my-project
```

This creates a project structure with:
- `data/`: Your SQLite database
- `migrations/`: SQL migration files
- `access/`: Access control policies
- `.airdb/`: Project configuration

### 3. Database Schema

Create a migration to define your schema.

```bash
airdb migrate create initial_schema
```

Edit the generated SQL file in `migrations/`, then apply it:

```bash
airdb migrate push
```

### 4. Sync with GitHub

Connect your project to a GitHub repository to share it with your team.

```bash
# Create a new private repo (or link existing)
airdb sync setup --create

# Push local changes (schema, migrations, config)
airdb sync push --message "Add users table"

# Pull remote changes
airdb sync pull
```

#### Handling Conflicts
If a teammate has modified the same file, AirDB detects conflicts:

```bash
airdb sync conflicts
# Lists conflicted files

airdb sync resolve migrations/001_init.sql --ours
# OR
airdb sync resolve migrations/001_init.sql --theirs
```

### 5. Start the API Server

Launch the auto-generated REST API.

```bash
airdb serve
```

Your API is now live at `http://localhost:54321`.

## 🔒 Security

- **Tokens**: GitHub OAuth tokens are stored securely in your operating system's keyring (Keychain on macOS, Credential Manager on Windows, Secret Service on Linux).
- **Policies**: Access to data is governed by `access/policies.json`. You can define rigorous RBAC rules for your API.

## 🏗️ Architecture

AirDB acts as a bridge:

`Local SQLite` <-> `AirDB Engine` <-> `Git / GitHub`

1.  **Engine**: Rust-based core handling DB operations and Git sync.
2.  **CLI**: Command-line interface for all operations.
3.  **Desktop**: Tauri-based GUI for visual management (Coming Soon).

## 📄 License

MIT License
