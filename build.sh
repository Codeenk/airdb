#!/bin/bash
# Build script for AirDB v0.2.1

cd /home/grim/Documents/Linux_projects/newtest/airdb

echo "Building frontend..."
npm run build

echo "Building Rust binaries..."
cd src-tauri

echo "Building airdb-cli..."
cargo build --release --bin airdb-cli

echo "Building airdb-bootstrap..."
cargo build --release --bin airdb-bootstrap  

echo "Building airdb-desktop..."
cargo build --release --bin airdb-desktop

echo "Staging sidecar binaries for Tauri..."
TARGET_TRIPLE=$(rustc -vV | sed -n 's/^host: //p' | tr -d '\r')
BIN_EXT=""
if [[ "$TARGET_TRIPLE" == *"windows"* ]]; then
	BIN_EXT=".exe"
fi

mkdir -p bin
cp "target/release/airdb-cli$BIN_EXT" "bin/airdb-cli-$TARGET_TRIPLE$BIN_EXT"
cp "target/release/airdb-bootstrap$BIN_EXT" "bin/airdb-bootstrap-$TARGET_TRIPLE$BIN_EXT"

if [[ "$BIN_EXT" == "" ]]; then
	chmod +x "bin/airdb-cli-$TARGET_TRIPLE" "bin/airdb-bootstrap-$TARGET_TRIPLE"
fi

echo "✓ Build complete!"
echo "Binaries are in: src-tauri/target/release/"
ls -lh target/release/airdb-* 2>/dev/null | grep -v ".d$"
