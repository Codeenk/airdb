//! Installer Module
//!
//! Handles installation tasks including PATH setup and binary placement

use std::path::{Path, PathBuf};
use std::fs;

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct Installer {
    app_name: String,
}

impl Installer {
    pub fn new(app_name: &str) -> Self {
        Self {
            app_name: app_name.to_string(),
        }
    }

    /// Install AirDB to the system PATH
    pub fn install_to_path(&self, source_binaries: &Path) -> Result<InstallInfo, InstallerError> {
        #[cfg(target_os = "windows")]
        return self.install_windows(source_binaries);
        
        #[cfg(target_os = "linux")]
        return self.install_linux(source_binaries);
        
        #[cfg(target_os = "macos")]
        return self.install_macos(source_binaries);
        
        #[cfg(not(any(target_os = "windows", target_os = "linux", target_os = "macos")))]
        Err(InstallerError::UnsupportedPlatform)
    }

    /// Check if AirDB is installed and available in PATH
    pub fn is_installed(&self) -> bool {
        which::which("airdb").is_ok()
    }

    /// Get the installation directory for binaries
    pub fn get_bin_dir() -> Result<PathBuf, InstallerError> {
        #[cfg(target_os = "windows")]
        {
            dirs::data_local_dir()
                .map(|d| d.join("AirDB").join("bin"))
                .ok_or(InstallerError::NoHomeDir)
        }
        
        #[cfg(target_os = "linux")]
        {
            // Prefer user-local bin (no root required)
            if let Some(home) = dirs::home_dir() {
                Ok(home.join(".local/bin"))
            } else {
                Err(InstallerError::NoHomeDir)
            }
        }
        
        #[cfg(target_os = "macos")]
        {
            // Prefer /usr/local/bin if writable, otherwise ~/.local/bin
            if is_writable("/usr/local/bin") {
                Ok(PathBuf::from("/usr/local/bin"))
            } else if let Some(home) = dirs::home_dir() {
                Ok(home.join(".local/bin"))
            } else {
                Err(InstallerError::NoHomeDir)
            }
        }
    }

    // ============ Windows Implementation ============
    
    #[cfg(target_os = "windows")]
    fn install_windows(&self, source_binaries: &Path) -> Result<InstallInfo, InstallerError> {
        use winreg::enums::*;
        use winreg::RegKey;
        
        let bin_dir = Self::get_bin_dir()?;
        fs::create_dir_all(&bin_dir)
            .map_err(|e| InstallerError::IoError(e.to_string()))?;
        
        // Copy binaries
        let binaries = vec![
            ("airdb.exe", "airdb-bootstrap.exe"),
            ("airdb-bootstrap.exe", "airdb-bootstrap.exe"),
            ("airdb-cli.exe", "airdb-cli.exe"),
        ];
        
        for (target_name, source_name) in binaries {
            let source = source_binaries.join(source_name);
            let target = bin_dir.join(target_name);
            
            if source.exists() {
                fs::copy(&source, &target)
                    .map_err(|e| InstallerError::IoError(e.to_string()))?;
            }
        }
        
        // Add to User PATH (not System PATH - no admin required)
        let hkcu = RegKey::predef(HKEY_CURRENT_USER);
        let env_key = hkcu.open_subkey_with_flags("Environment", KEY_READ | KEY_WRITE)
            .map_err(|e| InstallerError::RegistryError(e.to_string()))?;
        
        let current_path: String = env_key.get_value("Path")
            .unwrap_or_else(|_| String::new());
        
        let bin_dir_str = bin_dir.to_string_lossy();
        let path_added = if !current_path.split(';').any(|p| p.trim() == bin_dir_str.as_ref()) {
            let new_path = if current_path.is_empty() {
                bin_dir_str.to_string()
            } else {
                format!("{};{}", current_path, bin_dir_str)
            };
            
            env_key.set_value("Path", &new_path)
                .map_err(|e| InstallerError::RegistryError(e.to_string()))?;
            
            true
        } else {
            false
        };
        
        Ok(InstallInfo {
            bin_dir,
            path_added,
            restart_required: path_added,
            message: if path_added {
                "Added to PATH. Please restart your terminal.".to_string()
            } else {
                "Already in PATH.".to_string()
            },
        })
    }
    
    // ============ Linux Implementation ============
    
    #[cfg(target_os = "linux")]
    fn install_linux(&self, _source_binaries: &Path) -> Result<InstallInfo, InstallerError> {
        let bin_dir = Self::get_bin_dir()?;
        fs::create_dir_all(&bin_dir)
            .map_err(|e| InstallerError::IoError(e.to_string()))?;
        
        // Create symlinks to the versioned binaries
        let current_version_dir = dirs::home_dir()
            .ok_or(InstallerError::NoHomeDir)?
            .join(".local/share/airdb/current");
        
        let binaries = vec![
            ("airdb", "airdb-bootstrap"),
            ("airdb-bootstrap", "airdb-bootstrap"),
        ];
        
        for (link_name, target_name) in binaries {
            let target = current_version_dir.join(target_name);
            let link = bin_dir.join(link_name);
            
            // Remove existing symlink/file
            if link.exists() || link.symlink_metadata().is_ok() {
                fs::remove_file(&link).ok();
            }
            
            // Create symlink
            std::os::unix::fs::symlink(&target, &link)
                .map_err(|e| InstallerError::IoError(e.to_string()))?;
        }
        
        // Check if ~/.local/bin is in PATH
        let path_in_env = std::env::var("PATH")
            .unwrap_or_default()
            .split(':')
            .any(|p| p == bin_dir.to_string_lossy());
        
        let message = if !path_in_env {
            format!(
                "⚠️  {} is not in your PATH.\n\
                Add this line to your ~/.bashrc or ~/.zshrc:\n\
                export PATH=\"$HOME/.local/bin:$PATH\"",
                bin_dir.display()
            )
        } else {
            "✅ Installed successfully!".to_string()
        };
        
        Ok(InstallInfo {
            bin_dir,
            path_added: path_in_env,
            restart_required: false,
            message,
        })
    }
    
    // ============ macOS Implementation ============
    
    #[cfg(target_os = "macos")]
    fn install_macos(&self, source_binaries: &Path) -> Result<InstallInfo, InstallerError> {
        let bin_dir = Self::get_bin_dir()?;
        fs::create_dir_all(&bin_dir)
            .map_err(|e| InstallerError::IoError(e.to_string()))?;
        
        // Create symlinks
        let current_version_dir = dirs::home_dir()
            .ok_or(InstallerError::NoHomeDir)?
            .join("Library/Application Support/AirDB/current");
        
        let binaries = vec![
            ("airdb", "airdb-bootstrap"),
            ("airdb-bootstrap", "airdb-bootstrap"),
        ];
        
        for (link_name, target_name) in binaries {
            let target = current_version_dir.join(target_name);
            let link = bin_dir.join(link_name);
            
            if link.exists() || link.symlink_metadata().is_ok() {
                fs::remove_file(&link).ok();
            }
            
            std::os::unix::fs::symlink(&target, &link)
                .map_err(|e| InstallerError::IoError(e.to_string()))?;
        }
        
        let message = "✅ Installed successfully!\n\
            The 'airdb' command is now available.".to_string();
        
        Ok(InstallInfo {
            bin_dir,
            path_added: true,
            restart_required: false,
            message,
        })
    }
}

/// Installation result information
#[derive(Debug, Clone)]
pub struct InstallInfo {
    pub bin_dir: PathBuf,
    pub path_added: bool,
    pub restart_required: bool,
    pub message: String,
}

/// Installer errors
#[derive(Debug, Clone)]
pub enum InstallerError {
    UnsupportedPlatform,
    NoHomeDir,
    IoError(String),
    #[cfg(target_os = "windows")]
    RegistryError(String),
}

impl std::fmt::Display for InstallerError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            InstallerError::UnsupportedPlatform => write!(f, "Unsupported platform"),
            InstallerError::NoHomeDir => write!(f, "Could not find home directory"),
            InstallerError::IoError(e) => write!(f, "IO error: {}", e),
            #[cfg(target_os = "windows")]
            InstallerError::RegistryError(e) => write!(f, "Registry error: {}", e),
        }
    }
}

impl std::error::Error for InstallerError {}

/// Check if a path is writable
#[cfg(target_os = "macos")]
fn is_writable(path: &str) -> bool {
    use std::os::unix::fs::PermissionsExt;
    
    std::fs::metadata(path)
        .map(|m| m.permissions().mode() & 0o200 != 0)
        .unwrap_or(false)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_installer_creation() {
        let installer = Installer::new("AirDB");
        assert_eq!(installer.app_name, "AirDB");
    }

    #[test]
    fn test_get_bin_dir() {
        let result = Installer::get_bin_dir();
        assert!(result.is_ok());
    }
}
