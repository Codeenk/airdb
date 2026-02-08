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

echo "✓ Build complete!"
echo "Binaries are in: src-tauri/target/release/"
ls -lh target/release/airdb-* 2>/dev/null | grep -v ".d$"
