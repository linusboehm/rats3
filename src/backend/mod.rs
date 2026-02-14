use anyhow::Result;
use async_trait::async_trait;
use std::path::Path;

pub mod local;

#[cfg(feature = "s3")]
pub mod s3;

/// Represents a single entry in a directory listing
#[derive(Debug, Clone)]
pub struct Entry {
    pub name: String,
    pub is_dir: bool,
    pub size: Option<u64>,
    pub modified: Option<String>,
}

/// Result of listing a directory/prefix
#[derive(Debug)]
pub struct ListResult {
    pub entries: Vec<Entry>,
    pub prefix: String,
}

/// Preview content for a file
#[derive(Debug, Clone)]
pub enum PreviewContent {
    Text(String),
    Binary { size: u64, mime_type: Option<String> },
    TooLarge { size: u64 },
    Error(String),
}

/// Progress information for downloads
#[derive(Debug, Clone)]
pub struct DownloadProgress {
    pub downloaded: u64,
    pub total: Option<u64>,
    pub current_file: String,
}

/// Callback for download progress updates
pub type ProgressCallback = Box<dyn Fn(u64, Option<u64>) + Send + Sync>;

/// Backend trait for different storage systems (S3, local filesystem)
#[async_trait]
pub trait Backend: Send + Sync {
    /// List entries at the given prefix/path
    async fn list(&self, prefix: &str) -> Result<ListResult>;

    /// Get preview content for a file
    async fn get_preview(&self, path: &str, max_size: usize) -> Result<PreviewContent>;

    /// Download a single file to the destination path
    /// The progress callback is called with (downloaded_bytes, total_bytes)
    async fn download_file(
        &self,
        path: &str,
        destination: &Path,
        progress_callback: Option<ProgressCallback>,
    ) -> Result<()>;

    /// Get a human-readable display path
    fn get_display_path(&self, prefix: &str) -> String;

    /// Get the parent prefix/path (for navigating up)
    fn get_parent(&self, prefix: &str) -> Option<String>;
}
