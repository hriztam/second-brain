# justfile for second-brain
# Install just: brew install just
# Usage: just <recipe>

# Default recipe: show available commands
default:
    @just --list

# Build the Rust daemon in release mode
build-daemon:
    cd daemon && cargo build --release

# Build the Rust daemon in debug mode
build-daemon-debug:
    cd daemon && cargo build

# Build the macOS app (requires Xcode)
build-app:
    cd SecondBrainUI && xcodebuild -project SecondBrainUI.xcodeproj -scheme SecondBrainUI -configuration Release build

# Build the macOS app in debug mode
build-app-debug:
    cd SecondBrainUI && xcodebuild -project SecondBrainUI.xcodeproj -scheme SecondBrainUI -configuration Debug build

# Build everything
build: build-daemon build-app

# Run the daemon (foreground, for development)
run-daemon:
    cd daemon && RUST_LOG=debug cargo run

# Check daemon code without building
check-daemon:
    cd daemon && cargo check

# Run daemon tests
test-daemon:
    cd daemon && cargo test

# Install daemon binary and launchd plist
install-daemon:
    ./scripts/install-daemon.sh

# Uninstall daemon
uninstall-daemon:
    ./scripts/uninstall-daemon.sh

# Check daemon status
daemon-status:
    @launchctl list | grep com.secondbrain.daemon || echo "Daemon not loaded"

# View daemon logs
daemon-logs:
    @tail -f /tmp/second-brain-daemon.out.log /tmp/second-brain-daemon.err.log

# Clean build artifacts
clean:
    cd daemon && cargo clean
    rm -rf SecondBrainUI/build SecondBrainUI/DerivedData

# Format Rust code
fmt:
    cd daemon && cargo fmt

# Lint Rust code
lint:
    cd daemon && cargo clippy -- -D warnings
