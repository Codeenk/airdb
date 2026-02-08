//! Observability Metrics Module
//!
//! Tracks update success rate, rollback counts, and schema upgrade duration

use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};
use chrono::{DateTime, Utc};

/// Current metrics format version
pub const METRICS_VERSION: u32 = 1;

/// Metrics data store
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Metrics {
    pub version: u32,
    pub updates: UpdateMetrics,
    pub schema: SchemaMetrics,
    pub compatibility: CompatibilityMetrics,
    pub system: SystemMetrics,
}

/// Update-related metrics
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct UpdateMetrics {
    pub total_updates: u64,
    pub successful_updates: u64,
    pub failed_updates: u64,
    pub rollback_count: u64,
    pub last_update: Option<DateTime<Utc>>,
    pub last_update_version: Option<String>,
    pub average_update_duration_ms: Option<u64>,
    pub update_history: Vec<UpdateRecord>,
}

/// Single update record
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateRecord {
    pub timestamp: DateTime<Utc>,
    pub from_version: String,
    pub to_version: String,
    pub success: bool,
    pub duration_ms: u64,
    pub rollback_triggered: bool,
    pub error: Option<String>,
}

/// Schema-related metrics
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct SchemaMetrics {
    pub total_migrations: u64,
    pub successful_migrations: u64,
    pub failed_migrations: u64,
    pub average_migration_duration_ms: Option<u64>,
    pub last_migration: Option<DateTime<Utc>>,
    pub current_schema_version: Option<u32>,
}

/// Compatibility check metrics
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct CompatibilityMetrics {
    pub total_checks: u64,
    pub passed_checks: u64,
    pub failed_checks: u64,
    pub last_failure_reason: Option<String>,
    pub version_drift_detected: u64,
}

/// System resource metrics
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct SystemMetrics {
    pub db_size_bytes: Option<u64>,
    pub nosql_size_bytes: Option<u64>,
    pub backup_size_bytes: Option<u64>,
    pub collection_count: Option<u32>,
    pub table_count: Option<u32>,
}

impl Metrics {
    /// Create new metrics
    pub fn new() -> Self {
        Self {
            version: METRICS_VERSION,
            ..Default::default()
        }
    }

    /// Load metrics from file
    pub fn load(project_dir: &Path) -> std::io::Result<Self> {
        let metrics_path = project_dir.join(".airdb").join("metrics.json");
        
        if !metrics_path.exists() {
            return Ok(Self::new());
        }
        
        let content = fs::read_to_string(&metrics_path)?;
        let metrics: Metrics = serde_json::from_str(&content)
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))?;
        
        // Version compatibility
        if metrics.version > METRICS_VERSION {
            return Err(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                "Metrics version too new"
            ));
        }
        
        Ok(metrics)
    }

    /// Save metrics to file
    pub fn save(&self, project_dir: &Path) -> std::io::Result<()> {
        let metrics_dir = project_dir.join(".airdb");
        fs::create_dir_all(&metrics_dir)?;
        
        let metrics_path = metrics_dir.join("metrics.json");
        let content = serde_json::to_string_pretty(self)
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))?;
        
        fs::write(&metrics_path, content)
    }
}

impl UpdateMetrics {
    /// Record an update attempt
    pub fn record_update(
        &mut self,
        from_version: &str,
        to_version: &str,
        success: bool,
        duration_ms: u64,
        rollback_triggered: bool,
        error: Option<String>,
    ) {
        self.total_updates += 1;
        
        if success {
            self.successful_updates += 1;
        } else {
            self.failed_updates += 1;
        }
        
        if rollback_triggered {
            self.rollback_count += 1;
        }
        
        self.last_update = Some(Utc::now());
        self.last_update_version = Some(to_version.to_string());
        
        // Update rolling average
        let total = self.total_updates;
        if let Some(avg) = self.average_update_duration_ms {
            self.average_update_duration_ms = Some(
                ((avg * (total - 1)) + duration_ms) / total
            );
        } else {
            self.average_update_duration_ms = Some(duration_ms);
        }
        
        // Add to history (keep last 50)
        self.update_history.push(UpdateRecord {
            timestamp: Utc::now(),
            from_version: from_version.to_string(),
            to_version: to_version.to_string(),
            success,
            duration_ms,
            rollback_triggered,
            error,
        });
        
        if self.update_history.len() > 50 {
            self.update_history.remove(0);
        }
    }

    /// Get update success rate as percentage
    pub fn success_rate(&self) -> f64 {
        if self.total_updates == 0 {
            return 100.0;
        }
        (self.successful_updates as f64 / self.total_updates as f64) * 100.0
    }
}

impl SchemaMetrics {
    /// Record a migration
    pub fn record_migration(&mut self, success: bool, duration_ms: u64, new_version: Option<u32>) {
        self.total_migrations += 1;
        
        if success {
            self.successful_migrations += 1;
        } else {
            self.failed_migrations += 1;
        }
        
        self.last_migration = Some(Utc::now());
        
        if let Some(v) = new_version {
            self.current_schema_version = Some(v);
        }
        
        // Update rolling average
        let total = self.total_migrations;
        if let Some(avg) = self.average_migration_duration_ms {
            self.average_migration_duration_ms = Some(
                ((avg * (total - 1)) + duration_ms) / total
            );
        } else {
            self.average_migration_duration_ms = Some(duration_ms);
        }
    }
}

impl CompatibilityMetrics {
    /// Record a compatibility check
    pub fn record_check(&mut self, passed: bool, failure_reason: Option<String>) {
        self.total_checks += 1;
        
        if passed {
            self.passed_checks += 1;
        } else {
            self.failed_checks += 1;
            self.last_failure_reason = failure_reason;
        }
    }

    /// Record version drift detection
    pub fn record_version_drift(&mut self) {
        self.version_drift_detected += 1;
    }
}

/// Metrics collector for easy recording
pub struct MetricsCollector {
    project_dir: PathBuf,
}

impl MetricsCollector {
    pub fn new(project_dir: &Path) -> Self {
        Self {
            project_dir: project_dir.to_path_buf(),
        }
    }

    /// Record an update with automatic save
    pub fn record_update(
        &self,
        from_version: &str,
        to_version: &str,
        success: bool,
        duration_ms: u64,
        rollback_triggered: bool,
        error: Option<String>,
    ) -> std::io::Result<()> {
        let mut metrics = Metrics::load(&self.project_dir)?;
        metrics.updates.record_update(
            from_version,
            to_version,
            success,
            duration_ms,
            rollback_triggered,
            error,
        );
        metrics.save(&self.project_dir)
    }

    /// Record a migration with automatic save
    pub fn record_migration(
        &self,
        success: bool,
        duration_ms: u64,
        new_version: Option<u32>,
    ) -> std::io::Result<()> {
        let mut metrics = Metrics::load(&self.project_dir)?;
        metrics.schema.record_migration(success, duration_ms, new_version);
        metrics.save(&self.project_dir)
    }

    /// Record a compatibility check with automatic save
    pub fn record_compatibility_check(
        &self,
        passed: bool,
        failure_reason: Option<String>,
    ) -> std::io::Result<()> {
        let mut metrics = Metrics::load(&self.project_dir)?;
        metrics.compatibility.record_check(passed, failure_reason);
        metrics.save(&self.project_dir)
    }

    /// Get current metrics
    pub fn get_metrics(&self) -> std::io::Result<Metrics> {
        Metrics::load(&self.project_dir)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn test_metrics_creation_save_load() {
        let dir = tempdir().unwrap();
        let metrics = Metrics::new();
        
        metrics.save(dir.path()).unwrap();
        
        let loaded = Metrics::load(dir.path()).unwrap();
        assert_eq!(loaded.version, METRICS_VERSION);
    }

    #[test]
    fn test_update_metrics_recording() {
        let mut metrics = UpdateMetrics::default();
        
        metrics.record_update("0.1.0", "0.2.0", true, 1000, false, None);
        assert_eq!(metrics.total_updates, 1);
        assert_eq!(metrics.successful_updates, 1);
        assert_eq!(metrics.success_rate(), 100.0);
        
        metrics.record_update("0.2.0", "0.3.0", false, 500, true, Some("Error".to_string()));
        assert_eq!(metrics.total_updates, 2);
        assert_eq!(metrics.failed_updates, 1);
        assert_eq!(metrics.rollback_count, 1);
        assert_eq!(metrics.success_rate(), 50.0);
    }

    #[test]
    fn test_metrics_collector() {
        let dir = tempdir().unwrap();
        let collector = MetricsCollector::new(dir.path());
        
        collector.record_update("0.1.0", "0.2.0", true, 1000, false, None).unwrap();
        
        let metrics = collector.get_metrics().unwrap();
        assert_eq!(metrics.updates.total_updates, 1);
    }
}
