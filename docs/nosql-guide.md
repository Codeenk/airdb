# NoSQL Guide

AirDB supports JSON document collections alongside SQL tables.

## Creating Collections

```bash
airdb nosql init posts
```

Creates:
```
nosql/posts/
├── _meta.json        # Collection metadata
├── _schema.v1.json   # Schema definition
├── migrations/       # Schema migrations
└── .data/            # Documents
```

## Inserting Documents

```bash
# From file
airdb nosql insert posts --file post.json

# From stdin
echo '{"title": "Hello", "body": "World"}' | airdb nosql insert posts --stdin

# Via API
curl -X POST http://localhost:54321/api/nosql/posts \
  -H "Content-Type: application/json" \
  -d '{"title": "Hello", "body": "World"}'
```

Documents get auto-generated ULID IDs:
```json
{
  "_id": "01HQXR5JVKZ8QXRT4KXMG0NPWD",
  "title": "Hello",
  "body": "World",
  "_created": "2024-01-15T10:30:00Z"
}
```

## Querying

```bash
# List all
airdb nosql list posts

# Query with filter
airdb nosql query posts --where '{"author": "alice"}'

# Via API
GET /api/nosql/posts?author=alice&limit=10
```

## Schema Definition

Define schema in `_schema.v1.json`:
```json
{
  "version": 1,
  "fields": {
    "title": { "type": "string", "required": true },
    "body": { "type": "string" },
    "author": { "type": "string", "required": true },
    "tags": { "type": "array", "items": { "type": "string" } },
    "published": { "type": "boolean", "default": false }
  },
  "allow_additional": false
}
```

## Schema Migrations

```bash
airdb nosql migrate create posts add_category
```

Creates `migrations/002_add_category.json`:
```json
{
  "version": 2,
  "name": "add_category",
  "operations": [
    {
      "type": "add_field",
      "name": "category",
      "field_type": "string",
      "required": false,
      "default": "uncategorized"
    }
  ]
}
```

Apply:
```bash
airdb nosql migrate apply posts
```

## Hybrid SQL + NoSQL

Link NoSQL documents to SQL rows:

```bash
airdb hybrid link users posts --on user_id
```

Query with includes:
```json
{
  "query": "SELECT * FROM users WHERE id = 1",
  "include": {
    "posts": {
      "collection": "posts",
      "on": { "user_id": "id" }
    }
  }
}
```

Result:
```json
{
  "data": [{ "id": 1, "name": "Alice" }],
  "included": {
    "posts": [
      { "_id": "...", "title": "Hello", "user_id": 1 }
    ]
  }
}
```

## Field Types

| Type | Description |
|------|-------------|
| `string` | UTF-8 text |
| `number` | Integer or float |
| `boolean` | true/false |
| `array` | JSON array |
| `object` | Nested object |
| `datetime` | ISO 8601 |

## Best Practices

1. **Define schemas** - Catch errors early
2. **Use migrations** - Never edit schema directly
3. **Prefer SQL for relations** - NoSQL for flexible data
4. **Index frequently queried fields** - In schema definition
