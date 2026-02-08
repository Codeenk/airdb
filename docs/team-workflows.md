# Team Workflows

> **How multiple developers work together on AirDB projects**

## 📋 Table of Contents

- [Branch Isolation](#branch-isolation)
- [Merge Safety](#merge-safety)
- [Conflict Resolution](#conflict-resolution)
- [Role-Based Access Control](#role-based-access-control)
- [Best Practices](#best-practices)

---

## Branch Isolation

AirDB provides **full branch isolation** — each developer works in their own database state.

### How It Works

```bash
# Developer A creates a feature branch
git checkout -b feature/user-profiles

# Their local AirDB automatically switches to branch context
airdb status
# Shows: Branch: feature/user-profiles

# Changes are isolated until merged
airdb migrate create add_profile_fields
airdb migrate push
```

### Branch Context

AirDB tracks which branch you're on and isolates:

- **Migration state** - Applied migrations are branch-specific
- **Data preview** - Test data doesn't leak between branches
- **Lock state** - Operations block merges safely

### Switching Branches

```bash
git checkout main
# AirDB automatically:
# 1. Detects branch switch
# 2. Loads main branch migration state
# 3. Warns if migrations need syncing
```

---

## Merge Safety

AirDB prevents destructive merges with **automatic conflict detection**.

### Pre-Merge Checks

Before merging, AirDB verifies:

1. ✅ **Migration Compatibility** - No conflicting schema changes
2. ✅ **Lock State** - No active operations blocking merge
3. ✅ **Rollback Safety** - Down migrations exist

### Example: Safe Merge

```bash
# On feature branch
git checkout feature/add-comments
airdb migrate create add_comments_table
airdb migrate push

# Switch to main
git checkout main
git pull origin main

# Merge
git merge feature/add-comments
# AirDB automatically:
# - Validates migration order
# - Applies pending migrations
# - Updates version tracking
```

### Example: Conflict Detection

```bash
# Developer A: adds "email" column to users
# Developer B: also adds "email" column to users

# When B tries to merge:
git merge feature/add-email
# ❌ AirDB Error:
# Migration conflict detected:
#   - 001_add_email_to_users.sql (branch: feature/add-email)
#   - 001_add_user_email.sql (branch: main)
# Both modify table 'users', column 'email'
# Run: airdb sync conflicts
```

---

## Conflict Resolution

When conflicts occur, AirDB provides tools to resolve them safely.

### Step 1: List Conflicts

```bash
airdb sync conflicts
# Output:
# Conflicted migrations:
#   1. 001_add_email_to_users.sql
#   2. 002_add_user_roles.sql
# Conflicted files:
#   - sql/migrations/001_add_email_to_users.sql
```

### Step 2: Choose Resolution Strategy

**Option A: Use Local Version**

```bash
airdb sync resolve sql/migrations/001_add_email_to_users.sql --ours
```

**Option B: Use Remote Version**

```bash
airdb sync resolve sql/migrations/001_add_email_to_users.sql --theirs
```

**Option C: Manual Resolution**

```bash
# Edit the file manually
nano sql/migrations/001_add_email_to_users.sql

# Mark as resolved
airdb sync resolve sql/migrations/001_add_email_to_users.sql
```

### Step 3: Verify & Push

```bash
# Test migrations locally
airdb migrate check

# Push resolved state
airdb sync push --message "Resolved email field conflict"
```

---

## Role-Based Access Control

AirDB enforces access control for team members.

### Roles

| Role | Permissions |
|------|-------------|
| **Owner** | Full access, invite users, manage keys |
| **Developer** | Read/write, create migrations, push changes |
| **Read-Only** | View schema, run queries, no writes |

### Inviting Team Members

```bash
# Invite a developer
airdb invite john --role developer

# Invite read-only collaborator
airdb invite jane --role readonly
```

### API Key Management

```bash
# Create a deployment key
airdb keys create --name "Production API" --role readonly

# List all keys
airdb keys list

# Revoke a key
airdb keys revoke key_abc123
```

---

## Best Practices

### ✅ Do's

1. **Always create migrations** - Never edit schema directly
2. **Test locally first** - Run `airdb migrate check` before pushing
3. **Small, atomic changes** - One migration = one logical change
4. **Descriptive names** - `add_user_email` not `migration_001`
5. **Review PRs carefully** - Check both SQL and down migrations
6. **Use branch isolation** - Test features in separate branches

### ❌ Don'ts

1. **Don't skip down migrations** - Every up needs a down
2. **Don't edit applied migrations** - Create new ones instead
3. **Don't force push** - Destroys migration history
4. **Don't merge without testing** - Run migrations locally first
5. **Don't bypass conflict resolution** - Resolve properly

---

## Example: Full Team Workflow

### Scenario: Two Developers Add Features

**Developer A** (adds comments feature):

```bash
git checkout -b feature/comments
airdb migrate create add_comments_table
# Edit migration...
airdb migrate push
git push origin feature/comments
# Open PR
```

**Developer B** (adds likes feature):

```bash
git checkout -b feature/likes
airdb migrate create add_likes_table
# Edit migration...
airdb migrate push
git push origin feature/likes
# Open PR
```

**Maintainer** (merges both):

```bash
# Review PR #1 (comments)
git checkout feature/comments
airdb migrate check  # ✅ Pass
git checkout main
git merge feature/comments
airdb migrate push

# Review PR #2 (likes)
git checkout feature/likes
git pull origin main  # Sync with merged comments
airdb migrate check  # ✅ Pass (no conflicts)
git checkout main
git merge feature/likes
airdb migrate push
```

---

## Troubleshooting

### "Migration conflict detected"

**Cause**: Two branches modified the same table/column  
**Solution**: Use conflict resolution tools or manually merge migrations

### "Operation locked"

**Cause**: Another operation (serve, backup, migrate) is running  
**Solution**: Wait for operation to finish or stop it safely

### "Schema drift detected"

**Cause**: Local database doesn't match migration history  
**Solution**: Run `airdb migrate check` and apply pending migrations

---

## Next Steps

- [Conflict Resolution Guide](conflict-resolution.md)
- [Security Model](security.md)
- [Migrations Deep Dive](migrations.md)
