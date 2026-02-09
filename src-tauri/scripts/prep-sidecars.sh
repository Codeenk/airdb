#!/bin/bash
set -e

# Get host triple
TRIPLE=$(rustc -vV | grep host | cut -d ' ' -f 2)
echo "Preparing sidecars for $TRIPLE..."

# Create directory
mkdir -p src-tauri/bin

# Create dummy files to satisfy tauri-build check
touch "src-tauri/bin/airdb-cli-$TRIPLE"
touch "src-tauri/bin/airdb-bootstrap-$TRIPLE"

# Build release binaries
cargo build --manifest-path src-tauri/Cargo.toml --release --bin airdb-cli --bin airdb-bootstrap

# Copy and rename to sidecar format
mkdir -p src-tauri/bin
cp src-tauri/target/release/airdb-cli "src-tauri/bin/airdb-cli-$TRIPLE"
cp src-tauri/target/release/airdb-bootstrap "src-tauri/bin/airdb-bootstrap-$TRIPLE"

echo "Sidecars prepared in src-tauri/bin/"
ls -l src-tauri/bin/
