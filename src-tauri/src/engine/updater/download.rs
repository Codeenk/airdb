//! HTTP Download Manager
//! 
//! Handles update downloads with resume support and progress tracking.

use std::path::{Path, PathBuf};
use std::fs::{self, File};
use std::io::{self, Write, Read};
use reqwest::header::{RANGE, CONTENT_LENGTH, ACCEPT_RANGES};

/// Download result with metadata
#[derive(Debug)]
pub struct DownloadResult {
    pub path: PathBuf,
    pub bytes_downloaded: u64,
    pub resumed: bool,
}

/// Download error types
#[derive(Debug)]
pub enum DownloadError {
    Network(String),
    Io(io::Error),
    InvalidResponse(String),
    Cancelled,
}

impl From<io::Error> for DownloadError {
    fn from(e: io::Error) -> Self {
        Self::Io(e)
    }
}

impl From<reqwest::Error> for DownloadError {
    fn from(e: reqwest::Error) -> Self {
        Self::Network(e.to_string())
    }
}

impl std::fmt::Display for DownloadError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Network(e) => write!(f, "Network error: {}", e),
            Self::Io(e) => write!(f, "IO error: {}", e),
            Self::InvalidResponse(e) => write!(f, "Invalid response: {}", e),
            Self::Cancelled => write!(f, "Download cancelled"),
        }
    }
}

/// Download manager for update files
pub struct Downloader {
    client: reqwest::Client,
}

impl Downloader {
    /// Create a new downloader
    pub fn new() -> Self {
        let client = reqwest::Client::builder()
            .user_agent("AirDB-Updater")
            .build()
            .unwrap_or_default();
        Self { client }
    }

    /// Download a file with progress callback
    /// 
    /// Supports resuming interrupted downloads.
    pub async fn download<F>(
        &self,
        url: &str,
        dest: &Path,
        mut on_progress: F,
    ) -> Result<DownloadResult, DownloadError>
    where
        F: FnMut(u64, u64), // (downloaded, total)
    {
        // Check for partial download
        let partial_path = dest.with_extension("partial");
        let mut start_byte: u64 = 0;
        let mut file: File;
        let resumed: bool;

        if partial_path.exists() {
            let metadata = fs::metadata(&partial_path)?;
            start_byte = metadata.len();
            file = fs::OpenOptions::new()
                .write(true)
                .append(true)
                .open(&partial_path)?;
            resumed = true;
        } else {
            // Ensure parent directory exists
            if let Some(parent) = partial_path.parent() {
                fs::create_dir_all(parent)?;
            }
            file = File::create(&partial_path)?;
            resumed = false;
        }

        // Build request with range header if resuming
        let mut request = self.client.get(url);
        if start_byte > 0 {
            request = request.header(RANGE, format!("bytes={}-", start_byte));
        }

        let response = request.send().await?;

        if !response.status().is_success() && response.status().as_u16() != 206 {
            return Err(DownloadError::InvalidResponse(
                format!("HTTP {}", response.status())
            ));
        }

        // Get total size
        let total_size = if let Some(content_length) = response.headers().get(CONTENT_LENGTH) {
            content_length
                .to_str()
                .ok()
                .and_then(|s| s.parse::<u64>().ok())
                .map(|len| len + start_byte)
                .unwrap_or(0)
        } else {
            0
        };

        // Download in chunks
        let mut downloaded = start_byte;
        let mut stream = response.bytes_stream();
        
        use futures_util::StreamExt;
        while let Some(chunk_result) = stream.next().await {
            let chunk = chunk_result.map_err(|e| DownloadError::Network(e.to_string()))?;
            file.write_all(&chunk)?;
            downloaded += chunk.len() as u64;
            on_progress(downloaded, total_size);
        }

        file.flush()?;
        drop(file);

        // Rename to final destination
        fs::rename(&partial_path, dest)?;

        Ok(DownloadResult {
            path: dest.to_path_buf(),
            bytes_downloaded: downloaded,
            resumed,
        })
    }

    /// Check if server supports resume
    pub async fn supports_resume(&self, url: &str) -> bool {
        if let Ok(response) = self.client.head(url).send().await {
            response.headers()
                .get(ACCEPT_RANGES)
                .map(|v| v.to_str().unwrap_or("") == "bytes")
                .unwrap_or(false)
        } else {
            false
        }
    }

    /// Get the content length without downloading
    pub async fn get_content_length(&self, url: &str) -> Option<u64> {
        self.client
            .head(url)
            .send()
            .await
            .ok()?
            .headers()
            .get(CONTENT_LENGTH)?
            .to_str()
            .ok()?
            .parse()
            .ok()
    }
}

impl Default for Downloader {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_downloader_creation() {
        let downloader = Downloader::new();
        // Just verify it creates without panic
        assert!(true);
    }
}
