# Migrations

Migrations are versioned SQL scripts that evolve your database schema safely.

## Why Migrations?

| Without Migrations | With Migrations |
|-------------------|-----------------|
| Schema drift between environments | Consistent everywhere |
| "It works on my machine" | Reproducible history |
| Risky production changes | Reviewed before deploy |
| No rollback option | Every change is reversible |

## Creating Migrations

```bash
airdb migrate create add_posts_table
# Created: migrations/002_add_posts_table.sql
```

Edit the file:
```sql
-- Up
CREATE TABLE posts (
  id INTEGER PRIMARY KEY AUTOINCREMENT,
  title TEXT NOT NULL,
  body TEXT,
  user_id INTEGER REFERENCES users(id),
  created_at DATETIME DEFAULT CURRENT_TIMESTAMP
);

CREATE INDEX idx_posts_user ON posts(user_id);

-- Down
DROP INDEX idx_posts_user;
DROP TABLE posts;
```

## Applying Migrations

```bash
airdb migrate push
```

This:
1. Runs pending migrations
2. Records in `_migrations` table
3. Commits to Git
4. Pushes to GitHub

## Rollback

```bash
# Rollback last migration
airdb migrate rollback

# Rollback last 3
airdb migrate rollback 3
```

## Migration Best Practices

### 1. One Change Per Migration
```sql
-- GOOD: Single purpose
-- 002_add_posts_table.sql

-- BAD: Multiple unrelated changes
-- 002_add_posts_and_update_users_and_add_comments.sql
```

### 2. Always Write Down Migrations
```sql
-- Down
DROP TABLE posts;  -- Enables rollback
```

### 3. Use Transactions (Automatic)
AirDB wraps each migration in a transaction.

### 4. Test Locally First
```bash
airdb migrate push  # Local first
airdb sync push     # Then to GitHub
```

## NoSQL Migrations

NoSQL collections also use migrations:

```bash
airdb nosql migrate create posts add_author_field
```

Creates `nosql/posts/migrations/001_add_author_field.json`:
```json
{
  "version": 1,
  "name": "add_author_field",
  "operations": [
    {
      "type": "add_field",
      "name": "author",
      "field_type": "string",
      "required": false
    }
  ]
}
```

## Squashing Migrations

For cleaner history:

```bash
airdb migrate squash --before 010
# Combines 001-009 into a single migration
```

> ⚠️ Only squash migrations already deployed to all environments.
