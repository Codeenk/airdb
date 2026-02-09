//! Compatibility Testing Module
//!
//! Version matrix tests, upgrade path verification, and downgrade safety

use serde::{Deserialize, Serialize};
use std::path::Path;
use chrono::{DateTime, Utc};

/// Current compatibility test format version
pub const COMPAT_VERSION: u32 = 1;

/// Version compatibility result
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum CompatResult {
    Compatible,
    Warning,
    Incompatible,
    Unknown,
}

/// A single compatibility test
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompatTest {
    pub name: String,
    pub from_version: String,
    pub to_version: String,
    pub result: CompatResult,
    pub duration_ms: u64,
    pub details: Option<String>,
    pub timestamp: DateTime<Utc>,
}

/// Version matrix entry
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VersionMatrixEntry {
    pub source_version: String,
    pub target_version: String,
    pub upgrade_result: CompatResult,
    pub downgrade_result: CompatResult,
    pub tested_at: DateTime<Utc>,
}

/// The version compatibility matrix
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct VersionMatrix {
    pub version: u32,
    pub entries: Vec<VersionMatrixEntry>,
    pub last_updated: Option<DateTime<Utc>>,
}

impl VersionMatrix {
    pub fn new() -> Self {
        Self {
            version: COMPAT_VERSION,
            entries: Vec::new(),
            last_updated: Some(Utc::now()),
        }
    }

    /// Add or update a matrix entry
    pub fn add_entry(&mut self, entry: VersionMatrixEntry) {
        // Remove existing entry for same version pair
        self.entries.retain(|e| {
            !(e.source_version == entry.source_version && e.target_version == entry.target_version)
        });
        self.entries.push(entry);
        self.last_updated = Some(Utc::now());
    }

    /// Check if upgrade is safe
    pub fn is_upgrade_safe(&self, from: &str, to: &str) -> CompatResult {
        self.entries.iter()
            .find(|e| e.source_version == from && e.target_version == to)
            .map(|e| e.upgrade_result)
            .unwrap_or(CompatResult::Unknown)
    }

    /// Check if downgrade is safe
    pub fn is_downgrade_safe(&self, from: &str, to: &str) -> CompatResult {
        self.entries.iter()
            .find(|e| e.source_version == from && e.target_version == to)
            .map(|e| e.downgrade_result)
            .unwrap_or(CompatResult::Unknown)
    }

    /// Load matrix from file
    pub fn load(project_dir: &Path) -> std::io::Result<Self> {
        let matrix_path = project_dir.join(".airdb").join("version-matrix.json");
        
        if !matrix_path.exists() {
            return Ok(Self::new());
        }
        
        let content = std::fs::read_to_string(&matrix_path)?;
        serde_json::from_str(&content)
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))
    }

    /// Save matrix to file
    pub fn save(&self, project_dir: &Path) -> std::io::Result<()> {
        let matrix_dir = project_dir.join(".airdb");
        std::fs::create_dir_all(&matrix_dir)?;
        
        let matrix_path = matrix_dir.join("version-matrix.json");
        let content = serde_json::to_string_pretty(self)
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))?;
        
        std::fs::write(&matrix_path, content)
    }
}

/// Compatibility tester
pub struct CompatibilityTester {
    current_version: String,
    min_supported_version: String,
}

impl CompatibilityTester {
    pub fn new(current_version: &str, min_supported: &str) -> Self {
        Self {
            current_version: current_version.to_string(),
            min_supported_version: min_supported.to_string(),
        }
    }

    /// Verify an upgrade path is safe
    pub fn verify_upgrade_path(&self, from: &str, to: &str) -> CompatTest {
        let start = std::time::Instant::now();
        
        let (result, details) = if !self.is_version_supported(from) {
            (CompatResult::Incompatible, Some(format!("Source version {} is below minimum supported", from)))
        } else if !self.is_version_newer(from, to) {
            (CompatResult::Incompatible, Some("Target version must be newer than source".to_string()))
        } else if self.has_breaking_changes(from, to) {
            (CompatResult::Warning, Some("Upgrade path includes breaking changes".to_string()))
        } else {
            (CompatResult::Compatible, None)
        };

        CompatTest {
            name: "upgrade_path_verification".to_string(),
            from_version: from.to_string(),
            to_version: to.to_string(),
            result,
            duration_ms: start.elapsed().as_millis() as u64,
            details,
            timestamp: Utc::now(),
        }
    }

    /// Verify a downgrade is safe
    pub fn verify_downgrade_safety(&self, from: &str, to: &str) -> CompatTest {
        let start = std::time::Instant::now();
        
        let (result, details) = if !self.is_version_supported(to) {
            (CompatResult::Incompatible, Some(format!("Target version {} is below minimum supported", to)))
        } else if !self.is_version_newer(to, from) {
            (CompatResult::Incompatible, Some("Source version must be newer than target for downgrade".to_string()))
        } else if self.has_data_migrations(to, from) {
            (CompatResult::Warning, Some("Downgrade may lose data from newer migrations".to_string()))
        } else {
            (CompatResult::Compatible, None)
        };

        CompatTest {
            name: "downgrade_safety_check".to_string(),
            from_version: from.to_string(),
            to_version: to.to_string(),
            result,
            duration_ms: start.elapsed().as_millis() as u64,
            details,
            timestamp: Utc::now(),
        }
    }

    /// Check schema compatibility
    pub fn verify_schema_compatibility(&self, current_schema: u32, target_schema: u32) -> CompatTest {
        let start = std::time::Instant::now();
        
        let (result, details) = if target_schema > current_schema + 1 {
            (CompatResult::Warning, Some("Schema version jump detected, incremental migration recommended".to_string()))
        } else if target_schema < current_schema {
            (CompatResult::Warning, Some("Schema downgrade may cause data loss".to_string()))
        } else {
            (CompatResult::Compatible, None)
        };

        CompatTest {
            name: "schema_compatibility".to_string(),
            from_version: current_schema.to_string(),
            to_version: target_schema.to_string(),
            result,
            duration_ms: start.elapsed().as_millis() as u64,
            details,
            timestamp: Utc::now(),
        }
    }

    /// Check if version is supported
    fn is_version_supported(&self, version: &str) -> bool {
        compare_versions(version, &self.min_supported_version) >= 0
    }

    /// Check if to is newer than from
    fn is_version_newer(&self, from: &str, to: &str) -> bool {
        compare_versions(to, from) > 0
    }

    /// Check for breaking changes between versions
    fn has_breaking_changes(&self, _from: &str, _to: &str) -> bool {
        // Would check a breaking changes registry
        false
    }

    /// Check if there are data migrations between versions
    fn has_data_migrations(&self, _from: &str, _to: &str) -> bool {
        // Would check migration registry
        false
    }
}

/// Compare semantic versions (-1: a < b, 0: a == b, 1: a > b)
fn compare_versions(a: &str, b: &str) -> i32 {
    let parse = |v: &str| -> Vec<u32> {
        v.split('.')
            .filter_map(|p| p.parse().ok())
            .collect()
    };
    
    let va = parse(a);
    let vb = parse(b);
    
    for i in 0..va.len().max(vb.len()) {
        let pa = va.get(i).copied().unwrap_or(0);
        let pb = vb.get(i).copied().unwrap_or(0);
        
        if pa < pb { return -1; }
        if pa > pb { return 1; }
    }
    
    0
}

/// UI communication for updates
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateNotification {
    pub version: String,
    pub release_notes: String,
    pub breaking_changes: Vec<String>,
    pub new_features: Vec<String>,
    pub bug_fixes: Vec<String>,
    pub compatibility_warnings: Vec<String>,
}

impl UpdateNotification {
    pub fn new(version: &str) -> Self {
        Self {
            version: version.to_string(),
            release_notes: String::new(),
            breaking_changes: Vec::new(),
            new_features: Vec::new(),
            bug_fixes: Vec::new(),
            compatibility_warnings: Vec::new(),
        }
    }

    pub fn with_release_notes(mut self, notes: &str) -> Self {
        self.release_notes = notes.to_string();
        self
    }

    pub fn add_breaking_change(mut self, change: &str) -> Self {
        self.breaking_changes.push(change.to_string());
        self
    }

    pub fn add_feature(mut self, feature: &str) -> Self {
        self.new_features.push(feature.to_string());
        self
    }

    pub fn add_bug_fix(mut self, fix: &str) -> Self {
        self.bug_fixes.push(fix.to_string());
        self
    }

    /// Check if update has breaking changes
    pub fn has_breaking_changes(&self) -> bool {
        !self.breaking_changes.is_empty()
    }

    /// Format as markdown for UI display
    pub fn to_markdown(&self) -> String {
        let mut md = format!("# What's New in v{}\n\n", self.version);
        
        if !self.release_notes.is_empty() {
            md.push_str(&self.release_notes);
            md.push_str("\n\n");
        }
        
        if !self.breaking_changes.is_empty() {
            md.push_str("## ⚠️ Breaking Changes\n\n");
            for change in &self.breaking_changes {
                md.push_str(&format!("- {}\n", change));
            }
            md.push('\n');
        }
        
        if !self.new_features.is_empty() {
            md.push_str("## ✨ New Features\n\n");
            for feature in &self.new_features {
                md.push_str(&format!("- {}\n", feature));
            }
            md.push('\n');
        }
        
        if !self.bug_fixes.is_empty() {
            md.push_str("## 🐛 Bug Fixes\n\n");
            for fix in &self.bug_fixes {
                md.push_str(&format!("- {}\n", fix));
            }
        }
        
        md
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_version_comparison() {
        assert_eq!(compare_versions("1.0.0", "1.0.0"), 0);
        assert_eq!(compare_versions("1.0.0", "1.0.1"), -1);
        assert_eq!(compare_versions("1.1.0", "1.0.0"), 1);
        assert_eq!(compare_versions("2.0.0", "1.9.9"), 1);
    }

    #[test]
    fn test_upgrade_verification() {
        let tester = CompatibilityTester::new("0.2.0", "0.1.0");
        
        let result = tester.verify_upgrade_path("0.1.0", "0.2.0");
        assert_eq!(result.result, CompatResult::Compatible);
        
        let result = tester.verify_upgrade_path("0.2.0", "0.1.0");
        assert_eq!(result.result, CompatResult::Incompatible);
    }

    #[test]
    fn test_version_matrix() {
        let mut matrix = VersionMatrix::new();
        
        matrix.add_entry(VersionMatrixEntry {
            source_version: "0.1.0".to_string(),
            target_version: "0.2.0".to_string(),
            upgrade_result: CompatResult::Compatible,
            downgrade_result: CompatResult::Warning,
            tested_at: Utc::now(),
        });
        
        assert_eq!(matrix.is_upgrade_safe("0.1.0", "0.2.0"), CompatResult::Compatible);
        assert_eq!(matrix.is_downgrade_safe("0.1.0", "0.2.0"), CompatResult::Warning);
    }

    #[test]
    fn test_update_notification() {
        let notification = UpdateNotification::new("0.2.0")
            .with_release_notes("Major update with new features")
            .add_feature("NoSQL support")
            .add_breaking_change("Config format changed");
        
        assert!(notification.has_breaking_changes());
        assert!(notification.to_markdown().contains("Breaking Changes"));
    }
}
