# FAQ

## General

### What is AirDB?
AirDB is a local-first database platform that stores data in Git. It combines SQLite for relational data and JSON for flexible documents, with sync via GitHub.

### Why use Git for databases?
- **History**: Every change is versioned
- **Collaboration**: Branch, merge, PR workflows
- **Backup**: GitHub is your backup
- **Portability**: Clone anywhere and it works

### Is AirDB free?
Yes, AirDB is open source under MIT license. GitHub storage limits apply.

---

## Data & Storage

### Where is my data stored?
- **Local**: In your project directory (`local.db` + `nosql/`)
- **Remote**: GitHub repository (private by default)

### How big can my database be?
GitHub recommends repos under 1GB. For larger datasets, consider:
- Splitting into multiple repos
- Using Git LFS for binary data
- External storage with AirDB as metadata

### Can I use it offline?
Yes! AirDB is local-first. Sync when you're back online.

---

## Migrations

### Why are migrations mandatory?
Migrations ensure:
- Schema consistency across environments
- Reviewable changes (PRs)
- Rollback capability
- Team synchronization

### Can I edit the database directly?
Not recommended. Direct edits bypass migration history and cause drift.

### How do I undo a migration?
```bash
airdb migrate rollback
```

---

## Updates

### Will updates break my data?
No. Updates:
1. Are versioned and tested
2. Include automatic rollback
3. Create backups before schema changes

### What if an update fails?
Automatic rollback restores the previous version. Your data is safe.

### Can I skip updates?
Yes, but security fixes may require updates. `stable` channel only gets tested releases.

---

## Team Workflows

### How do multiple people work on the same project?
1. Clone the repo
2. Branch for features
3. Make changes locally
4. Push / PR for review
5. Merge applies migrations

### What if two people add the same table?
Conflict detection on PR. Rename or coordinate.

### What about concurrent writes?
AirDB is designed for dev/staging workflows, not high-concurrency production. For production, export to a traditional database.

---

## Security

### Is my data encrypted?
- **At rest**: Use OS-level encryption (FileVault, BitLocker)
- **GitHub**: Private repos, encrypted storage
- **API**: API keys required for remote access

### Who can access my database?
- **Local**: Anyone with filesystem access
- **GitHub**: Repo collaborators only
- **API**: Users with valid API keys

---

## Troubleshooting

### "Migration conflict detected"
Two migrations have the same version. Renumber one.

### "Database locked"
Another process is using the database. Close other AirDB instances.

### "Update blocked"
A migration or backup is running. Wait for completion.

### Command not found: airdb
Restart terminal or check PATH. See [Installation](installation.md).
