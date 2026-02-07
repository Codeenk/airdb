//! Health Check System
//! 
//! Verifies that a new version starts correctly and triggers rollback if not.

use std::process::{Command, Stdio};
use std::time::{Duration, Instant};
use std::path::Path;

/// Health check configuration
#[derive(Debug, Clone)]
pub struct HealthConfig {
    /// Maximum time to wait for startup
    pub startup_timeout: Duration,
    /// Command to run for health check
    pub health_command: Option<String>,
}

impl Default for HealthConfig {
    fn default() -> Self {
        Self {
            startup_timeout: Duration::from_secs(30),
            health_command: None,
        }
    }
}

/// Health check result
#[derive(Debug, Clone)]
pub enum HealthResult {
    Healthy,
    Unhealthy { reason: String },
    Timeout,
}

/// Health checker for new versions
pub struct HealthChecker {
    config: HealthConfig,
}

impl HealthChecker {
    /// Create a new health checker
    pub fn new(config: HealthConfig) -> Self {
        Self { config }
    }

    /// Create with default configuration
    pub fn with_defaults() -> Self {
        Self::new(HealthConfig::default())
    }

    /// Check if a binary can start successfully
    pub fn check_binary_starts(&self, binary_path: &Path) -> HealthResult {
        if !binary_path.exists() {
            return HealthResult::Unhealthy {
                reason: "Binary not found".to_string(),
            };
        }

        // Try to run with --version to verify it starts
        let start = Instant::now();
        let result = Command::new(binary_path)
            .arg("--version")
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .spawn();

        match result {
            Ok(mut child) => {
                // Wait for completion with timeout
                loop {
                    match child.try_wait() {
                        Ok(Some(status)) => {
                            if status.success() {
                                return HealthResult::Healthy;
                            } else {
                                return HealthResult::Unhealthy {
                                    reason: format!("Exit code: {:?}", status.code()),
                                };
                            }
                        }
                        Ok(None) => {
                            if start.elapsed() > self.config.startup_timeout {
                                let _ = child.kill();
                                return HealthResult::Timeout;
                            }
                            std::thread::sleep(Duration::from_millis(100));
                        }
                        Err(e) => {
                            return HealthResult::Unhealthy {
                                reason: format!("Wait error: {}", e),
                            };
                        }
                    }
                }
            }
            Err(e) => HealthResult::Unhealthy {
                reason: format!("Failed to start: {}", e),
            },
        }
    }

    /// Run a custom health check command
    pub fn run_health_command(&self, working_dir: &Path) -> HealthResult {
        let Some(command) = &self.config.health_command else {
            // No custom health check, assume healthy
            return HealthResult::Healthy;
        };

        let start = Instant::now();
        
        #[cfg(unix)]
        let result = Command::new("sh")
            .arg("-c")
            .arg(command)
            .current_dir(working_dir)
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .spawn();

        #[cfg(windows)]
        let result = Command::new("cmd")
            .arg("/C")
            .arg(command)
            .current_dir(working_dir)
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .spawn();

        match result {
            Ok(mut child) => {
                loop {
                    match child.try_wait() {
                        Ok(Some(status)) => {
                            if status.success() {
                                return HealthResult::Healthy;
                            } else {
                                return HealthResult::Unhealthy {
                                    reason: format!("Health check failed: {:?}", status.code()),
                                };
                            }
                        }
                        Ok(None) => {
                            if start.elapsed() > self.config.startup_timeout {
                                let _ = child.kill();
                                return HealthResult::Timeout;
                            }
                            std::thread::sleep(Duration::from_millis(100));
                        }
                        Err(e) => {
                            return HealthResult::Unhealthy {
                                reason: format!("Wait error: {}", e),
                            };
                        }
                    }
                }
            }
            Err(e) => HealthResult::Unhealthy {
                reason: format!("Failed to run health check: {}", e),
            },
        }
    }

    /// Perform full health check on a version
    pub fn check_version(&self, binary_path: &Path) -> HealthResult {
        // First check if binary can start
        let binary_result = self.check_binary_starts(binary_path);
        if !matches!(binary_result, HealthResult::Healthy) {
            return binary_result;
        }

        // Then run custom health check if configured
        if let Some(parent) = binary_path.parent() {
            self.run_health_command(parent)
        } else {
            HealthResult::Healthy
        }
    }

    /// Check if we need to trigger a rollback based on boot count
    pub fn should_rollback(failed_boot_count: u32, max_fails: u32) -> bool {
        failed_boot_count >= max_fails
    }
}

impl Default for HealthChecker {
    fn default() -> Self {
        Self::with_defaults()
    }
}

/// Marker file for boot success tracking
pub struct BootMarker;

impl BootMarker {
    const MARKER_NAME: &'static str = ".boot_success";

    /// Create a boot success marker
    pub fn mark_success(version_dir: &Path) -> std::io::Result<()> {
        let marker_path = version_dir.join(Self::MARKER_NAME);
        std::fs::write(&marker_path, "1")?;
        Ok(())
    }

    /// Check if previous boot was successful
    pub fn was_last_boot_successful(version_dir: &Path) -> bool {
        version_dir.join(Self::MARKER_NAME).exists()
    }

    /// Clear the boot marker (called at startup before health check)
    pub fn clear_marker(version_dir: &Path) -> std::io::Result<()> {
        let marker_path = version_dir.join(Self::MARKER_NAME);
        if marker_path.exists() {
            std::fs::remove_file(&marker_path)?;
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn test_health_checker_creation() {
        let checker = HealthChecker::with_defaults();
        assert_eq!(checker.config.startup_timeout, Duration::from_secs(30));
    }

    #[test]
    fn test_should_rollback() {
        assert!(!HealthChecker::should_rollback(0, 3));
        assert!(!HealthChecker::should_rollback(1, 3));
        assert!(!HealthChecker::should_rollback(2, 3));
        assert!(HealthChecker::should_rollback(3, 3));
        assert!(HealthChecker::should_rollback(4, 3));
    }

    #[test]
    fn test_boot_marker() {
        let dir = tempdir().unwrap();
        
        assert!(!BootMarker::was_last_boot_successful(dir.path()));
        
        BootMarker::mark_success(dir.path()).unwrap();
        assert!(BootMarker::was_last_boot_successful(dir.path()));
        
        BootMarker::clear_marker(dir.path()).unwrap();
        assert!(!BootMarker::was_last_boot_successful(dir.path()));
    }

    #[test]
    fn test_check_nonexistent_binary() {
        let checker = HealthChecker::with_defaults();
        let result = checker.check_binary_starts(Path::new("/nonexistent/binary"));
        assert!(matches!(result, HealthResult::Unhealthy { .. }));
    }
}
