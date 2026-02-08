# Updates & Rollback

AirDB safely updates itself with automatic rollback protection.

## How Updates Work

```
┌─────────────┐    ┌─────────────┐    ┌─────────────┐
│   Check     │ →  │  Download   │ →  │   Verify    │
│  (GitHub)   │    │  (Binary)   │    │ (Checksum)  │
└─────────────┘    └─────────────┘    └─────────────┘
                                            ↓
┌─────────────┐    ┌─────────────┐    ┌─────────────┐
│   Done ✓    │ ←  │   Health    │ ←  │   Switch    │
│             │    │   Check     │    │  (Restart)  │
└─────────────┘    └─────────────┘    └─────────────┘
```

## Checking for Updates

```bash
airdb update check
# ✔ Current: 0.1.0
# ⬆ Available: 0.2.0
#   • Fixes NoSQL migration edge case
#   • Improves rollback safety
```

Or in the UI: **Settings → Updates → Check for Updates**

## Applying Updates

```bash
airdb update apply
# ⬇ Downloading 0.2.0...
# ✔ Verified checksum
# ⏳ Restart required to complete
```

The update applies on next restart.

## Automatic Rollback

If the new version fails to start correctly:

1. **Health check fails** - New version doesn't respond
2. **Rollback triggered** - Previous version restored
3. **User notified** - UI shows rollback reason

```
⚠️ Automatic Rollback
The update failed to start correctly and was 
automatically rolled back to version 0.1.0.
Reason: Health check timeout after 30s
```

## Update Channels

| Channel | Description |
|---------|-------------|
| `stable` | Production-ready (default) |
| `beta` | New features in testing |
| `nightly` | Bleeding edge |

Change channel:
```bash
airdb update channel beta
```

## Operation Locks

Updates are blocked during critical operations:

| Operation | Update Blocked? |
|-----------|----------------|
| Migration running | ✅ Yes |
| Backup in progress | ✅ Yes |
| API server running | ✅ Yes |
| Branch preview active | ✅ Yes |
| Idle | ❌ No |

The update will wait until operations complete.

## Manual Rollback

If needed:
```bash
airdb update rollback
# ✔ Rolled back to 0.1.0
```

## Update Files

AirDB stores update state in:
```
~/.airdb/
├── state.json        # Update state machine
├── versions/         # Downloaded binaries
│   ├── 0.1.0/
│   └── 0.2.0/
└── current           # Symlink to active version
```

## Offline Mode

Updates require internet. On offline:
- Update check skipped
- Current version continues working
- Next online → check resumes
