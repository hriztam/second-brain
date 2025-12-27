#!/bin/bash
# Uninstall second-brain daemon
set -euo pipefail

PLIST_DEST="$HOME/Library/LaunchAgents/com.secondbrain.daemon.plist"
BINARY_PATH="/usr/local/bin/second-brain-daemon"
SOCKET_PATH="$HOME/.local/share/second-brain/daemon.sock"

echo "=== Uninstalling second-brain daemon ==="

# Unload from launchd
if launchctl list | grep -q com.secondbrain.daemon; then
    echo "Stopping daemon..."
    launchctl bootout "gui/$(id -u)" "$PLIST_DEST" 2>/dev/null || true
fi

# Remove plist
if [[ -f "$PLIST_DEST" ]]; then
    echo "Removing launchd plist..."
    rm -f "$PLIST_DEST"
fi

# Remove binary
if [[ -f "$BINARY_PATH" ]]; then
    echo "Removing binary (requires sudo)..."
    sudo rm -f "$BINARY_PATH"
fi

# Remove socket if it exists
if [[ -e "$SOCKET_PATH" ]]; then
    echo "Removing socket file..."
    rm -f "$SOCKET_PATH"
fi

echo "✓ Daemon uninstalled"
