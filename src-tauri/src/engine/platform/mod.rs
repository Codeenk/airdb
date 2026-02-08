//! Platform Detection and Support Module
//!
//! macOS, Linux, and Windows platform integration

use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// Supported platforms
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Platform {
    MacOS,
    Linux,
    Windows,
    Unknown,
}

impl Platform {
    /// Detect current platform
    pub fn current() -> Self {
        #[cfg(target_os = "macos")]
        return Platform::MacOS;
        
        #[cfg(target_os = "linux")]
        return Platform::Linux;
        
        #[cfg(target_os = "windows")]
        return Platform::Windows;
        
        #[cfg(not(any(target_os = "macos", target_os = "linux", target_os = "windows")))]
        return Platform::Unknown;
    }

    /// Check if this is macOS
    pub fn is_macos(&self) -> bool {
        matches!(self, Platform::MacOS)
    }

    /// Check if this is Linux
    pub fn is_linux(&self) -> bool {
        matches!(self, Platform::Linux)
    }

    /// Check if this is Windows
    pub fn is_windows(&self) -> bool {
        matches!(self, Platform::Windows)
    }

    /// Get platform-specific app data directory
    pub fn app_data_dir(&self) -> Option<PathBuf> {
        match self {
            Platform::MacOS => dirs::data_dir().map(|d| d.join("AirDB")),
            Platform::Linux => dirs::data_dir().map(|d| d.join("airdb")),
            Platform::Windows => dirs::data_dir().map(|d| d.join("AirDB")),
            Platform::Unknown => None,
        }
    }

    /// Get platform-specific config directory
    pub fn config_dir(&self) -> Option<PathBuf> {
        match self {
            Platform::MacOS => dirs::config_dir().map(|d| d.join("AirDB")),
            Platform::Linux => dirs::config_dir().map(|d| d.join("airdb")),
            Platform::Windows => dirs::config_dir().map(|d| d.join("AirDB")),
            Platform::Unknown => None,
        }
    }

    /// Get platform-specific log directory
    pub fn log_dir(&self) -> Option<PathBuf> {
        match self {
            Platform::MacOS => dirs::home_dir().map(|d| d.join("Library/Logs/AirDB")),
            Platform::Linux => dirs::data_dir().map(|d| d.join("airdb/logs")),
            Platform::Windows => dirs::data_dir().map(|d| d.join("AirDB/logs")),
            Platform::Unknown => None,
        }
    }

    /// Get the update manifest filename for this platform
    pub fn manifest_filename(&self) -> &'static str {
        // Same manifest format across all platforms
        "update-manifest.json"
    }

    /// Get the binary extension for this platform
    pub fn binary_extension(&self) -> &'static str {
        match self {
            Platform::Windows => ".exe",
            _ => "",
        }
    }

    /// Get the bundle extension for this platform
    pub fn bundle_extension(&self) -> &'static str {
        match self {
            Platform::MacOS => ".app",
            Platform::Linux => ".AppImage",
            Platform::Windows => ".exe",
            Platform::Unknown => "",
        }
    }
}

/// Platform-specific updater configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlatformConfig {
    pub platform: Platform,
    pub app_data_dir: Option<PathBuf>,
    pub config_dir: Option<PathBuf>,
    pub log_dir: Option<PathBuf>,
    pub bundle_type: String,
    pub supports_hot_reload: bool,
}

impl PlatformConfig {
    /// Get current platform configuration
    pub fn current() -> Self {
        let platform = Platform::current();
        
        Self {
            platform,
            app_data_dir: platform.app_data_dir(),
            config_dir: platform.config_dir(),
            log_dir: platform.log_dir(),
            bundle_type: match platform {
                Platform::MacOS => "dmg".to_string(),
                Platform::Linux => "appimage".to_string(),
                Platform::Windows => "nsis".to_string(),
                Platform::Unknown => "unknown".to_string(),
            },
            supports_hot_reload: !platform.is_macos(), // macOS bundles are harder to hot reload
        }
    }

    /// Check if current platform is supported
    pub fn is_supported(&self) -> bool {
        !matches!(self.platform, Platform::Unknown)
    }
}

/// macOS-specific utilities
pub mod macos {
    use std::path::Path;
    use std::process::Command;

    /// Check if running from a signed app bundle
    pub fn is_signed_bundle() -> bool {
        #[cfg(target_os = "macos")]
        {
            let output = Command::new("codesign")
                .args(["-v", "-v", "--strict"])
                .arg(std::env::current_exe().unwrap_or_default())
                .output();
            
            output.map(|o| o.status.success()).unwrap_or(false)
        }
        
        #[cfg(not(target_os = "macos"))]
        false
    }

    /// Verify code signature of a bundle
    pub fn verify_signature(bundle_path: &Path) -> bool {
        #[cfg(target_os = "macos")]
        {
            let output = Command::new("codesign")
                .args(["-v", "-v", "--strict"])
                .arg(bundle_path)
                .output();
            
            output.map(|o| o.status.success()).unwrap_or(false)
        }
        
        #[cfg(not(target_os = "macos"))]
        {
            let _ = bundle_path;
            false
        }
    }

    /// Check Gatekeeper status (macOS security)
    pub fn gatekeeper_enabled() -> bool {
        #[cfg(target_os = "macos")]
        {
            let output = Command::new("spctl")
                .args(["--status"])
                .output();
            
            output.map(|o| {
                String::from_utf8_lossy(&o.stdout).contains("enabled")
            }).unwrap_or(true)
        }
        
        #[cfg(not(target_os = "macos"))]
        false
    }
}

/// Linux-specific utilities
pub mod linux {
    use std::path::Path;

    /// Check if running from AppImage
    pub fn is_appimage() -> bool {
        std::env::var("APPIMAGE").is_ok()
    }

    /// Get AppImage path if running from one
    pub fn appimage_path() -> Option<String> {
        std::env::var("APPIMAGE").ok()
    }

    /// Check if running from Flatpak
    pub fn is_flatpak() -> bool {
        std::env::var("FLATPAK_ID").is_ok()
    }

    /// Check if running from Snap
    pub fn is_snap() -> bool {
        std::env::var("SNAP").is_ok()
    }

    /// Get the actual executable path (handles AppImage)
    pub fn executable_path() -> Option<std::path::PathBuf> {
        if is_appimage() {
            appimage_path().map(std::path::PathBuf::from)
        } else {
            std::env::current_exe().ok()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_platform_detection() {
        let platform = Platform::current();
        
        #[cfg(target_os = "macos")]
        assert!(platform.is_macos());
        
        #[cfg(target_os = "linux")]
        assert!(platform.is_linux());
        
        #[cfg(target_os = "windows")]
        assert!(platform.is_windows());
    }

    #[test]
    fn test_platform_config() {
        let config = PlatformConfig::current();
        assert!(config.is_supported());
        assert!(config.app_data_dir.is_some());
    }

    #[test]
    fn test_binary_extension() {
        let platform = Platform::current();
        
        #[cfg(target_os = "windows")]
        assert_eq!(platform.binary_extension(), ".exe");
        
        #[cfg(not(target_os = "windows"))]
        assert_eq!(platform.binary_extension(), "");
    }
}
