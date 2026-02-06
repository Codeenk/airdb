# Conflict Resolution Guide

When multiple team members work on database schemas simultaneously, merge conflicts can occur. AirDB provides tools to detect and resolve these conflicts safely.

---

## Understanding Schema Conflicts

### What Causes Conflicts?

Conflicts happen when:
1. Two developers modify the same migration file
2. Migrations are created in different orders on different machines
3. Schema changes are incompatible (e.g., both add a column with the same name)

### Example Scenario

**Developer A:**
```sql
-- migration: 003_add_user_fields.sql
ALTER TABLE users ADD COLUMN phone TEXT;
```

**Developer B** (working simultaneously):
```sql
-- migration: 003_add_user_fields.sql  
ALTER TABLE users ADD COLUMN mobile TEXT;
```

When Developer B pulls Developer A's changes, Git detects a conflict.

---

## Detecting Conflicts

### After Pulling Changes

```bash
airdb sync pull
```

If conflicts exist, you'll see:
```
⚠️  Merge conflicts detected
   Files with conflicts:
   - sql/migrations/003_add_user_fields.sql
   
Use `airdb sync conflicts` to list all conflicts
Use `airdb sync resolve <file> --ours|--theirs` to resolve
```

### List All Conflicts

```bash
airdb sync conflicts
```

Output:
```
⚠️  Merge Conflicts Detected:
   ❌ sql/migrations/003_add_user_fields.sql
   ❌ sql/migrations/004_add_posts.sql

Use `airdb sync resolve <FILE> --ours` or `--theirs` to resolve.
```

---

## Resolution Strategies

### Option 1: Keep Your Version (`--ours`)

Use when your local changes should take precedence:

```bash
airdb sync resolve sql/migrations/003_add_user_fields.sql --ours
```

This keeps **your local version** and discards the remote changes.

**When to use:**
- You know your schema is correct
- Remote changes were experimental
- You're the schema owner for this table

### Option 2: Accept Remote Version (`--theirs`)

Use when the remote version should be used:

```bash
airdb sync resolve sql/migrations/003_add_user_fields.sql --theirs
```

This **accepts the remote version** and discards your local changes.

**When to use:**
- Remote changes are more complete
- You want to align with team decisions
- Your local changes were experimental

### Option 3: Manual Merge

For complex conflicts, manually edit the file:

```bash
# Open the conflicted file
vim sql/migrations/003_add_user_fields.sql
```

You'll see conflict markers:
```sql
-- up
<<<<<<< HEAD (ours)
ALTER TABLE users ADD COLUMN phone TEXT;
=======
ALTER TABLE users ADD COLUMN mobile TEXT;
>>>>>>> origin/main (theirs)

-- down
<<<<<<< HEAD
ALTER TABLE users DROP COLUMN phone;
=======
ALTER TABLE users DROP COLUMN mobile;
>>>>>>> origin/main
```

**Resolve manually:**
```sql
-- up
ALTER TABLE users ADD COLUMN phone TEXT;
ALTER TABLE users ADD COLUMN mobile TEXT;

-- down
ALTER TABLE users DROP COLUMN mobile;
ALTER TABLE users DROP COLUMN phone;
```

Then stage the resolved file:
```bash
git add sql/migrations/003_add_user_fields.sql
```

---

## Complete Workflow

### Step-by-Step Resolution

```bash
# 1. Pull latest changes
airdb sync pull

# 2. Check for conflicts
airdb sync conflicts

# 3. Resolve each conflict
airdb sync resolve sql/migrations/003_add_user_fields.sql --ours

# 4. Verify resolution
airdb sync conflicts
# Should show: ✅ No merge conflicts detected

# 5. Test migrations locally
airdb migrate push

# 6. Push resolved changes
airdb sync push -m "Resolved migration conflicts"
```

---

## Best Practices

### 1. Communicate with Your Team

Before making schema changes:
- Announce in team chat
- Check if others are working on the same tables
- Coordinate migration numbering

### 2. Pull Before Creating Migrations

```bash
# Always pull first
airdb sync pull

# Then create your migration
airdb migrate create add_new_feature
```

### 3. Use Descriptive Migration Names

**Bad:**
```bash
airdb migrate create update_users
```

**Good:**
```bash
airdb migrate create add_email_verification_to_users
```

### 4. Keep Migrations Small

One logical change per migration:
- ✅ `add_user_avatar_column.sql`
- ✅ `create_posts_table.sql`
- ❌ `update_everything.sql`

### 5. Test Before Pushing

```bash
# Apply migration locally
airdb migrate push

# Test with API
airdb serve

# Verify schema
sqlite3 data/app.db ".schema users"

# Push only if tests pass
airdb sync push -m "Add avatar column"
```

---

## Advanced Scenarios

### Conflicting Migration Numbers

If two developers create migration `003` simultaneously:

**Resolution:**
1. Rename one migration to the next number:
   ```bash
   mv sql/migrations/003_add_posts.sql sql/migrations/004_add_posts.sql
   ```

2. Update the migration order in your database
3. Push the renamed migration

### Incompatible Schema Changes

If migrations conflict logically (e.g., both drop the same column):

1. Coordinate with the other developer
2. Decide on the correct approach
3. Create a new migration that reconciles both changes
4. Document the decision in the migration comment

---

## Conflict Prevention

### Use Feature Branches (Advanced)

```bash
# Create a feature branch
git checkout -b feature/add-comments

# Make changes
airdb migrate create add_comments_table
airdb migrate push

# Push to feature branch
git push origin feature/add-comments

# Create PR for review before merging
```

### Establish Migration Ownership

Assign table ownership:
- **Alice:** `users`, `auth` tables
- **Bob:** `posts`, `comments` tables
- **Charlie:** `analytics` tables

This reduces conflicts by design.

---

## Troubleshooting

### "Cannot push: conflicts exist"

```bash
# Check for unresolved conflicts
airdb sync conflicts

# Resolve all conflicts first
airdb sync resolve <file> --ours

# Then push
airdb sync push
```

### "Migration already applied"

If a migration was applied locally but conflicts remotely:

```bash
# Check migration status
airdb status

# If needed, rollback local migration
# (Manual SQL rollback or restore from backup)

# Then pull and resolve
airdb sync pull
```

---

## Getting Help

If you're unsure how to resolve a conflict:

1. **Don't panic** - conflicts are normal in team workflows
2. **Ask your team** - discuss the best resolution strategy
3. **Review the changes** - use `git diff` to see what changed
4. **Test locally** - always test before pushing

---

**Related Guides:**
- [Getting Started](getting-started.md)
- [Security Best Practices](security.md)
- [CLI Reference](../README.md#-cli-commands)
