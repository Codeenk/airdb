# Security Best Practices

AirDB is designed with security in mind. This guide covers best practices for secure deployment and usage.

---

## Authentication & Authorization

### GitHub OAuth

AirDB uses **OAuth Device Flow** for GitHub authentication:

✅ **Secure:**
- No password storage
- Token-based authentication
- Automatic token refresh

❌ **Never:**
- Share your device code
- Store tokens in plain text
- Commit tokens to repositories

### Token Storage

Tokens are stored in your OS keyring:

| OS | Storage Location |
|----|------------------|
| **macOS** | Keychain |
| **Windows** | Credential Manager |
| **Linux** | Secret Service (gnome-keyring/kwallet) |

**Verify token storage:**
```bash
airdb auth status
```

---

## API Key Management

### Creating API Keys

```bash
# Generate a new API key
airdb keys create production-api --role readonly

# Output:
# 🔑 API Key Created
#    Key: airdb_1a2b3c4d5e6f7g8h9i0j
#    ID: abc123
#    Role: readonly
#
# ⚠️  Save this key securely - it won't be shown again!
```

### Key Security Rules

1. **Never commit API keys to Git**
   ```bash
   # Add to .gitignore
   echo "*.env" >> .gitignore
   echo ".airdb/keys.json" >> .gitignore
   ```

2. **Use environment variables**
   ```bash
   export AIRDB_API_KEY="airdb_1a2b3c4d5e6f7g8h9i0j"
   ```

3. **Rotate keys regularly**
   ```bash
   # Revoke old key
   airdb keys revoke abc123
   
   # Create new key
   airdb keys create production-api-v2 --role readonly
   ```

4. **Use least privilege**
   - `readonly` for read-only access
   - `developer` for write access
   - `admin` only when necessary

---

## Role-Based Access Control (RBAC)

### Available Roles

| Role | Permissions | Use Case |
|------|-------------|----------|
| `admin` | Full repository access | Project owners |
| `developer` | Push/pull, create migrations | Active developers |
| `readonly` | Pull-only, read API | CI/CD, monitoring |

### Inviting Team Members

```bash
# Invite with appropriate role
airdb invite alice --role developer
airdb invite bob --role readonly
```

### Access File

Team access is tracked in `access/team.json`:
```json
{
  "members": [
    {
      "username": "alice",
      "role": "developer",
      "added_at": "2026-02-06T12:00:00Z"
    }
  ]
}
```

**Security tip:** Review this file regularly to audit team access.

---

## Database Security

### SQLite Best Practices

1. **File Permissions**
   ```bash
   # Restrict database access
   chmod 600 data/app.db
   ```

2. **WAL Mode** (enabled by default)
   - Better concurrency
   - Atomic commits
   - Crash recovery

3. **Backup Strategy**
   ```bash
   # Regular backups
   sqlite3 data/app.db ".backup data/backup.db"
   ```

### Sensitive Data

**Never store in migrations:**
- Passwords (even hashed)
- API keys
- Personal identifiable information (PII)
- Credit card numbers

**Use environment variables instead:**
```sql
-- ❌ Bad
INSERT INTO config VALUES ('api_key', 'secret123');

-- ✅ Good
-- Store reference only, load from env at runtime
INSERT INTO config VALUES ('api_key_ref', 'ENV:API_KEY');
```

---

## Network Security

### API Server

When running `airdb serve`:

1. **Bind to localhost only** (default)
   ```bash
   airdb serve --host 127.0.0.1
   ```

2. **Use HTTPS in production**
   ```bash
   # Behind reverse proxy (nginx/caddy)
   airdb serve --host 127.0.0.1 --port 8080
   ```

3. **Enable authentication**
   ```bash
   # Require API key for all requests
   airdb serve --require-auth
   ```

### Firewall Rules

```bash
# Linux (ufw)
sudo ufw allow from 192.168.1.0/24 to any port 54321

# macOS (pf)
# Add to /etc/pf.conf:
# pass in proto tcp from 192.168.1.0/24 to any port 54321
```

---

## GitHub Repository Security

### Repository Settings

1. **Make repositories private** (default)
   ```bash
   airdb sync setup --create --private
   ```

2. **Enable branch protection**
   - Go to GitHub → Settings → Branches
   - Protect `main` branch
   - Require pull request reviews

3. **Enable security alerts**
   - Settings → Security & analysis
   - Enable Dependabot alerts

### Secrets Management

**Never commit:**
- `.airdb/keys.json` (API keys)
- `.env` files
- Database files (`data/*.db`)

**Use `.gitignore`:**
```gitignore
# AirDB
data/*.db
data/*.db-*
.airdb/keys.json
*.env
*.log
```

---

## Deployment Security

### Production Checklist

- [ ] Use HTTPS for all connections
- [ ] Enable API key authentication
- [ ] Restrict database file permissions
- [ ] Use read-only API keys for monitoring
- [ ] Enable GitHub branch protection
- [ ] Set up automated backups
- [ ] Review team access regularly
- [ ] Rotate API keys quarterly
- [ ] Monitor access logs
- [ ] Keep AirDB updated

### Environment Variables

```bash
# .env (never commit!)
AIRDB_API_KEY=airdb_prod_key_here
AIRDB_DB_PATH=/secure/path/to/app.db
AIRDB_LOG_LEVEL=warn
```

Load in production:
```bash
source .env
airdb serve --require-auth
```

---

## Incident Response

### If API Key is Compromised

```bash
# 1. Immediately revoke
airdb keys revoke <key-id>

# 2. Generate new key
airdb keys create emergency-key --role readonly

# 3. Update all services
# 4. Review access logs
# 5. Notify team
```

### If GitHub Token is Compromised

```bash
# 1. Logout
airdb auth logout

# 2. Revoke token on GitHub
# Go to: https://github.com/settings/tokens

# 3. Re-authenticate
airdb auth login

# 4. Review repository access
```

---

## Compliance

### GDPR Considerations

If storing EU user data:
- Document data retention policies
- Implement data deletion procedures
- Provide data export functionality
- Maintain audit logs

### Audit Logging

Enable detailed logging:
```bash
export AIRDB_LOG_LEVEL=info
airdb serve > access.log 2>&1
```

Review logs regularly:
```bash
grep "API_KEY" access.log
grep "ERROR" access.log
```

---

## Security Updates

### Stay Updated

```bash
# Check current version
airdb --version

# Update to latest
# Download from: https://github.com/Codeenk/airdb/releases
```

### Security Advisories

Monitor:
- [GitHub Security Advisories](https://github.com/Codeenk/airdb/security/advisories)
- [Release Notes](https://github.com/Codeenk/airdb/releases)

---

## Reporting Security Issues

**Found a vulnerability?**

**DO NOT** open a public issue.

Instead:
1. Email: security@airdb.dev (if available)
2. Or use [GitHub Security Advisories](https://github.com/Codeenk/airdb/security/advisories/new)

We'll respond within 48 hours.

---

## Additional Resources

- [OWASP Top 10](https://owasp.org/www-project-top-ten/)
- [SQLite Security](https://www.sqlite.org/security.html)
- [GitHub Security Best Practices](https://docs.github.com/en/code-security)

---

**Security is a shared responsibility. Stay vigilant!** 🔒
