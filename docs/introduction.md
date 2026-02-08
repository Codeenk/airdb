# AirDB

**Local-first, GitHub-backed database platform for developers.**

AirDB lets you build database-backed applications that work offline-first, sync via Git, and deploy anywhere.

## ✨ Key Features

- **SQL + NoSQL** - Use SQLite for relational data, JSON collections for flexible documents
- **GitHub-Native** - Your database lives in Git, with automatic sync and conflict resolution
- **Migrations First** - All schema changes are versioned, reviewable, and reversible
- **Self-Updating** - Automatic updates with rollback protection
- **Team-Ready** - RBAC, branch isolation, and merge safety built-in

## 🚀 Quick Start

```bash
# Initialize a new project
airdb init --name myproject

# Create a migration
airdb migrate create add_users_table

# Start local API server
airdb serve

# Push to GitHub
airdb sync push
```

## 📖 Documentation

- [Installation](docs/installation.md)
- [Quick Start Guide](docs/quickstart.md)
- [CLI Reference](docs/cli-reference.md)
- [SQL Guide](docs/sql-guide.md)
- [NoSQL Guide](docs/nosql-guide.md)
- [Migrations](docs/migrations.md)
- [Updates & Rollback](docs/updates-and-rollback.md)
- [Team Workflows](docs/team-workflows.md)
- [Security Model](docs/security.md)
- [FAQ](docs/faq.md)

## 💡 Why AirDB?

| Traditional DB | AirDB |
|---------------|-------|
| Server required | Works offline |
| Manual backups | Git = automatic history |
| Schema drift | Migrations enforce consistency |
| Update anxiety | Automatic rollback on failure |
| Permission complexity | Simple RBAC with GitHub teams |

## 🛠 Development

```bash
# Build from source
cargo build --release

# Run tests
cargo test

# Run with dev hot-reload
npm run tauri dev
```

## 📝 License

MIT License - see [LICENSE](LICENSE) for details.
