# Getting Started with AirDB

This guide will walk you through setting up your first AirDB project, creating migrations, and syncing with GitHub.

---

## Prerequisites

- **Operating System:** Linux, macOS, or Windows
- **GitHub Account:** Required for sync features
- **Git:** Installed and configured

---

## Installation

### Linux

**Option 1: AppImage (Recommended)**
```bash
# Download from releases
wget https://github.com/Codeenk/airdb/releases/latest/download/airdb_0.1.0_amd64.AppImage

# Make executable
chmod +x airdb_0.1.0_amd64.AppImage

# Run
./airdb_0.1.0_amd64.AppImage
```

**Option 2: Debian/Ubuntu Package**
```bash
wget https://github.com/Codeenk/airdb/releases/latest/download/airdb_0.1.0_amd64.deb
sudo dpkg -i airdb_0.1.0_amd64.deb
```

### macOS
```bash
# Download and install
curl -LO https://github.com/Codeenk/airdb/releases/latest/download/AirDB_0.1.0_aarch64.dmg
open AirDB_0.1.0_aarch64.dmg
```

### Windows
Download `AirDB_0.1.0_x64_en-US.msi` from [releases](https://github.com/Codeenk/airdb/releases) and run the installer.

---

## Step 1: Initialize Your First Project

```bash
# Create a new project
airdb init my-database

# Navigate to the project
cd my-database
```

This creates the following structure:
```
my-database/
├── airdb.config.json    # Project configuration
├── sql/
│   └── migrations/      # Migration files go here
├── access/              # Team access configuration
├── api/                 # Generated API specs
└── data/                # SQLite database files
```

---

## Step 2: Create Your First Migration

```bash
airdb migrate create create_users_table
```

This generates a file like `sql/migrations/001_create_users_table.sql`.

Edit the file:
```sql
-- up
CREATE TABLE users (
  id INTEGER PRIMARY KEY AUTOINCREMENT,
  username TEXT NOT NULL UNIQUE,
  email TEXT NOT NULL UNIQUE,
  created_at DATETIME DEFAULT CURRENT_TIMESTAMP
);

CREATE INDEX idx_users_email ON users(email);

-- down
DROP INDEX idx_users_email;
DROP TABLE users;
```

> **Note:** The `-- up` section runs when applying the migration. The `-- down` section runs when rolling back.

---

## Step 3: Apply the Migration

```bash
airdb migrate push
```

Expected output:
```
✅ Applied migration: 001_create_users_table.sql
   Database: data/app.db
```

Verify the migration:
```bash
airdb status
```

---

## Step 4: Authenticate with GitHub

```bash
airdb auth login
```

You'll see:
```
🔐 Starting GitHub authentication...

📋 Copy this code: A1B2-C3D4

🌐 Open: https://github.com/login/device

⏳ Waiting for authorization...
```

1. Copy the code
2. Open the URL in your browser
3. Paste the code and authorize

Once complete:
```
✅ Authenticated as YourUsername
   Token stored securely in OS keyring
```

---

## Step 5: Set Up GitHub Sync

```bash
airdb sync setup --create
```

This will:
1. Create a new private GitHub repository
2. Initialize git in your project
3. Add the remote origin
4. Create initial `.gitignore`

---

## Step 6: Push Your Schema to GitHub

```bash
airdb sync push -m "Initial schema: users table"
```

Expected output:
```
📤 Committing changes...
   Added: sql/migrations/001_create_users_table.sql
   
🚀 Pushing to GitHub...
   ✅ Pushed to origin/main
```

Visit your GitHub repository to see the synced schema!

---

## Step 7: Start the API Server

```bash
airdb serve
```

Output:
```
🚀 AirDB API Server
   Project: my-database
   Listening: http://localhost:54321
   Press Ctrl+C to stop
```

Open http://localhost:54321/swagger-ui to explore the auto-generated REST API.

---

## Next Steps

### Create More Migrations
```bash
airdb migrate create add_posts_table
# Edit the migration file
airdb migrate push
airdb sync push -m "Add posts table"
```

### Invite Team Members
```bash
airdb invite alice --role developer
```

### Generate API Keys
```bash
airdb keys create production-api --role readonly
```

### Pull Changes from Collaborators
```bash
airdb sync pull
```

---

## Common Workflows

### Making Schema Changes
```bash
# 1. Create migration
airdb migrate create add_column_to_users

# 2. Edit migration file
# 3. Apply locally
airdb migrate push

# 4. Test changes
airdb serve

# 5. Push to GitHub
airdb sync push -m "Add bio column to users"
```

### Collaborating with Team
```bash
# Pull latest changes
airdb sync pull

# If conflicts occur
airdb sync conflicts
airdb sync resolve migration.sql --ours
airdb sync push
```

---

## Troubleshooting

### "Not authenticated" error
```bash
airdb auth status
# If not authenticated:
airdb auth login
```

### Migration conflicts
See [Conflict Resolution Guide](conflict-resolution.md)

### Database locked
Ensure no other processes are using the database:
```bash
# Check for running API server
ps aux | grep airdb
```

---

## Desktop UI Alternative

If you prefer a graphical interface:

1. Launch the AirDB desktop app
2. Click "Create New Project"
3. Follow the on-screen wizard
4. Use the UI to create migrations and sync

The desktop app provides the same functionality as the CLI with a visual interface.

---

## What's Next?

- **[Conflict Resolution Guide](conflict-resolution.md)** - Handle merge conflicts
- **[Security Best Practices](security.md)** - Secure your deployment
- **[CLI Reference](../README.md#-cli-commands)** - Full command list

---

**Need help?** Open an issue on [GitHub](https://github.com/Codeenk/airdb/issues).
