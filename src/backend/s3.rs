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

                if size > max_size as u64 {
                    return Ok(PreviewContent::TooLarge { size });
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
                    Ok(content) => Ok(PreviewContent::Text(content)),
                    Err(_) => {
                        // Binary content
                        Ok(PreviewContent::Binary { size, mime_type })
                    }
                }
            }
            Err(e) => Ok(PreviewContent::Error(format!(
                "Failed to access object: {}",
                e
            ))),
        }
    }

    /// Download a single file from S3 (READ-ONLY operation)
    /// Uses GetObject which is a read-only S3 operation
    async fn download_file(
        &self,
        path: &str,
        destination: &Path,
        progress_callback: Option<crate::backend::ProgressCallback>,
    ) -> Result<()> {
        use tokio::io::AsyncWriteExt;

        let key = path.trim_start_matches('/');

        // Get the object (READ-ONLY operation)
        let response = self
            .client
            .get_object()
            .bucket(&self.bucket)
            .key(key)
            .send()
            .await
            .context("Failed to get S3 object")?;

        let total_size = response.content_length().map(|s| s as u64);

        // Create destination file
        let mut file = tokio::fs::File::create(destination)
            .await
            .context("Failed to create destination file")?;

        // Stream the content to file with progress reporting
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

    fn get_display_path(&self, prefix: &str) -> String {
        format!("s3://{}/{}", self.bucket, prefix)
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
