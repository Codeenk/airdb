#!/bin/bash
# Rebuild v0.2.1 release with corrected install script

set -e

cd /home/grim/Documents/Linux_projects/newtest/airdb

echo "Deleting old release (if exists)..."
gh release delete v0.2.1 --repo Codeenk/airdb -y 2>/dev/null || echo "Release already deleted or doesn't exist"

echo "Deleting old tag (if exists)..."
git tag -d v0.2.1 2>/dev/null || echo "Local tag already deleted"
git push origin :refs/tags/v0.2.1 2>/dev/null || echo "Remote tag already deleted"

echo "Creating new tag..."
git tag v0.2.1
git push origin v0.2.1

echo "Building binaries..."
cd src-tauri
cargo build --release --bin airdb-bootstrap
cargo build --release --bin airdb-cli  
cargo build --release --bin airdb-desktop

echo "Copying binaries to release directory..."
cd ..
mkdir -p release/0.2.1/linux
cp src-tauri/target/release/airdb-bootstrap release/0.2.1/linux/
cp src-tauri/target/release/airdb-cli release/0.2.1/linux/
cp src-tauri/target/release/airdb-desktop release/0.2.1/linux/

echo "Creating tarball..."
cd release/0.2.1
tar -czf airdb-0.2.1-linux.tar.gz linux/

echo "Creating checksums..."
cd linux
sha256sum airdb-bootstrap airdb-cli airdb-desktop > ../checksums-linux.txt
cd ..

echo "Creating GitHub release..."
gh release create v0.2.1 \
  --repo Codeenk/airdb \
  --title "AirDB v0.2.1 - Productization Release" \
  --notes-file ../../RELEASE_NOTES.md \
  airdb-0.2.1-linux.tar.gz \
  linux/airdb-bootstrap \
  linux/airdb-cli \
  linux/airdb-desktop \
  linux/install.sh \
  checksums-linux.txt

echo "✅ Release v0.2.1 recreated successfully!"
