//! Auto-Start System
//!
//! Cross-platform system boot integration for AirDB

use serde::{Deserialize, Serialize};
use std::path::Path;

/// Autostart status
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AutostartStatus {
    pub enabled: bool,
    pub last_start_success: Option<bool>,
    pub last_start_time: Option<chrono::DateTime<chrono::Utc>>,
    pub error_message: Option<String>,
}

impl Default for AutostartStatus {
    fn default() -> Self {
        Self {
            enabled: false,
            last_start_success: None,
            last_start_time: None,
            error_message: None,
        }
    }
}

/// Autostart manager - handles platform-specific autostart registration
pub struct AutostartManager {
    app_name: String,
    bootstrapper_path: String,
}

impl AutostartManager {
    pub fn new(app_name: &str, bootstrapper_path: &str) -> Self {
        Self {
            app_name: app_name.to_string(),
            bootstrapper_path: bootstrapper_path.to_string(),
        }
    }

    /// Enable autostart
    pub fn enable(&self) -> Result<(), AutostartError> {
        #[cfg(target_os = "windows")]
        return self.enable_windows();
        
        #[cfg(target_os = "linux")]
        return self.enable_linux();
        
        #[cfg(target_os = "macos")]
        return self.enable_macos();
        
        #[cfg(not(any(target_os = "windows", target_os = "linux", target_os = "macos")))]
        Err(AutostartError::UnsupportedPlatform)
    }

    /// Disable autostart
    pub fn disable(&self) -> Result<(), AutostartError> {
        #[cfg(target_os = "windows")]
        return self.disable_windows();
        
        #[cfg(target_os = "linux")]
        return self.disable_linux();
        
        #[cfg(target_os = "macos")]
        return self.disable_macos();
        
        #[cfg(not(any(target_os = "windows", target_os = "linux", target_os = "macos")))]
        Err(AutostartError::UnsupportedPlatform)
    }

    /// Check if autostart is enabled
    pub fn is_enabled(&self) -> bool {
        #[cfg(target_os = "windows")]
        return self.is_enabled_windows();
        
        #[cfg(target_os = "linux")]
        return self.is_enabled_linux();
        
        #[cfg(target_os = "macos")]
        return self.is_enabled_macos();
        
        #[cfg(not(any(target_os = "windows", target_os = "linux", target_os = "macos")))]
        false
    }

    // ============ Windows Implementation ============
    
    #[cfg(target_os = "windows")]
    fn enable_windows(&self) -> Result<(), AutostartError> {
        use std::process::Command;
        
        // Use schtasks to create a scheduled task that runs at logon
        let task_name = format!("{} AutoStart", self.app_name);
        
        let output = Command::new("schtasks")
            .args([
                "/Create",
                "/TN", &task_name,
                "/TR", &self.bootstrapper_path,
                "/SC", "ONLOGON",
                "/RL", "LIMITED",
                "/F",  // Force overwrite if exists
            ])
            .output()
            .map_err(|e| AutostartError::CommandFailed(e.to_string()))?;
        
        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(AutostartError::CommandFailed(stderr.to_string()));
        }
        
        Ok(())
    }

    #[cfg(target_os = "windows")]
    fn disable_windows(&self) -> Result<(), AutostartError> {
        use std::process::Command;
        
        let task_name = format!("{} AutoStart", self.app_name);
        
        let output = Command::new("schtasks")
            .args(["/Delete", "/TN", &task_name, "/F"])
            .output()
            .map_err(|e| AutostartError::CommandFailed(e.to_string()))?;
        
        if !output.status.success() {
            // Task might not exist, which is fine
            let stderr = String::from_utf8_lossy(&output.stderr);
            if !stderr.contains("does not exist") {
                return Err(AutostartError::CommandFailed(stderr.to_string()));
            }
        }
        
        Ok(())
    }

    #[cfg(target_os = "windows")]
    fn is_enabled_windows(&self) -> bool {
        use std::process::Command;
        
        let task_name = format!("{} AutoStart", self.app_name);
        
        Command::new("schtasks")
            .args(["/Query", "/TN", &task_name])
            .output()
            .map(|o| o.status.success())
            .unwrap_or(false)
    }

    // ============ Linux Implementation ============
    
    #[cfg(target_os = "linux")]
    fn enable_linux(&self) -> Result<(), AutostartError> {
        use std::fs;
        
        let autostart_dir = dirs::config_dir()
            .ok_or(AutostartError::NoConfigDir)?
            .join("autostart");
        
        fs::create_dir_all(&autostart_dir)
            .map_err(|e| AutostartError::IoError(e.to_string()))?;
        
        let desktop_file = autostart_dir.join(format!("{}.desktop", self.app_name.to_lowercase()));
        
        let content = format!(
            r#"[Desktop Entry]
Type=Application
Name={}
Exec={}
Hidden=false
NoDisplay=false
X-GNOME-Autostart-enabled=true
Comment=Start {} on login
"#,
            self.app_name, self.bootstrapper_path, self.app_name
        );
        
        fs::write(&desktop_file, content)
            .map_err(|e| AutostartError::IoError(e.to_string()))?;
        
        Ok(())
    }

    #[cfg(target_os = "linux")]
    fn disable_linux(&self) -> Result<(), AutostartError> {
        use std::fs;
        
        let autostart_dir = dirs::config_dir()
            .ok_or(AutostartError::NoConfigDir)?
            .join("autostart");
        
        let desktop_file = autostart_dir.join(format!("{}.desktop", self.app_name.to_lowercase()));
        
        if desktop_file.exists() {
            fs::remove_file(&desktop_file)
                .map_err(|e| AutostartError::IoError(e.to_string()))?;
        }
        
        Ok(())
    }

    #[cfg(target_os = "linux")]
    fn is_enabled_linux(&self) -> bool {
        let autostart_dir = dirs::config_dir()
            .map(|d| d.join("autostart"))
            .unwrap_or_default();
        
        autostart_dir.join(format!("{}.desktop", self.app_name.to_lowercase())).exists()
    }

    // ============ macOS Implementation ============
    
    #[cfg(target_os = "macos")]
    fn enable_macos(&self) -> Result<(), AutostartError> {
        use std::fs;
        
        let launch_agents = dirs::home_dir()
            .ok_or(AutostartError::NoConfigDir)?
            .join("Library/LaunchAgents");
        
        fs::create_dir_all(&launch_agents)
            .map_err(|e| AutostartError::IoError(e.to_string()))?;
        
        let plist_path = launch_agents.join(format!("com.{}.bootstrap.plist", 
            self.app_name.to_lowercase()));
        
        let content = format!(
            r#"<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
    <key>Label</key>
    <string>com.{}.bootstrap</string>
    <key>ProgramArguments</key>
    <array>
        <string>{}</string>
    </array>
    <key>RunAtLoad</key>
    <true/>
    <key>KeepAlive</key>
    <false/>
</dict>
</plist>
"#,
            self.app_name.to_lowercase(),
            self.bootstrapper_path
        );
        
        fs::write(&plist_path, content)
            .map_err(|e| AutostartError::IoError(e.to_string()))?;
        
        Ok(())
    }

    #[cfg(target_os = "macos")]
    fn disable_macos(&self) -> Result<(), AutostartError> {
        use std::fs;
        
        let plist_path = dirs::home_dir()
            .ok_or(AutostartError::NoConfigDir)?
            .join("Library/LaunchAgents")
            .join(format!("com.{}.bootstrap.plist", self.app_name.to_lowercase()));
        
        if plist_path.exists() {
            fs::remove_file(&plist_path)
                .map_err(|e| AutostartError::IoError(e.to_string()))?;
        }
        
        Ok(())
    }

    #[cfg(target_os = "macos")]
    fn is_enabled_macos(&self) -> bool {
        dirs::home_dir()
            .map(|h| h.join("Library/LaunchAgents")
                .join(format!("com.{}.bootstrap.plist", self.app_name.to_lowercase()))
                .exists())
            .unwrap_or(false)
    }
}

/// Autostart errors
#[derive(Debug, Clone)]
pub enum AutostartError {
    UnsupportedPlatform,
    NoConfigDir,
    IoError(String),
    CommandFailed(String),
}

impl std::fmt::Display for AutostartError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AutostartError::UnsupportedPlatform => write!(f, "Unsupported platform"),
            AutostartError::NoConfigDir => write!(f, "Could not find config directory"),
            AutostartError::IoError(e) => write!(f, "IO error: {}", e),
            AutostartError::CommandFailed(e) => write!(f, "Command failed: {}", e),
        }
    }
}

impl std::error::Error for AutostartError {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_autostart_status_default() {
        let status = AutostartStatus::default();
        assert!(!status.enabled);
    }

    #[test]
    fn test_autostart_manager_creation() {
        let manager = AutostartManager::new("AirDB", "/usr/local/bin/airdb-bootstrap");
        assert_eq!(manager.app_name, "AirDB");
    }
}
