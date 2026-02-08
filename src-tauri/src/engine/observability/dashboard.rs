//! Health Dashboard Module
//!
//! Repo health checks, version drift warnings, and migration status

use serde::{Deserialize, Serialize};
use std::path::Path;
use super::metrics::Metrics;

/// Overall health status
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum HealthStatus {
    Healthy,
    Warning,
    Critical,
    Unknown,
}

/// A single health check result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthCheck {
    pub name: String,
    pub status: HealthStatus,
    pub message: String,
    pub details: Option<serde_json::Value>,
}

/// Complete health dashboard report
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthDashboard {
    pub overall_status: HealthStatus,
    pub checks: Vec<HealthCheck>,
    pub warnings: Vec<String>,
    pub migration_status: MigrationStatusReport,
    pub resource_usage: ResourceUsage,
}

/// Migration status for dashboard
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct MigrationStatusReport {
    pub current_version: Option<u32>,
    pub pending_count: u32,
    pub last_run: Option<chrono::DateTime<chrono::Utc>>,
    pub last_success: bool,
}

/// System resource usage
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ResourceUsage {
    pub db_size_mb: f64,
    pub nosql_size_mb: f64,
    pub backup_size_mb: f64,
    pub total_size_mb: f64,
}

/// Health dashboard generator
pub struct HealthDashboardGenerator {
    thresholds: HealthThresholds,
}

/// Configurable health thresholds
#[derive(Debug, Clone)]
pub struct HealthThresholds {
    /// Update success rate below this triggers warning
    pub update_success_rate_warning: f64,
    /// Update success rate below this triggers critical
    pub update_success_rate_critical: f64,
    /// Days since last backup triggers warning
    pub backup_age_warning_days: u32,
    /// Rollback count above this triggers warning
    pub rollback_count_warning: u64,
    /// DB size above this (MB) triggers warning
    pub db_size_warning_mb: f64,
}

impl Default for HealthThresholds {
    fn default() -> Self {
        Self {
            update_success_rate_warning: 90.0,
            update_success_rate_critical: 70.0,
            backup_age_warning_days: 7,
            rollback_count_warning: 3,
            db_size_warning_mb: 500.0,
        }
    }
}

impl HealthDashboardGenerator {
    pub fn new() -> Self {
        Self {
            thresholds: HealthThresholds::default(),
        }
    }

    pub fn with_thresholds(thresholds: HealthThresholds) -> Self {
        Self { thresholds }
    }

    /// Generate a complete health dashboard
    pub fn generate(&self, project_dir: &Path) -> std::io::Result<HealthDashboard> {
        let metrics = Metrics::load(project_dir).unwrap_or_default();
        
        let mut checks = Vec::new();
        let mut warnings = Vec::new();

        // Check update success rate
        let update_check = self.check_update_health(&metrics);
        if update_check.status == HealthStatus::Warning || update_check.status == HealthStatus::Critical {
            warnings.push(update_check.message.clone());
        }
        checks.push(update_check);

        // Check rollback count
        let rollback_check = self.check_rollback_health(&metrics);
        if rollback_check.status != HealthStatus::Healthy {
            warnings.push(rollback_check.message.clone());
        }
        checks.push(rollback_check);

        // Check compatibility
        let compat_check = self.check_compatibility_health(&metrics);
        if compat_check.status != HealthStatus::Healthy {
            warnings.push(compat_check.message.clone());
        }
        checks.push(compat_check);

        // Check version drift
        let drift_check = self.check_version_drift(&metrics);
        if drift_check.status != HealthStatus::Healthy {
            warnings.push(drift_check.message.clone());
        }
        checks.push(drift_check);

        // Calculate resource usage
        let resource_usage = self.calculate_resource_usage(project_dir);

        // Check DB size
        let size_check = self.check_db_size(&resource_usage);
        if size_check.status != HealthStatus::Healthy {
            warnings.push(size_check.message.clone());
        }
        checks.push(size_check);

        // Determine overall status
        let overall_status = self.determine_overall_status(&checks);

        // Migration status
        let migration_status = MigrationStatusReport {
            current_version: metrics.schema.current_schema_version,
            pending_count: 0, // Would need to check migrations directory
            last_run: metrics.schema.last_migration,
            last_success: metrics.schema.failed_migrations == 0 || 
                          metrics.schema.successful_migrations > metrics.schema.failed_migrations,
        };

        Ok(HealthDashboard {
            overall_status,
            checks,
            warnings,
            migration_status,
            resource_usage,
        })
    }

    fn check_update_health(&self, metrics: &Metrics) -> HealthCheck {
        let rate = metrics.updates.success_rate();
        
        let (status, message) = if metrics.updates.total_updates == 0 {
            (HealthStatus::Unknown, "No updates recorded yet".to_string())
        } else if rate < self.thresholds.update_success_rate_critical {
            (HealthStatus::Critical, format!("Update success rate critically low: {:.1}%", rate))
        } else if rate < self.thresholds.update_success_rate_warning {
            (HealthStatus::Warning, format!("Update success rate below threshold: {:.1}%", rate))
        } else {
            (HealthStatus::Healthy, format!("Update success rate: {:.1}%", rate))
        };

        HealthCheck {
            name: "Update Success Rate".to_string(),
            status,
            message,
            details: Some(serde_json::json!({
                "total": metrics.updates.total_updates,
                "successful": metrics.updates.successful_updates,
                "failed": metrics.updates.failed_updates,
                "rate": rate
            })),
        }
    }

    fn check_rollback_health(&self, metrics: &Metrics) -> HealthCheck {
        let count = metrics.updates.rollback_count;
        
        let (status, message) = if count == 0 {
            (HealthStatus::Healthy, "No rollbacks recorded".to_string())
        } else if count >= self.thresholds.rollback_count_warning {
            (HealthStatus::Warning, format!("High rollback count: {}", count))
        } else {
            (HealthStatus::Healthy, format!("Rollback count: {}", count))
        };

        HealthCheck {
            name: "Rollback Count".to_string(),
            status,
            message,
            details: Some(serde_json::json!({ "count": count })),
        }
    }

    fn check_compatibility_health(&self, metrics: &Metrics) -> HealthCheck {
        let failed = metrics.compatibility.failed_checks;
        
        let (status, message) = if failed == 0 {
            (HealthStatus::Healthy, "All compatibility checks passed".to_string())
        } else {
            (HealthStatus::Warning, format!("{} compatibility check failures", failed))
        };

        HealthCheck {
            name: "Compatibility Checks".to_string(),
            status,
            message,
            details: Some(serde_json::json!({
                "total": metrics.compatibility.total_checks,
                "passed": metrics.compatibility.passed_checks,
                "failed": failed,
                "last_failure": metrics.compatibility.last_failure_reason
            })),
        }
    }

    fn check_version_drift(&self, metrics: &Metrics) -> HealthCheck {
        let drift = metrics.compatibility.version_drift_detected;
        
        let (status, message) = if drift == 0 {
            (HealthStatus::Healthy, "No version drift detected".to_string())
        } else {
            (HealthStatus::Warning, format!("Version drift detected {} times", drift))
        };

        HealthCheck {
            name: "Version Drift".to_string(),
            status,
            message,
            details: Some(serde_json::json!({ "drift_count": drift })),
        }
    }

    fn check_db_size(&self, usage: &ResourceUsage) -> HealthCheck {
        let (status, message) = if usage.total_size_mb > self.thresholds.db_size_warning_mb {
            (HealthStatus::Warning, format!("Database size exceeds threshold: {:.1} MB", usage.total_size_mb))
        } else {
            (HealthStatus::Healthy, format!("Database size: {:.1} MB", usage.total_size_mb))
        };

        HealthCheck {
            name: "Database Size".to_string(),
            status,
            message,
            details: Some(serde_json::json!({
                "db_size_mb": usage.db_size_mb,
                "nosql_size_mb": usage.nosql_size_mb,
                "total_mb": usage.total_size_mb
            })),
        }
    }

    fn calculate_resource_usage(&self, project_dir: &Path) -> ResourceUsage {
        let db_size = dir_size(&project_dir.join("data")).unwrap_or(0);
        let nosql_size = dir_size(&project_dir.join("nosql")).unwrap_or(0);
        let backup_size = dir_size(&project_dir.join("backups")).unwrap_or(0);
        
        ResourceUsage {
            db_size_mb: db_size as f64 / 1_048_576.0,
            nosql_size_mb: nosql_size as f64 / 1_048_576.0,
            backup_size_mb: backup_size as f64 / 1_048_576.0,
            total_size_mb: (db_size + nosql_size + backup_size) as f64 / 1_048_576.0,
        }
    }

    fn determine_overall_status(&self, checks: &[HealthCheck]) -> HealthStatus {
        if checks.iter().any(|c| c.status == HealthStatus::Critical) {
            HealthStatus::Critical
        } else if checks.iter().any(|c| c.status == HealthStatus::Warning) {
            HealthStatus::Warning
        } else if checks.iter().any(|c| c.status == HealthStatus::Unknown) {
            HealthStatus::Unknown
        } else {
            HealthStatus::Healthy
        }
    }
}

impl Default for HealthDashboardGenerator {
    fn default() -> Self {
        Self::new()
    }
}

/// Calculate directory size recursively
fn dir_size(path: &Path) -> std::io::Result<u64> {
    let mut total = 0;
    
    if !path.exists() {
        return Ok(0);
    }
    
    for entry in std::fs::read_dir(path)? {
        let entry = entry?;
        let metadata = entry.metadata()?;
        
        if metadata.is_dir() {
            total += dir_size(&entry.path())?;
        } else {
            total += metadata.len();
        }
    }
    
    Ok(total)
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn test_health_dashboard_generation() {
        let dir = tempdir().unwrap();
        let generator = HealthDashboardGenerator::new();
        
        let dashboard = generator.generate(dir.path()).unwrap();
        
        // With no data, should be unknown/healthy
        assert!(!dashboard.checks.is_empty());
    }

    #[test]
    fn test_health_thresholds() {
        let thresholds = HealthThresholds::default();
        assert_eq!(thresholds.update_success_rate_warning, 90.0);
        assert_eq!(thresholds.rollback_count_warning, 3);
    }
}
