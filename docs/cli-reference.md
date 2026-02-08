# CLI Reference

Complete command reference for the `airdb` CLI.

## Global Options

| Flag | Description |
|------|-------------|
| `-p, --project <DIR>` | Project directory (default: current) |
| `-f, --format <FORMAT>` | Output format: `text` or `json` |
| `--help` | Show help |
| `--version` | Show version |

## Commands

### Project Management

#### `airdb init`
Initialize a new AirDB project.

```bash
airdb init --name myproject
airdb init --name myproject --visibility public
airdb init --name myproject --no-github  # Skip GitHub setup
```

#### `airdb status`
Show project status.

```bash
airdb status
# 📊 AirDB Project Status
#    Project: myproject
#    Migrations: 5 applied, 0 pending
#    Tables: users, posts, comments
```

#### `airdb info`
Detailed project information.

```bash
airdb info
airdb info --format json  # Machine-readable
```

---

### Migrations

#### `airdb migrate create <name>`
Create a new migration file.

```bash
airdb migrate create add_users_table
# Created: migrations/001_add_users_table.sql
```

#### `airdb migrate push`
Apply pending migrations and push to GitHub.

```bash
airdb migrate push
# ✓ Applied 2 migrations
# ✓ Pushed to main
```

#### `airdb migrate check`
Check migration status without applying.

```bash
airdb migrate check
# Pending: 2 migrations
```

#### `airdb migrate rollback [COUNT]`
Rollback last N migrations.

```bash
airdb migrate rollback      # Rollback 1
airdb migrate rollback 3    # Rollback 3
```

#### `airdb migrate list`
List all migrations with status.

```bash
airdb migrate list
```

---

### API Server

#### `airdb serve`
Start local REST API server.

```bash
airdb serve                     # Default: 127.0.0.1:54321
airdb serve --port 8080
airdb serve --host 0.0.0.0      # Allow external access
```

---

### NoSQL

#### `airdb nosql init <collection>`
Initialize a NoSQL collection.

```bash
airdb nosql init posts
```

#### `airdb nosql list`
List all collections.

#### `airdb nosql insert <collection>`
Insert a document.

```bash
airdb nosql insert posts --file post.json
echo '{"title": "Hello"}' | airdb nosql insert posts --stdin
```

#### `airdb nosql migrate create <collection> <name>`
Create a schema migration.

```bash
airdb nosql migrate create posts add_author_field
```

---

### GitHub Sync

#### `airdb sync setup`
Set up GitHub repository.

```bash
airdb sync setup              # Link existing repo
airdb sync setup --create     # Create new repo
```

#### `airdb sync push`
Push changes to GitHub.

```bash
airdb sync push
airdb sync push --message "Add feature"
```

#### `airdb sync pull`
Pull changes from GitHub.

```bash
airdb sync pull
```

#### `airdb sync status`
Check sync status.

---

### Updates

#### `airdb update check`
Check for updates.

```bash
airdb update check
# ✔ Current: 0.1.0
# ⬆ Available: 0.2.0
```

#### `airdb update apply`
Apply pending update.

```bash
airdb update apply
```

---

### Authentication

#### `airdb auth login`
Authenticate with GitHub.

#### `airdb auth logout`
Sign out.

#### `airdb auth status`
Show auth status.

---

### API Keys

#### `airdb keys list`
List API keys.

#### `airdb keys create`
Create new API key.

```bash
airdb keys create --name "production" --role admin
```

#### `airdb keys revoke <id>`
Revoke an API key.

---

## JSON Output

Add `--format json` for machine-readable output:

```bash
airdb status --format json | jq '.migrations.pending'
```
