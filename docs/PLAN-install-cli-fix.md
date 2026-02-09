# Plan: Fix CLI Bundling in Installers

## Context
The current `.deb` and `.msi` installers do not include `airdb-cli` or `airdb-bootstrap`, only the main desktop application. This prevents the `airdb` command from working out-of-the-box.

## Brainstorming & Approach
**Option A: Sidecar Bundling (Selected)**
- Use Tauri's `externalBin` feature to bundle `airdb-cli` and `airdb-bootstrap`.
- Requires binaries to be named with target-triple suffix (e.g., `airdb-cli-x86_64-unknown-linux-gnu`).
- Requires a build step to rename compiled binaries.
- Requires updates to `Installer` logic to find and symlink these sidecars.

## Task Breakdown

### 1. Build Preparation
- [ ] Create `scripts/prep-sidecars.sh` to build and rename binaries with target triples.
- [ ] Ensure this script runs before `tauri build`.

### 2. Configuration
- [ ] Modify `src-tauri/tauri.conf.json` to add:
  ```json
  "bundle": {
    "externalBin": ["airdb-cli", "airdb-bootstrap"]
  }
  ```

### 3. Installer Logic Update
- [ ] Modify `src-tauri/src/engine/installer/mod.rs`:
  - Update `install_linux` / `install_windows` to locate the bundled sidecars.
  - On Linux/macOS: Symlink the sidecar (with triple) to `~/.local/bin/airdb` (without triple).
  - On Windows: Copy the sidecar (with triple) to `%LOCALAPPDATA%\AirDB\bin\airdb.exe`.

### 4. Verification
- [ ] Verify `cargo build` produces the named binaries.
- [ ] Verify `tauri build` includes them in the bundle.

## Binary Naming Strategy
Current Host: `x86_64-unknown-linux-gnu` (Linux)
Binaries to produce:
- `src-tauri/bin/airdb-cli-x86_64-unknown-linux-gnu`
- `src-tauri/bin/airdb-bootstrap-x86_64-unknown-linux-gnu`

## Agent Assignments
- **Orchestrator**: Manage the workflow.
- **Backend Specialist**: Rust code changes (`installer/mod.rs`, build scripts).
