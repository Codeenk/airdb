//! Team Workflow Module
//!
//! Branch-aware operations and conflict-safe merges

pub mod branch;
pub mod merge;

pub use branch::{BranchContext, BranchLock};
pub use merge::{MergeStrategy, ThreeWayMerge, ConflictResolution};
