# WebKit Symbol Lookup Error Fix
**Last Updated:** 2026-02-12  
**Issue:** `undefined symbol: __libc_pthread_init, version GLIBC_PRIVATE`  

---

## 🔴 Problem

When running `npm run tauri dev`, WebKit crashes with:

```
/usr/lib/x86_64-linux-gnu/webkit2gtk-4.1/WebKitNetworkProcess: symbol lookup error: 
/snap/core20/current/lib/x86_64-linux-gnu/libpthread.so.0: undefined symbol: __libc_pthread_init, version GLIBC_PRIVATE
```

### Root Cause

Snap packages (especially `core20`, `core22`, and GTK-related snaps) interfere with system libraries by adding their paths to `LD_LIBRARY_PATH`. When WebKit tries to load libraries, it picks up incompatible versions from snap directories instead of system libraries.

## Solutions

### Solution 1: Use the Development Launcher Script (Recommended)

Run the application using the provided wrapper script:
```bash
cd src-tauri
./run-dev.sh
```

This script:
- Removes snap paths from `LD_LIBRARY_PATH`
- Clears snap-related environment variables
- Forces use of system GTK and WebKit libraries
- Runs the development server with a clean environment

### Solution 2: Manual Environment Cleanup

Before running the application, clean your environment:
```bash
# Remove snap paths from LD_LIBRARY_PATH
export LD_LIBRARY_PATH=$(echo "$LD_LIBRARY_PATH" | tr ':' '\n' | grep -v '/snap/' | tr '\n' ':' | sed 's/:$//')

# Clear snap variables
unset SNAP SNAP_NAME SNAP_REVISION SNAP_LIBRARY_PATH

# Set system library paths
export GTK_PATH=/usr/lib/x86_64-linux-gnu/gtk-3.0
export WEBKIT_DISABLE_COMPOSITING_MODE=1

# Run the application
npm run tauri dev
```

### Solution 3: Unset LD_LIBRARY_PATH Completely

If you don't need custom library paths:
```bash
unset LD_LIBRARY_PATH
npm run tauri dev
```

### Solution 4: System-Wide Fix

If you have snap packages that are interfering with many applications, consider:

1. Check which snap packages are installed:
```bash
snap list
```

2. Remove unused snap packages or core snaps if not needed:
```bash
sudo snap remove <package-name>
```

3. Or modify your shell profile (~/.bashrc or ~/.zshrc) to always filter snap paths:
```bash
# Add this to your shell profile
if [ -n "$LD_LIBRARY_PATH" ]; then
    export LD_LIBRARY_PATH=$(echo "$LD_LIBRARY_PATH" | tr ':' '\n' | grep -v '/snap/' | tr '\n' ':' | sed 's/:$//')
fi
```

## Verification

After applying the fix, you should see:
- No "symbol lookup error" messages
- WebKit loads successfully
- The application window opens normally

## Files Created

- `run-dev.sh` - Main development launcher with environment cleanup
- `env-setup.sh` - Environment configuration script
- `WEBKIT_FIX.md` - This documentation file

## Additional Notes

The following GTK warnings are harmless and can be ignored:
```
Gtk-Message: Failed to load module "colorreload-gtk-module"
Gtk-Message: Failed to load module "window-decorations-gtk-module"
Gtk-Message: Failed to load module "appmenu-gtk-module"
```

These are optional GTK modules and don't affect functionality.
