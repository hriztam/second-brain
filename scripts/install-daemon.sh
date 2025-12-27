#!/bin/bash
# Install second-brain daemon as a LaunchAgent
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(dirname "$SCRIPT_DIR")"

DAEMON_BINARY="$PROJECT_ROOT/daemon/target/release/second-brain-daemon"
PLIST_SOURCE="$PROJECT_ROOT/daemon/resources/com.secondbrain.daemon.plist"
PLIST_DEST="$HOME/Library/LaunchAgents/com.secondbrain.daemon.plist"
INSTALL_DIR="/usr/local/bin"

echo "=== Installing second-brain daemon ==="

# Check if daemon binary exists
if [[ ! -f "$DAEMON_BINARY" ]]; then
    echo "Error: Daemon binary not found at $DAEMON_BINARY"
    echo "Run 'just build-daemon' first."
    exit 1
fi

# Unload existing daemon if running
if launchctl list | grep -q com.secondbrain.daemon; then
    echo "Stopping existing daemon..."
    launchctl bootout "gui/$(id -u)" "$PLIST_DEST" 2>/dev/null || true
fi

# Create install directory if needed
if [[ ! -d "$INSTALL_DIR" ]]; then
    echo "Creating $INSTALL_DIR (requires sudo)..."
    sudo mkdir -p "$INSTALL_DIR"
fi

# Copy binary
echo "Installing binary to $INSTALL_DIR..."
sudo cp "$DAEMON_BINARY" "$INSTALL_DIR/second-brain-daemon"
sudo chmod 755 "$INSTALL_DIR/second-brain-daemon"

# Create LaunchAgents directory if needed
mkdir -p "$HOME/Library/LaunchAgents"

# Copy plist
echo "Installing launchd plist..."
cp "$PLIST_SOURCE" "$PLIST_DEST"

# Load the daemon
echo "Loading daemon..."
launchctl bootstrap "gui/$(id -u)" "$PLIST_DEST"

# Verify it's running
sleep 1
if launchctl list | grep -q com.secondbrain.daemon; then
    echo "✓ Daemon installed and running"
    echo ""
    echo "Logs: tail -f /tmp/second-brain-daemon.*.log"
    echo "Stop: launchctl bootout gui/$(id -u) $PLIST_DEST"
else
    echo "✗ Daemon failed to start"
    echo "Check logs: cat /tmp/second-brain-daemon.err.log"
    exit 1
fi
