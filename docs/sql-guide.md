# SQL Guide

AirDB uses SQLite for relational data, exposed via migrations and REST API.

## Creating Tables

Always use migrations:

```bash
airdb migrate create add_users
```

```sql
-- Up
CREATE TABLE users (
  id INTEGER PRIMARY KEY AUTOINCREMENT,
  email TEXT UNIQUE NOT NULL,
  name TEXT,
  role TEXT DEFAULT 'user',
  created_at DATETIME DEFAULT CURRENT_TIMESTAMP
);

-- Down
DROP TABLE users;
```

## Data Types

| Type | Description |
|------|-------------|
| `INTEGER` | 64-bit integer |
| `REAL` | 64-bit float |
| `TEXT` | UTF-8 string |
| `BLOB` | Binary data |
| `DATETIME` | ISO 8601 string |

## Indexes

```sql
-- Single column
CREATE INDEX idx_users_email ON users(email);

-- Composite
CREATE INDEX idx_posts_user_date ON posts(user_id, created_at);

-- Unique
CREATE UNIQUE INDEX idx_users_email ON users(email);
```

## Foreign Keys

```sql
CREATE TABLE posts (
  id INTEGER PRIMARY KEY,
  user_id INTEGER NOT NULL REFERENCES users(id) ON DELETE CASCADE,
  title TEXT NOT NULL
);
```

## REST API

Start the server:
```bash
airdb serve
```

### List Tables
```bash
GET /api/tables
# ["users", "posts"]
```

### Query Rows
```bash
GET /api/tables/users?limit=10&offset=0
```

Response:
```json
{
  "data": [
    {"id": 1, "email": "alice@example.com", "name": "Alice"}
  ],
  "count": 1,
  "limit": 10,
  "offset": 0
}
```

### Insert Row
```bash
POST /api/tables/users
Content-Type: application/json

{"email": "bob@example.com", "name": "Bob"}
```

### Update Row
```bash
PUT /api/tables/users/1
Content-Type: application/json

{"name": "Bob Smith"}
```

### Delete Row
```bash
DELETE /api/tables/users/1
```

## Common Patterns

### Soft Delete
```sql
ALTER TABLE users ADD COLUMN deleted_at DATETIME;

-- Query active users
SELECT * FROM users WHERE deleted_at IS NULL;
```

### Timestamps
```sql
CREATE TABLE posts (
  id INTEGER PRIMARY KEY,
  created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
  updated_at DATETIME DEFAULT CURRENT_TIMESTAMP
);

-- Update trigger
CREATE TRIGGER update_posts_timestamp 
AFTER UPDATE ON posts
BEGIN
  UPDATE posts SET updated_at = CURRENT_TIMESTAMP WHERE id = NEW.id;
END;
```

### JSON Column
```sql
CREATE TABLE events (
  id INTEGER PRIMARY KEY,
  type TEXT NOT NULL,
  payload TEXT  -- Store JSON as TEXT
);
```
