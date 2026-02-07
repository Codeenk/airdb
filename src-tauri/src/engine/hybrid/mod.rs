//! Hybrid SQL/NoSQL Module
//!
//! Unified interface for cross-engine operations

pub mod relations;
pub mod airql;
pub mod executor;

pub use relations::{Relation, RelationType, FieldRef, EngineType, RelationsManifest, CascadeBehavior};
pub use airql::{AirQuery, AirFilter, AirResult};
pub use executor::HybridExecutor;
