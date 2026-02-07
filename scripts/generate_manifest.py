#!/usr/bin/env python3
"""
Generate update manifest for AirDB releases.

This script creates the update manifest JSON that contains:
- Version info
- Platform-specific artifact URLs and checksums
- Optional ED25519 signature

Usage:
    python scripts/generate_manifest.py --version 0.2.0 --channel stable
"""

import argparse
import hashlib
import json
import os
from datetime import datetime
from pathlib import Path

def calculate_sha256(filepath: Path) -> str:
    """Calculate SHA256 checksum of a file."""
    sha256_hash = hashlib.sha256()
    with open(filepath, "rb") as f:
        for chunk in iter(lambda: f.read(4096), b""):
            sha256_hash.update(chunk)
    return sha256_hash.hexdigest()

def find_artifacts(bundle_dir: Path, version: str) -> dict:
    """Find platform artifacts in the bundle directory."""
    artifacts = {}
    
    # Linux artifacts
    linux_patterns = [
        f"**/airdb_{version}_amd64.AppImage",
        f"**/airdb_{version}_amd64.deb",
    ]
    for pattern in linux_patterns:
        for file in bundle_dir.glob(pattern):
            if file.suffix == ".AppImage":
                artifacts["linux"] = {
                    "url": f"https://github.com/Codeenk/airdb/releases/download/v{version}/{file.name}",
                    "sha256": calculate_sha256(file),
                    "size": file.stat().st_size,
                    "filename": file.name
                }
                break
    
    # Windows artifacts
    windows_patterns = [
        f"**/airdb_{version}_x64-setup.exe",
        f"**/airdb_{version}_x64_en-US.msi",
    ]
    for pattern in windows_patterns:
        for file in bundle_dir.glob(pattern):
            if file.suffix in [".exe", ".msi"]:
                artifacts["windows"] = {
                    "url": f"https://github.com/Codeenk/airdb/releases/download/v{version}/{file.name}",
                    "sha256": calculate_sha256(file),
                    "size": file.stat().st_size,
                    "filename": file.name
                }
                break
    
    # macOS artifacts
    macos_patterns = [
        f"**/airdb_{version}_x64.dmg",
        f"**/airdb_{version}_aarch64.dmg",
    ]
    for pattern in macos_patterns:
        for file in bundle_dir.glob(pattern):
            if file.suffix == ".dmg":
                artifacts["macos"] = {
                    "url": f"https://github.com/Codeenk/airdb/releases/download/v{version}/{file.name}",
                    "sha256": calculate_sha256(file),
                    "size": file.stat().st_size,
                    "filename": file.name
                }
                break
    
    return artifacts

def generate_manifest(
    version: str,
    channel: str = "stable",
    min_version: str = "0.1.0",
    changelog: list = None,
    bundle_dir: Path = None
) -> dict:
    """Generate the update manifest."""
    
    manifest = {
        "version": version,
        "channel": channel,
        "release_date": datetime.utcnow().strftime("%Y-%m-%d"),
        "min_supported_version": min_version,
        "changelog": changelog or [],
        "artifacts": {},
        "signature": ""  # Placeholder - sign in CI
    }
    
    if bundle_dir and bundle_dir.exists():
        manifest["artifacts"] = find_artifacts(bundle_dir, version)
    
    return manifest

def main():
    parser = argparse.ArgumentParser(description="Generate AirDB update manifest")
    parser.add_argument("--version", required=True, help="Release version (e.g., 0.2.0)")
    parser.add_argument("--channel", default="stable", help="Release channel")
    parser.add_argument("--min-version", default="0.1.0", help="Minimum supported version")
    parser.add_argument("--changelog", nargs="+", help="Changelog entries")
    parser.add_argument("--bundle-dir", type=Path, help="Path to bundle directory")
    parser.add_argument("--output", type=Path, default=Path("update-manifest.json"))
    
    args = parser.parse_args()
    
    manifest = generate_manifest(
        version=args.version,
        channel=args.channel,
        min_version=args.min_version,
        changelog=args.changelog,
        bundle_dir=args.bundle_dir
    )
    
    with open(args.output, "w") as f:
        json.dump(manifest, f, indent=2)
    
    print(f"✅ Generated manifest: {args.output}")
    print(json.dumps(manifest, indent=2))

if __name__ == "__main__":
    main()
