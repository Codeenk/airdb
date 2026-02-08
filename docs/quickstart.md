# Quick Start

Create your first AirDB project in 5 minutes.

## 1. Initialize Project

```bash
mkdir myapp && cd myapp
airdb init --name myapp
```

This creates:
```
myapp/
├── airdb.json         # Project config
├── local.db          # SQLite database
├── migrations/       # SQL migrations
└── nosql/            # NoSQL collections
```

## 2. Create a Table

```bash
airdb migrate create add_users
```

Edit `migrations/001_add_users.sql`:
```sql
-- Up
CREATE TABLE users (
  id INTEGER PRIMARY KEY AUTOINCREMENT,
  email TEXT UNIQUE NOT NULL,
  name TEXT,
  created_at DATETIME DEFAULT CURRENT_TIMESTAMP
);

-- Down
DROP TABLE users;
```

Apply:
```bash
airdb migrate push
# ✓ Applied 1 migration
```

## 3. Start API Server

```bash
airdb serve
# 🚀 API running at http://localhost:54321
```

Now you have a REST API:
```bash
# Insert a user
curl -X POST http://localhost:54321/api/tables/users \
  -H "Content-Type: application/json" \
  -d '{"email": "alice@example.com", "name": "Alice"}'

# List users
curl http://localhost:54321/api/tables/users
```

## 4. Push to GitHub

```bash
airdb sync setup --create
# ✓ Created repo: github.com/you/myapp

airdb sync push --message "Add users table"
# ✓ Pushed to main
```

## 5. Add NoSQL Collection

Create a posts collection:
```bash
airdb nosql init posts
```

Insert documents:
```bash
airdb nosql insert posts --file post.json
# Or via API:
# POST http://localhost:54321/api/nosql/posts
```

## What's Next?

- [SQL Guide](sql-guide.md) - Tables, indexes, relationships
- [NoSQL Guide](nosql-guide.md) - Collections, queries, schemas
- [Migrations](migrations.md) - Schema versioning deep-dive
- [Team Workflows](team-workflows.md) - Collaboration patterns
