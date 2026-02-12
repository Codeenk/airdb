#!/bin/bash

# AirDB Clean Environment Configuration
# This script provides a clean environment for running AirDB without snap interference

# Function to clean library paths
clean_library_paths() {
    if [ -n "$LD_LIBRARY_PATH" ]; then
        echo "$LD_LIBRARY_PATH" | tr ':' '\n' | grep -v '/snap/' | tr '\n' ':' | sed 's/:$//'
    fi
}

# Function to check for problematic snap packages
check_snap_conflicts() {
    if command -v snap &> /dev/null; then
        echo "Checking for potentially conflicting snap packages..."
        snap list 2>/dev/null | grep -E '(core20|core22|gnome|gtk)'
    fi
}

# Export cleaned environment
export LD_LIBRARY_PATH=$(clean_library_paths)

# Clear snap-related variables that cause WebKit conflicts
unset SNAP SNAP_NAME SNAP_REVISION SNAP_LIBRARY_PATH SNAP_INSTANCE_NAME
unset SNAP_USER_COMMON SNAP_USER_DATA SNAP_COMMON SNAP_DATA
unset SNAP_INSTANCE_KEY SNAP_COOKIE

# Force use of system libraries
export GTK_PATH=/usr/lib/x86_64-linux-gnu/gtk-3.0
export GDK_BACKEND=x11
export WEBKIT_DISABLE_COMPOSITING_MODE=1
export WEBKIT_DISABLE_DMABUF_RENDERER=1

# Add system library paths explicitly (prepend to ensure priority)
export LD_LIBRARY_PATH="/usr/lib/x86_64-linux-gnu:/lib/x86_64-linux-gnu:${LD_LIBRARY_PATH}"

# Remove any remaining snap paths from LD_LIBRARY_PATH
export LD_LIBRARY_PATH=$(echo "$LD_LIBRARY_PATH" | sed 's|/snap/[^:]*:||g' | sed 's|:/snap/[^:]*||g' | sed 's|^/snap/[^:]*:||')

# Print environment info for debugging
echo "==================================="
echo "AirDB Environment Configuration"
echo "==================================="
echo "LD_LIBRARY_PATH: ${LD_LIBRARY_PATH:-<not set>}"
echo "GTK_PATH: ${GTK_PATH:-<not set>}"
echo "GDK_BACKEND: ${GDK_BACKEND:-<not set>}"
echo "==================================="
echo ""

# Check for conflicts
check_snap_conflicts

echo ""
echo "Starting AirDB..."
echo "==================================="
