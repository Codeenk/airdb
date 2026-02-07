//! Version Manager
//! 
//! Manages the versioned filesystem layout and atomic version switching.

use std::path::{Path, PathBuf};
use std::fs;
use std::io;

#[cfg(unix)]
use std::os::unix::fs::symlink;

/// Version Manager handles the filesystem layout for side-by-side updates
pub struct VersionManager {
    /// Base directory for AirDB data
    base_dir: PathBuf,
}

impl VersionManager {
    /// Create a new version manager
    /// 
    /// Linux: ~/.local/share/airdb/
    /// Windows: %LOCALAPPDATA%\AirDB\
    pub fn new() -> Result<Self, VersionError> {
        let base_dir = Self::get_base_dir()?;
        Ok(Self { base_dir })
    }

    /// Create with custom base directory (for testing)
    pub fn with_base_dir(base_dir: PathBuf) -> Self {
        Self { base_dir }
    }

    /// Get the platform-specific base directory
    fn get_base_dir() -> Result<PathBuf, VersionError> {
        #[cfg(target_os = "linux")]
        {
            let home = dirs::home_dir()
                .ok_or_else(|| VersionError::NoHomeDir)?;
            Ok(home.join(".local/share/airdb"))
        }
        
        #[cfg(target_os = "windows")]
        {
            let local_app_data = dirs::data_local_dir()
                .ok_or_else(|| VersionError::NoHomeDir)?;
            Ok(local_app_data.join("AirDB"))
        }
        
        #[cfg(target_os = "macos")]
        {
            let app_support = dirs::data_dir()
                .ok_or_else(|| VersionError::NoHomeDir)?;
            Ok(app_support.join("AirDB"))
        }
    }

    /// Initialize the directory structure
    pub fn init(&self) -> Result<(), VersionError> {
        // Create all required directories
        fs::create_dir_all(self.versions_dir())?;
        fs::create_dir_all(self.updater_dir())?;
        fs::create_dir_all(self.logs_dir())?;
        Ok(())
    }

    /// Get the versions directory
    pub fn versions_dir(&self) -> PathBuf {
        self.base_dir.join("versions")
    }

    /// Get the current symlink/directory
    pub fn current_dir(&self) -> PathBuf {
        self.base_dir.join("current")
    }

    /// Get the updater directory
    pub fn updater_dir(&self) -> PathBuf {
        self.base_dir.join("updater")
    }

    /// Get the logs directory
    pub fn logs_dir(&self) -> PathBuf {
        self.base_dir.join("logs")
    }

    /// Get the state.json path
    pub fn state_path(&self) -> PathBuf {
        self.base_dir.join("state.json")
    }

    /// Get the path for a specific version
    pub fn version_path(&self, version: &str) -> PathBuf {
        self.versions_dir().join(version)
    }

    /// Get the temp download directory for a version
    pub fn temp_version_path(&self, version: &str) -> PathBuf {
        self.versions_dir().join(format!(".tmp-{}", version))
    }

    /// Check if a version is installed
    pub fn is_version_installed(&self, version: &str) -> bool {
        self.version_path(version).exists()
    }

    /// List all installed versions
    pub fn list_versions(&self) -> Result<Vec<String>, VersionError> {
        let versions_dir = self.versions_dir();
        if !versions_dir.exists() {
            return Ok(vec![]);
        }

        let mut versions = Vec::new();
        for entry in fs::read_dir(&versions_dir)? {
            let entry = entry?;
            let path = entry.path();
            if path.is_dir() {
                if let Some(name) = path.file_name().and_then(|n| n.to_str()) {
                    // Skip temp directories
                    if !name.starts_with(".tmp-") {
                        versions.push(name.to_string());
                    }
                }
            }
        }
        
        // Sort by semver
        versions.sort_by(|a, b| {
            let a_parts: Vec<u32> = a.split('.').filter_map(|s| s.parse().ok()).collect();
            let b_parts: Vec<u32> = b.split('.').filter_map(|s| s.parse().ok()).collect();
            a_parts.cmp(&b_parts)
        });
        
        Ok(versions)
    }

    /// Get the current version (from symlink target)
    pub fn get_current_version(&self) -> Result<Option<String>, VersionError> {
        let current = self.current_dir();
        if !current.exists() {
            return Ok(None);
        }

        #[cfg(unix)]
        {
            let target = fs::read_link(&current)?;
            if let Some(name) = target.file_name().and_then(|n| n.to_str()) {
                return Ok(Some(name.to_string()));
            }
        }

        #[cfg(windows)]
        {
            // On Windows, we use a marker file instead of symlinks
            let marker = current.join(".version");
            if marker.exists() {
                let version = fs::read_to_string(&marker)?;
                return Ok(Some(version.trim().to_string()));
            }
        }

        Ok(None)
    }

    /// Switch to a new version atomically
    #[cfg(unix)]
    pub fn switch_version(&self, version: &str) -> Result<(), VersionError> {
        let version_path = self.version_path(version);
        if !version_path.exists() {
            return Err(VersionError::VersionNotFound(version.to_string()));
        }

        let current = self.current_dir();
        let temp_link = self.base_dir.join(".current_new");

        // Create new symlink
        if temp_link.exists() {
            fs::remove_file(&temp_link)?;
        }
        symlink(&version_path, &temp_link)?;

        // Atomic rename
        fs::rename(&temp_link, &current)?;

        Ok(())
    }

    /// Switch to a new version atomically (Windows)
    #[cfg(windows)]
    pub fn switch_version(&self, version: &str) -> Result<(), VersionError> {
        let version_path = self.version_path(version);
        if !version_path.exists() {
            return Err(VersionError::VersionNotFound(version.to_string()));
        }

        let current = self.current_dir();
        
        // Create current directory if not exists
        fs::create_dir_all(&current)?;
        
        // Write version marker
        let marker = current.join(".version");
        fs::write(&marker, version)?;

        // Copy executable (or create junction)
        // For simplicity, we use a marker file approach
        // The bootstrapper reads the marker and launches from versions/X/

        Ok(())
    }

    /// Install a version from temp directory
    pub fn install_version(&self, version: &str) -> Result<(), VersionError> {
        let temp_path = self.temp_version_path(version);
        let final_path = self.version_path(version);

        if !temp_path.exists() {
            return Err(VersionError::TempNotFound(version.to_string()));
        }

        // Atomic rename from temp to final
        if final_path.exists() {
            fs::remove_dir_all(&final_path)?;
        }
        fs::rename(&temp_path, &final_path)?;

        Ok(())
    }

    /// Remove a version
    pub fn remove_version(&self, version: &str) -> Result<(), VersionError> {
        let path = self.version_path(version);
        if path.exists() {
            fs::remove_dir_all(&path)?;
        }
        Ok(())
    }

    /// Clean up temp directories
    pub fn cleanup_temp(&self) -> Result<(), VersionError> {
        let versions_dir = self.versions_dir();
        if !versions_dir.exists() {
            return Ok(());
        }

        for entry in fs::read_dir(&versions_dir)? {
            let entry = entry?;
            let path = entry.path();
            if let Some(name) = path.file_name().and_then(|n| n.to_str()) {
                if name.starts_with(".tmp-") {
                    fs::remove_dir_all(&path)?;
                }
            }
        }
        Ok(())
    }

    /// Get the binary path for a version
    pub fn get_binary_path(&self, version: &str) -> PathBuf {
        let version_dir = self.version_path(version);
        
        #[cfg(target_os = "windows")]
        {
            version_dir.join("airdb.exe")
        }
        
        #[cfg(not(target_os = "windows"))]
        {
            version_dir.join("airdb-desktop")
        }
    }

    /// Get the CLI binary path for a version
    pub fn get_cli_path(&self, version: &str) -> PathBuf {
        let version_dir = self.version_path(version);
        
        #[cfg(target_os = "windows")]
        {
            version_dir.join("airdb-cli.exe")
        }
        
        #[cfg(not(target_os = "windows"))]
        {
            version_dir.join("airdb-cli")
        }
    }
}

/// Version manager errors
#[derive(Debug)]
pub enum VersionError {
    NoHomeDir,
    IoError(io::Error),
    VersionNotFound(String),
    TempNotFound(String),
}

impl From<io::Error> for VersionError {
    fn from(e: io::Error) -> Self {
        Self::IoError(e)
    }
}

impl std::fmt::Display for VersionError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::NoHomeDir => write!(f, "Could not determine home directory"),
            Self::IoError(e) => write!(f, "IO error: {}", e),
            Self::VersionNotFound(v) => write!(f, "Version {} not found", v),
            Self::TempNotFound(v) => write!(f, "Temp directory for {} not found", v),
        }
    }
}

impl std::error::Error for VersionError {}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn test_version_manager_init() {
        let dir = tempdir().unwrap();
        let vm = VersionManager::with_base_dir(dir.path().to_path_buf());
        vm.init().unwrap();
        
        assert!(vm.versions_dir().exists());
        assert!(vm.updater_dir().exists());
        assert!(vm.logs_dir().exists());
    }

    #[test]
    fn test_list_versions_empty() {
        let dir = tempdir().unwrap();
        let vm = VersionManager::with_base_dir(dir.path().to_path_buf());
        vm.init().unwrap();
        
        let versions = vm.list_versions().unwrap();
        assert!(versions.is_empty());
    }

    #[test]
    fn test_list_versions() {
        let dir = tempdir().unwrap();
        let vm = VersionManager::with_base_dir(dir.path().to_path_buf());
        vm.init().unwrap();
        
        // Create some version directories
        fs::create_dir_all(vm.version_path("0.1.0")).unwrap();
        fs::create_dir_all(vm.version_path("0.2.0")).unwrap();
        fs::create_dir_all(vm.version_path("0.1.1")).unwrap();
        
        let versions = vm.list_versions().unwrap();
        assert_eq!(versions, vec!["0.1.0", "0.1.1", "0.2.0"]);
    }

    #[test]
    fn test_is_version_installed() {
        let dir = tempdir().unwrap();
        let vm = VersionManager::with_base_dir(dir.path().to_path_buf());
        vm.init().unwrap();
        
        fs::create_dir_all(vm.version_path("0.1.0")).unwrap();
        
        assert!(vm.is_version_installed("0.1.0"));
        assert!(!vm.is_version_installed("0.2.0"));
    }

    #[cfg(unix)]
    #[test]
    fn test_switch_version() {
        let dir = tempdir().unwrap();
        let vm = VersionManager::with_base_dir(dir.path().to_path_buf());
        vm.init().unwrap();
        
        fs::create_dir_all(vm.version_path("0.1.0")).unwrap();
        fs::create_dir_all(vm.version_path("0.2.0")).unwrap();
        
        vm.switch_version("0.1.0").unwrap();
        assert_eq!(vm.get_current_version().unwrap(), Some("0.1.0".to_string()));
        
        vm.switch_version("0.2.0").unwrap();
        assert_eq!(vm.get_current_version().unwrap(), Some("0.2.0".to_string()));
    }
}
