//! Observability Module
//!
//! Metrics tracking and health dashboard

pub mod metrics;
pub mod dashboard;

pub use metrics::{Metrics, UpdateMetrics, SchemaMetrics, MetricsCollector, METRICS_VERSION};
pub use dashboard::{HealthDashboard, HealthDashboardGenerator, HealthStatus, HealthCheck, HealthThresholds};
