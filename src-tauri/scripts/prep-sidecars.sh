#!/bin/bash
set -e

ROOT_DIR=$(cd "$(dirname "$0")/../.." && pwd)

# Get host triple
TRIPLE=$(rustc -vV | sed -n 's/^host: //p' | tr -d '\r')
EXT=""
if [[ "$TRIPLE" == *"windows"* ]]; then
	EXT=".exe"
fi

echo "Preparing sidecars for $TRIPLE..."

mkdir -p "$ROOT_DIR/src-tauri/bin"

# Ensure frontendDist exists before building Rust binaries
if [[ ! -d "$ROOT_DIR/dist" ]]; then
	echo "dist/ missing, building frontend..."
	(cd "$ROOT_DIR" && npm run build)
fi

# Create dummy files to satisfy tauri-build check
touch "$ROOT_DIR/src-tauri/bin/airdb-cli-$TRIPLE$EXT"
touch "$ROOT_DIR/src-tauri/bin/airdb-bootstrap-$TRIPLE$EXT"

# Build release binaries
cargo build --manifest-path "$ROOT_DIR/src-tauri/Cargo.toml" --release --bin airdb-cli --bin airdb-bootstrap

# Copy and rename to sidecar format
cp "$ROOT_DIR/src-tauri/target/release/airdb-cli$EXT" "$ROOT_DIR/src-tauri/bin/airdb-cli-$TRIPLE$EXT"
cp "$ROOT_DIR/src-tauri/target/release/airdb-bootstrap$EXT" "$ROOT_DIR/src-tauri/bin/airdb-bootstrap-$TRIPLE$EXT"

if [[ "$EXT" == "" ]]; then
	chmod +x "$ROOT_DIR/src-tauri/bin/airdb-cli-$TRIPLE" "$ROOT_DIR/src-tauri/bin/airdb-bootstrap-$TRIPLE"
fi

echo "Sidecars prepared in src-tauri/bin/"
ls -l "$ROOT_DIR/src-tauri/bin/"
