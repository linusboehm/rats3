use super::{Backend, Entry, ListResult, PreviewContent};
use anyhow::{Context, Result};
use async_trait::async_trait;
use std::fs;
use std::path::{Path, PathBuf};

/// Local filesystem backend for testing
pub struct LocalBackend {
    root: PathBuf,
}

impl LocalBackend {
    pub fn new(root: PathBuf) -> Result<Self> {
        if !root.exists() {
            anyhow::bail!("Root directory does not exist: {}", root.display());
        }
        if !root.is_dir() {
            anyhow::bail!("Root path is not a directory: {}", root.display());
        }
        Ok(Self { root })
    }

    fn resolve_path(&self, prefix: &str) -> PathBuf {
        let prefix = prefix.trim_start_matches('/');
        if prefix.is_empty() {
            self.root.clone()
        } else {
            self.root.join(prefix)
        }
    }
}

#[async_trait]
impl Backend for LocalBackend {
    async fn list(&self, prefix: &str) -> Result<ListResult> {
        let path = self.resolve_path(prefix);

        let mut entries = Vec::new();
        let dir_entries = fs::read_dir(&path)
            .with_context(|| format!("Failed to read directory: {}", path.display()))?;

        for entry in dir_entries {
            let entry = entry?;
            let metadata = entry.metadata()?;
            let name = entry.file_name().to_string_lossy().to_string();

            entries.push(Entry {
                name,
                is_dir: metadata.is_dir(),
                size: if metadata.is_file() {
                    Some(metadata.len())
                } else {
                    None
                },
                modified: metadata
                    .modified()
                    .ok()
                    .and_then(|t| {
                        t.duration_since(std::time::UNIX_EPOCH)
                            .ok()
                            .map(|d| d.as_secs())
                    })
                    .map(|secs| {
                        // Format as ISO 8601
                        let dt = chrono::DateTime::from_timestamp(secs as i64, 0)
                            .unwrap_or_default();
                        dt.format("%Y-%m-%d %H:%M:%S").to_string()
                    }),
            });
        }

        // Sort: directories first, then by name
        entries.sort_by(|a, b| {
            match (a.is_dir, b.is_dir) {
                (true, false) => std::cmp::Ordering::Less,
                (false, true) => std::cmp::Ordering::Greater,
                _ => a.name.cmp(&b.name),
            }
        });

        Ok(ListResult {
            entries,
            prefix: prefix.to_string(),
        })
    }

    async fn get_preview(&self, path: &str, max_size: usize) -> Result<PreviewContent> {
        let file_path = self.resolve_path(path);

        if !file_path.exists() {
            return Ok(PreviewContent::Error("File not found".to_string()));
        }

        let metadata = fs::metadata(&file_path)?;
        if !metadata.is_file() {
            return Ok(PreviewContent::Error("Not a file".to_string()));
        }

        let size = metadata.len();
        if size > max_size as u64 {
            return Ok(PreviewContent::TooLarge { size });
        }

        // Try to read as text
        match fs::read_to_string(&file_path) {
            Ok(content) => Ok(PreviewContent::Text(content)),
            Err(_) => {
                // Binary file
                let mime_type = mime_guess::from_path(&file_path)
                    .first()
                    .map(|m| m.to_string());
                Ok(PreviewContent::Binary { size, mime_type })
            }
        }
    }

    async fn download_file(
        &self,
        path: &str,
        destination: &Path,
        progress_callback: Option<crate::backend::ProgressCallback>,
    ) -> Result<()> {
        use tokio::io::{AsyncReadExt, AsyncWriteExt};

        let source = self.resolve_path(path);

        // Get file size for progress reporting
        let metadata = tokio::fs::metadata(&source).await
            .with_context(|| format!("Failed to read metadata for {}", source.display()))?;
        let total_size = metadata.len();

        // Open source and destination files
        let mut src_file = tokio::fs::File::open(&source).await
            .with_context(|| format!("Failed to open {}", source.display()))?;
        let mut dest_file = tokio::fs::File::create(destination).await
            .with_context(|| format!("Failed to create {}", destination.display()))?;

        // Copy with progress reporting
        let mut buffer = vec![0u8; 8192];
        let mut downloaded = 0u64;

        loop {
            let n = src_file.read(&mut buffer).await
                .with_context(|| format!("Failed to read from {}", source.display()))?;

            if n == 0 {
                break;
            }

            dest_file.write_all(&buffer[..n]).await
                .with_context(|| format!("Failed to write to {}", destination.display()))?;

            downloaded += n as u64;

            if let Some(ref callback) = progress_callback {
                callback(downloaded, Some(total_size));
            }
        }

        Ok(())
    }

    fn get_display_path(&self, prefix: &str) -> String {
        let path = self.resolve_path(prefix);
        format!("local://{}", path.display())
    }

    fn get_parent(&self, prefix: &str) -> Option<String> {
        let prefix = prefix.trim_start_matches('/').trim_end_matches('/');
        if prefix.is_empty() {
            return None;
        }

        let path = Path::new(prefix);
        path.parent().and_then(|p| {
            let parent = p.to_string_lossy().to_string();
            if parent.is_empty() {
                Some(String::new())
            } else {
                Some(parent)
            }
        })
    }
}
