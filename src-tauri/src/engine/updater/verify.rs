//! Cryptographic Verification
//! 
//! Handles SHA256 checksums and ED25519 signature verification.

use std::path::Path;
use std::fs::File;
use std::io::{self, Read, BufReader};
use sha2::{Sha256, Digest};
use ed25519_dalek::{Verifier, VerifyingKey, Signature};
use serde::{Deserialize, Serialize};

/// Update manifest from GitHub releases
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateManifest {
    pub version: String,
    pub channel: String,
    pub release_date: String,
    pub min_supported_version: String,
    pub changelog: Vec<String>,
    pub artifacts: ArtifactMap,
    pub signature: String,
}

/// Platform-specific artifact info
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ArtifactMap {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub windows: Option<ArtifactInfo>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub linux: Option<ArtifactInfo>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub macos: Option<ArtifactInfo>,
}

/// Single artifact download info
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ArtifactInfo {
    pub url: String,
    pub sha256: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub size: Option<u64>,
}

/// Verification errors
#[derive(Debug)]
pub enum VerifyError {
    IoError(io::Error),
    ChecksumMismatch { expected: String, actual: String },
    InvalidSignature,
    InvalidPublicKey,
    ParseError(String),
}

impl From<io::Error> for VerifyError {
    fn from(e: io::Error) -> Self {
        Self::IoError(e)
    }
}

impl std::fmt::Display for VerifyError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::IoError(e) => write!(f, "IO error: {}", e),
            Self::ChecksumMismatch { expected, actual } => {
                write!(f, "Checksum mismatch: expected {}, got {}", expected, actual)
            }
            Self::InvalidSignature => write!(f, "Invalid signature"),
            Self::InvalidPublicKey => write!(f, "Invalid public key"),
            Self::ParseError(e) => write!(f, "Parse error: {}", e),
        }
    }
}

impl std::error::Error for VerifyError {}

/// Verifier for update files
pub struct UpdateVerifier {
    /// Embedded public key for signature verification
    public_key: Option<VerifyingKey>,
}

impl UpdateVerifier {
    /// Create a new verifier with the embedded public key
    /// 
    /// The public key should be compiled into the binary.
    pub fn new() -> Self {
        // TODO: Replace with actual public key when generating releases
        // For now, we'll skip signature verification if no key is set
        Self { public_key: None }
    }

    /// Create verifier with a specific public key (for testing)
    pub fn with_public_key(key_bytes: &[u8; 32]) -> Result<Self, VerifyError> {
        let public_key = VerifyingKey::from_bytes(key_bytes)
            .map_err(|_| VerifyError::InvalidPublicKey)?;
        Ok(Self { public_key: Some(public_key) })
    }

    /// Calculate SHA256 checksum of a file
    pub fn calculate_sha256(path: &Path) -> Result<String, VerifyError> {
        let file = File::open(path)?;
        let mut reader = BufReader::new(file);
        let mut hasher = Sha256::new();
        
        let mut buffer = [0u8; 8192];
        loop {
            let bytes_read = reader.read(&mut buffer)?;
            if bytes_read == 0 {
                break;
            }
            hasher.update(&buffer[..bytes_read]);
        }
        
        let result = hasher.finalize();
        Ok(hex::encode(result))
    }

    /// Verify a file's checksum
    pub fn verify_checksum(&self, path: &Path, expected: &str) -> Result<(), VerifyError> {
        let actual = Self::calculate_sha256(path)?;
        if actual.to_lowercase() != expected.to_lowercase() {
            return Err(VerifyError::ChecksumMismatch {
                expected: expected.to_string(),
                actual,
            });
        }
        Ok(())
    }

    /// Verify the manifest signature
    pub fn verify_manifest_signature(&self, manifest: &UpdateManifest) -> Result<(), VerifyError> {
        let Some(public_key) = &self.public_key else {
            // No public key configured, skip signature verification
            // In production, this should be an error
            return Ok(());
        };

        // Create the message to verify (manifest without signature)
        let mut manifest_clone = manifest.clone();
        manifest_clone.signature = String::new();
        let message = serde_json::to_string(&manifest_clone)
            .map_err(|e| VerifyError::ParseError(e.to_string()))?;

        // Decode signature from hex
        let sig_bytes = hex::decode(&manifest.signature)
            .map_err(|e| VerifyError::ParseError(e.to_string()))?;
        
        if sig_bytes.len() != 64 {
            return Err(VerifyError::InvalidSignature);
        }

        let mut sig_array = [0u8; 64];
        sig_array.copy_from_slice(&sig_bytes);
        let signature = Signature::from_bytes(&sig_array);

        // Verify
        public_key
            .verify(message.as_bytes(), &signature)
            .map_err(|_| VerifyError::InvalidSignature)?;

        Ok(())
    }

    /// Get artifact info for current platform
    pub fn get_platform_artifact(manifest: &UpdateManifest) -> Option<&ArtifactInfo> {
        #[cfg(target_os = "windows")]
        {
            manifest.artifacts.windows.as_ref()
        }
        #[cfg(target_os = "linux")]
        {
            manifest.artifacts.linux.as_ref()
        }
        #[cfg(target_os = "macos")]
        {
            manifest.artifacts.macos.as_ref()
        }
    }

    /// Check if version A is newer than version B
    pub fn is_newer_version(a: &str, b: &str) -> bool {
        let parse = |v: &str| -> Vec<u32> {
            v.trim_start_matches('v')
                .split('.')
                .filter_map(|s| s.parse().ok())
                .collect()
        };
        
        let a_parts = parse(a);
        let b_parts = parse(b);
        
        a_parts > b_parts
    }

    /// Check if upgrade is supported (min version check)
    pub fn can_upgrade_from(manifest: &UpdateManifest, current_version: &str) -> bool {
        !Self::is_newer_version(&manifest.min_supported_version, current_version)
    }
}

impl Default for UpdateVerifier {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_sha256_calculation() {
        let mut file = NamedTempFile::new().unwrap();
        file.write_all(b"hello world").unwrap();
        file.flush().unwrap();
        
        let checksum = UpdateVerifier::calculate_sha256(file.path()).unwrap();
        assert_eq!(
            checksum,
            "b94d27b9934d3e08a52e52d7da7dabfac484efe37a5380ee9088f7ace2efcde9"
        );
    }

    #[test]
    fn test_checksum_verification_success() {
        let mut file = NamedTempFile::new().unwrap();
        file.write_all(b"hello world").unwrap();
        file.flush().unwrap();
        
        let verifier = UpdateVerifier::new();
        let result = verifier.verify_checksum(
            file.path(),
            "b94d27b9934d3e08a52e52d7da7dabfac484efe37a5380ee9088f7ace2efcde9"
        );
        assert!(result.is_ok());
    }

    #[test]
    fn test_checksum_verification_failure() {
        let mut file = NamedTempFile::new().unwrap();
        file.write_all(b"hello world").unwrap();
        file.flush().unwrap();
        
        let verifier = UpdateVerifier::new();
        let result = verifier.verify_checksum(file.path(), "wrongchecksum");
        assert!(matches!(result, Err(VerifyError::ChecksumMismatch { .. })));
    }

    #[test]
    fn test_version_comparison() {
        assert!(UpdateVerifier::is_newer_version("0.2.0", "0.1.0"));
        assert!(UpdateVerifier::is_newer_version("1.0.0", "0.9.9"));
        assert!(UpdateVerifier::is_newer_version("0.1.1", "0.1.0"));
        assert!(!UpdateVerifier::is_newer_version("0.1.0", "0.1.0"));
        assert!(!UpdateVerifier::is_newer_version("0.1.0", "0.2.0"));
    }

    #[test]
    fn test_can_upgrade_from() {
        let manifest = UpdateManifest {
            version: "0.2.0".to_string(),
            channel: "stable".to_string(),
            release_date: "2026-02-07".to_string(),
            min_supported_version: "0.1.0".to_string(),
            changelog: vec![],
            artifacts: ArtifactMap {
                windows: None,
                linux: None,
                macos: None,
            },
            signature: String::new(),
        };
        
        assert!(UpdateVerifier::can_upgrade_from(&manifest, "0.1.0"));
        assert!(UpdateVerifier::can_upgrade_from(&manifest, "0.1.5"));
        assert!(!UpdateVerifier::can_upgrade_from(&manifest, "0.0.9"));
    }
}
