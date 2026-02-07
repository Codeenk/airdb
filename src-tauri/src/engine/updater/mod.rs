//! AirDB Self-Update System
//! 
//! A rollback-safe, cryptographically verified update system.
//! 
//! Components:
//! - `state` - Update state machine
//! - `version_manager` - Filesystem operations
//! - `download` - HTTP download with resume
//! - `verify` - Checksum and signature verification
//! - `health` - Startup health checks

pub mod state;
pub mod version_manager;
pub mod download;
pub mod verify;
pub mod health;

pub use state::{UpdateState, UpdateStatus};
pub use version_manager::VersionManager;
