#![cfg(feature = "s3")]

use super::{Backend, Entry, ListResult, PreviewContent};
use anyhow::{Context, Result};
use async_trait::async_trait;
use aws_sdk_s3::Client;
use std::collections::HashSet;
use std::path::Path;

/// S3 backend implementation
pub struct S3Backend {
    client: Client,
    bucket: String,
}

impl S3Backend {
    pub async fn new(bucket: String) -> Result<Self> {
        let config = aws_config::load_defaults(aws_config::BehaviorVersion::latest()).await;
        let client = Client::new(&config);

        Ok(Self { client, bucket })
    }

    pub fn from_uri(uri: &str) -> Result<(String, String)> {
        // Parse s3://bucket/prefix
        let uri = uri.strip_prefix("s3://")
            .context("URI must start with s3://")?;

        let parts: Vec<&str> = uri.splitn(2, '/').collect();
        let bucket = parts[0].to_string();
        let prefix = if parts.len() > 1 {
            parts[1].to_string()
        } else {
            String::new()
        };

        Ok((bucket, prefix))
    }

    /// Download a large file using parallel range requests.
    /// Splits the file into 8 MB parts and fetches up to 8 concurrently,
    /// writing each part directly to its offset in a pre-allocated file.
    async fn download_multipart(
        &self,
        key: &str,
        destination: &Path,
        total_size: u64,
        progress_callback: Option<crate::backend::ProgressCallback>,
    ) -> Result<()> {
        use std::os::unix::fs::FileExt;
        use std::sync::Arc;
        use std::sync::atomic::{AtomicU64, Ordering};

        const PART_SIZE: u64 = 8 * 1024 * 1024; // 8 MB per part
        const MAX_CONCURRENT: usize = 8;

        // Pre-allocate the output file so parts can write in parallel without resizing
        let file = std::fs::File::create(destination)
            .context("Failed to create destination file")?;
        file.set_len(total_size)
            .context("Failed to pre-allocate destination file")?;
        let file = Arc::new(file);

        // Build list of (start_byte, end_byte) ranges
        let key_str = key.to_string();
        let parts: Vec<(u64, u64)> = (0..total_size)
            .step_by(PART_SIZE as usize)
            .map(|start| (start, (start + PART_SIZE - 1).min(total_size - 1)))
            .collect();

        let total_downloaded = Arc::new(AtomicU64::new(0));
        let semaphore = Arc::new(tokio::sync::Semaphore::new(MAX_CONCURRENT));
        let callback: Option<Arc<dyn Fn(u64, Option<u64>) + Send + Sync>> =
            progress_callback.map(Arc::from);

        let mut join_set = tokio::task::JoinSet::new();

        for (part_start, part_end) in parts {
            let client = self.client.clone();
            let bucket = self.bucket.clone();
            let key = key_str.clone();
            let file = file.clone();
            let total_downloaded = total_downloaded.clone();
            let callback = callback.clone();
            let semaphore = semaphore.clone();

            join_set.spawn(async move {
                let _permit = semaphore
                    .acquire()
                    .await
                    .expect("semaphore closed unexpectedly");

                let range = format!("bytes={}-{}", part_start, part_end);
                let response = client
                    .get_object()
                    .bucket(&bucket)
                    .key(&key)
                    .range(range)
                    .send()
                    .await
                    .context("Failed to request S3 object part")?;

                let bytes = response
                    .body
                    .collect()
                    .await
                    .context("Failed to read S3 object part")?
                    .into_bytes();

                // pwrite: thread-safe positional write, no seek or mutex needed
                file.write_at(&bytes, part_start)
                    .context("Failed to write part to file")?;

                // Report aggregated progress across all parts
                let prev = total_downloaded.fetch_add(bytes.len() as u64, Ordering::Relaxed);
                if let Some(ref cb) = callback {
                    cb(prev + bytes.len() as u64, Some(total_size));
                }

                Ok::<(), anyhow::Error>(())
            });
        }

        // Collect results; dropping join_set on error aborts remaining part tasks
        while let Some(result) = join_set.join_next().await {
            match result {
                Ok(Ok(())) => {}
                Ok(Err(e)) => {
                    let _ = std::fs::remove_file(destination);
                    return Err(e);
                }
                Err(join_err) => {
                    let _ = std::fs::remove_file(destination);
                    anyhow::bail!("Download part panicked: {}", join_err);
                }
            }
        }

        Ok(())
    }
}

#[async_trait]
impl Backend for S3Backend {
    /// List S3 objects at the given prefix (READ-ONLY operation)
    /// Uses ListObjectsV2 which is a read-only S3 operation
    async fn list(&self, prefix: &str) -> Result<ListResult> {

        let prefix = if prefix.is_empty() {
            "".to_string()
        } else {
            format!("{}/", prefix.trim_end_matches('/'))
        };

        // List objects with delimiter to get directory-like structure
        // This is a READ-ONLY operation
        let mut response = self
            .client
            .list_objects_v2()
            .bucket(&self.bucket)
            .prefix(&prefix)
            .delimiter("/")
            .into_paginator()
            .send();

        let mut entries = Vec::new();
        let mut seen_dirs = HashSet::new();

        // Paginate through all results
        while let Some(result) = response.next().await {
            let output = result.context("Failed to list S3 objects")?;

            // Add directories (common prefixes)
            for common_prefix in output.common_prefixes() {
                if let Some(p) = common_prefix.prefix() {
                    let name = p
                        .strip_prefix(&prefix)
                        .unwrap_or(p)
                        .trim_end_matches('/')
                        .to_string();

                    if !name.is_empty() && seen_dirs.insert(name.clone()) {
                        entries.push(Entry {
                            name,
                            is_dir: true,
                            size: None,
                            modified: None,
                        });
                    }
                }
            }

            // Add files
            for object in output.contents() {
                let key = object.key().unwrap_or("");

                // Skip if this is the prefix itself
                if key == prefix {
                    continue;
                }

                let name = key
                    .strip_prefix(&prefix)
                    .unwrap_or(key)
                    .to_string();

                // Skip if this looks like a directory marker
                if name.ends_with('/') {
                    continue;
                }

                entries.push(Entry {
                    name,
                    is_dir: false,
                    size: object.size().map(|s| s as u64),
                    modified: object.last_modified().map(|t| {
                        let secs = t.secs();
                        chrono::DateTime::from_timestamp(secs, 0)
                            .map(|dt| dt.format("%Y-%m-%d %H:%M:%S").to_string())
                            .unwrap_or_default()
                    }),
                });
            }
        }

        // Sort: directories first, then by name (same as LocalBackend)
        entries.sort_by(|a, b| match (a.is_dir, b.is_dir) {
            (true, false) => std::cmp::Ordering::Less,
            (false, true) => std::cmp::Ordering::Greater,
            _ => a.name.cmp(&b.name),
        });

        Ok(ListResult {
            entries,
            prefix: prefix.trim_end_matches('/').to_string(),
        })
    }

    /// Get preview of an S3 object (READ-ONLY operation)
    /// Uses GetObject which is a read-only S3 operation
    /// Get preview of an S3 object (READ-ONLY operation)
    /// Uses GetObject which is a read-only S3 operation
    async fn get_preview(&self, path: &str, max_size: usize) -> Result<PreviewContent> {
        let key = path.trim_start_matches('/');

        // First, check object size with HeadObject (READ-ONLY operation)
        let head_result = self
            .client
            .head_object()
            .bucket(&self.bucket)
            .key(key)
            .send()
            .await;

        match head_result {
            Ok(head) => {
                let size = head.content_length().unwrap_or(0) as u64;

                // Extract object metadata from the HEAD response
                let modified = head.last_modified().and_then(|t| {
                    chrono::DateTime::from_timestamp(t.secs(), 0)
                        .map(|dt| dt.format("%Y-%m-%d %H:%M:%S UTC").to_string())
                });
                let etag = head.e_tag().map(|s| s.trim_matches('"').to_string());
                let storage_class = head
                    .storage_class()
                    .map(|sc| sc.as_str().to_string());
                let version_id = head.version_id().map(|s| s.to_string());

                // Resolve the 1-based ordinal for this version (oldest = 1, newest = N).
                // list_object_versions returns versions newest-first; the current version
                // sits at some index i, so its ordinal is total - i.
                let version_number: Option<usize> = if let Some(ref vid) = version_id {
                    match self
                        .client
                        .list_object_versions()
                        .bucket(&self.bucket)
                        .prefix(key)
                        .send()
                        .await
                    {
                        Ok(output) => {
                            let versions: Vec<_> = output
                                .versions()
                                .iter()
                                .filter(|v| v.key() == Some(key))
                                .collect();
                            let total = versions.len();
                            versions
                                .iter()
                                .position(|v| v.version_id() == Some(vid.as_str()))
                                .map(|i| total - i)
                        }
                        Err(_) => None,
                    }
                } else {
                    None
                };

                if size > max_size as u64 {
                    return Ok(PreviewContent::TooLarge { size, modified, etag, storage_class, version_id, version_number });
                }

                // Get the object content (READ-ONLY operation)
                let response = self
                    .client
                    .get_object()
                    .bucket(&self.bucket)
                    .key(key)
                    .send()
                    .await
                    .context("Failed to get S3 object")?;

                // Get content type before consuming the body
                let mime_type = response.content_type().map(|s| s.to_string());

                // Read the content
                let bytes = response
                    .body
                    .collect()
                    .await
                    .context("Failed to read S3 object body")?
                    .into_bytes();

                // Try to convert to text
                match String::from_utf8(bytes.to_vec()) {
                    Ok(content) => Ok(PreviewContent::Text(content, super::FileMetadata {
                        size: Some(size),
                        modified: modified.clone(),
                        etag: etag.clone(),
                        storage_class: storage_class.clone(),
                        version_id: version_id.clone(),
                        version_number,
                    })),
                    Err(_) => {
                        Ok(PreviewContent::Binary { size, mime_type, modified, etag, storage_class, version_id, version_number })
                    }
                }
            }
            Err(e) => Ok(PreviewContent::Error(format!(
                "Failed to access object: {}",
                e
            ))),
        }
    }

    /// Download a single file from S3 (READ-ONLY operation).
    /// For files >= 16 MB, uses parallel range requests for higher throughput.
    async fn download_file(
        &self,
        path: &str,
        destination: &Path,
        progress_callback: Option<crate::backend::ProgressCallback>,
    ) -> Result<()> {
        use tokio::io::AsyncWriteExt;

        let key = path.trim_start_matches('/');

        // Get the object (READ-ONLY operation); inspect content-length before reading body
        let response = self
            .client
            .get_object()
            .bucket(&self.bucket)
            .key(key)
            .send()
            .await
            .context("Failed to get S3 object")?;

        let total_size = response.content_length().map(|s| s as u64);

        // For large files, drop this response and use parallel range requests instead
        const MULTIPART_THRESHOLD: u64 = 16 * 1024 * 1024; // 16 MB
        if let Some(size) = total_size {
            if size >= MULTIPART_THRESHOLD {
                drop(response);
                return self.download_multipart(key, destination, size, progress_callback).await;
            }
        }

        // Small file: stream the already-open response directly
        let mut file = tokio::fs::File::create(destination)
            .await
            .context("Failed to create destination file")?;

        let mut body = response.body;
        let mut downloaded = 0u64;

        while let Some(chunk) = body.try_next().await.context("Failed to read S3 object body")? {
            file.write_all(&chunk)
                .await
                .context("Failed to write to destination file")?;

            downloaded += chunk.len() as u64;

            if let Some(ref callback) = progress_callback {
                callback(downloaded, total_size);
            }
        }

        Ok(())
    }

    fn location_name(&self) -> String {
        format!("s3://{}", self.bucket)
    }

    fn get_display_path(&self, prefix: &str) -> String {
        format!("s3://{}/{}", self.bucket, prefix)
    }

    fn uri_to_prefix(&self, uri: &str) -> Option<String> {
        let bucket_prefix = format!("s3://{}/", self.bucket);
        if let Some(prefix) = uri.strip_prefix(&bucket_prefix) {
            Some(prefix.to_string())
        } else if uri == format!("s3://{}", self.bucket) {
            Some(String::new())
        } else {
            None
        }
    }

    fn get_parent(&self, prefix: &str) -> Option<String> {
        let prefix = prefix.trim_end_matches('/');
        if prefix.is_empty() {
            return None;
        }

        let parts: Vec<&str> = prefix.split('/').collect();
        if parts.len() == 1 {
            Some(String::new())
        } else {
            Some(parts[..parts.len() - 1].join("/"))
        }
    }
}
