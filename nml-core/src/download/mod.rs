//! Download engine for NML

use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::Duration;

use futures::future::join_all;
use reqwest::{Client, Response, StatusCode};
use tokio::fs;
use tokio::io::AsyncWriteExt;
use tokio::sync::Semaphore;
use tokio::time::timeout;

use crate::error::{NMLError, Result};

pub mod mirror;

use mirror::{Mirror, MirrorStatus};

/// Download task
#[derive(Debug, Clone)]
pub struct DownloadTask {
    /// Source URL
    pub url: String,
    /// Destination path
    pub dest: PathBuf,
    /// Expected SHA1
    pub sha1: Option<String>,
    /// Expected size
    pub size: Option<u64>,
    /// Priority (higher = more important)
    pub priority: i32,
}

/// Download engine
pub struct DownloadEngine {
    client: Client,
    mirrors: Vec<Mirror>,
    max_concurrent: usize,
    chunk_size: usize,
    timeout: Duration,
}

impl DownloadEngine {
    /// Create a new download engine
    pub fn new() -> Self {
        Self {
            client: Client::new(),
            mirrors: Self::default_mirrors(),
            max_concurrent: 8,
            chunk_size: 1024 * 1024, // 1MB
            timeout: Duration::from_secs(30),
        }
    }

    /// Create with custom config
    pub fn with_config(max_concurrent: usize, mirrors: Vec<Mirror>) -> Self {
        Self {
            client: Client::new(),
            mirrors,
            max_concurrent,
            chunk_size: 1024 * 1024,
            timeout: Duration::from_secs(30),
        }
    }

    /// Test all mirrors and sort by latency
    pub async fn test_mirrors(&mut self) -> Result<()> {
        let mut results = Vec::new();

        for mirror in &self.mirrors {
            let start = std::time::Instant::now();
            let test_url = mirror.base_url.clone() + "/mc/game/version_manifest.json";
            
            let status = match timeout(Duration::from_secs(5), 
                self.client.head(&test_url).send()).await {
                Ok(Ok(response)) if response.status().is_success() => {
                    let latency = start.elapsed().as_millis() as u64;
                    MirrorStatus::Healthy { latency }
                }
                _ => MirrorStatus::Unhealthy,
            };

            results.push((mirror.clone(), status));
        }

        // Sort by latency, unhealthy at the end
        results.sort_by(|a, b| {
            match (&a.1, &b.1) {
                (MirrorStatus::Healthy { latency: a_lat }, MirrorStatus::Healthy { latency: b_lat }) => {
                    a_lat.cmp(b_lat)
                }
                (MirrorStatus::Healthy { .. }, MirrorStatus::Unhealthy) => std::cmp::Ordering::Less,
                (MirrorStatus::Unhealthy, MirrorStatus::Healthy { .. }) => std::cmp::Ordering::Greater,
                (MirrorStatus::Unhealthy, MirrorStatus::Unhealthy) => std::cmp::Ordering::Equal,
            }
        });

        self.mirrors = results.into_iter().map(|(m, _)| m).collect();
        Ok(())
    }

    /// Download a single file
    pub async fn download<F>(&self, task: &DownloadTask, mut progress: F) -> Result<()>
    where
        F: FnMut(f32),
    {
        // Create parent directory
        if let Some(parent) = task.dest.parent() {
            fs::create_dir_all(parent).await?;
        }

        // Try mirrors in order
        let urls = self.get_mirror_urls(&task.url);

        for (idx, url) in urls.iter().enumerate() {
            match self.download_from_url(url, &task.dest, task.size, &mut progress).await {
                Ok(()) => {
                    // Verify SHA1 if provided
                    if let Some(expected_sha1) = &task.sha1 {
                        let actual_sha1 = self.calculate_sha1(&task.dest).await?;
                        if actual_sha1 != *expected_sha1 {
                            tracing::warn!("SHA1 mismatch for {}, retrying...", task.dest.display());
                            continue;
                        }
                    }
                    return Ok(());
                }
                Err(e) => {
                    tracing::warn!("Download from {} failed: {}", url, e);
                    if idx == urls.len() - 1 {
                        return Err(e);
                    }
                }
            }
        }

        Err(NMLError::DownloadFailed(format!(
            "All mirrors failed for {}",
            task.dest.display()
        )))
    }

    /// Download multiple files in parallel
    pub async fn download_batch<F>(
        &self,
        tasks: Vec<DownloadTask>,
        mut progress: F,
    ) -> Result<Vec<Result<()>>>
    where
        F: FnMut(usize, usize, f32),
    {
        let semaphore = Arc::new(Semaphore::new(self.max_concurrent));
        let total = tasks.len();
        let completed = Arc::new(std::sync::atomic::AtomicUsize::new(0));

        let futures: Vec<_> = tasks
            .into_iter()
            .enumerate()
            .map(|(idx, task)| {
                let semaphore = semaphore.clone();
                let completed = completed.clone();
                
                async move {
                    let _permit = semaphore.acquire().await.unwrap();
                    
                    let result = self
                        .download(&task, |p| {
                            let c = completed.load(std::sync::atomic::Ordering::Relaxed);
                            let overall = (c as f32 + p) / total as f32;
                            progress(idx, total, overall);
                        })
                        .await;

                    completed.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
                    result
                }
            })
            .collect();

        let results = join_all(futures).await;
        Ok(results)
    }

    /// Download with chunked parallel (for large files)
    pub async fn download_chunked<F>(
        &self,
        task: &DownloadTask,
        num_chunks: usize,
        progress: F,
    ) -> Result<()>
    where
        F: FnMut(f32),
    {
        // Get file size
        let file_size = match self.get_file_size(&task.url).await {
            Ok(size) => size,
            Err(_) => {
                // Fall back to regular download
                return self.download(task, progress).await;
            }
        };

        if file_size < 10 * 1024 * 1024 {
            // Files smaller than 10MB don't need chunked download
            return self.download(task, progress).await;
        }

        // Calculate chunks
        let chunk_size = file_size / num_chunks as u64;
        let temp_dir = task.dest.parent().unwrap_or(Path::new(".")).join(".tmp");
        fs::create_dir_all(&temp_dir).await?;

        let mut chunk_files = Vec::new();
        let mut futures = Vec::new();

        for i in 0..num_chunks {
            let start = i as u64 * chunk_size;
            let end = if i == num_chunks - 1 {
                file_size - 1
            } else {
                (i as u64 + 1) * chunk_size - 1
            };

            let temp_file = temp_dir.join(format!("chunk_{}", i));
            chunk_files.push(temp_file.clone());

            let url = task.url.clone();
            let client = self.client.clone();

            futures.push(async move {
                Self::download_chunk(&client, &url, &temp_file, start, end).await
            });
        }

        // Download all chunks
        let results: Vec<_> = join_all(futures).await;

        // Check for errors
        for result in results {
            result?;
        }

        // Merge chunks
        self.merge_chunks(&chunk_files, &task.dest).await?;

        // Clean up temp files
        for chunk_file in &chunk_files {
            let _ = fs::remove_file(chunk_file).await;
        }
        let _ = fs::remove_dir(&temp_dir).await;

        Ok(())
    }

    // Private helpers

    async fn download_from_url<F>(
        &self,
        url: &str,
        dest: &Path,
        expected_size: Option<u64>,
        progress: &mut F,
    ) -> Result<()>
    where
        F: FnMut(f32),
    {
        let response = self
            .client
            .get(url)
            .timeout(self.timeout)
            .send()
            .await?;

        if !response.status().is_success() {
            return Err(NMLError::DownloadFailed(format!(
                "HTTP {}",
                response.status()
            )));
        }

        let total_size = expected_size.or_else(|| response.content_length()).unwrap_or(0);
        let mut file = fs::File::create(dest).await?;
        let mut stream = response.bytes_stream();
        let mut downloaded = 0u64;

        while let Some(chunk) = stream.next().await {
            let chunk = chunk?;
            file.write_all(&chunk).await?;
            downloaded += chunk.len() as u64;

            if total_size > 0 {
                progress(downloaded as f32 / total_size as f32);
            }
        }

        file.flush().await?;
        progress(1.0);

        Ok(())
    }

    async fn download_chunk(
        client: &Client,
        url: &str,
        dest: &Path,
        start: u64,
        end: u64,
    ) -> Result<()> {
        let response = client
            .get(url)
            .header("Range", format!("bytes={}-{}", start, end))
            .send()
            .await?;

        if !response.status().is_success() && response.status() != StatusCode::PARTIAL_CONTENT {
            return Err(NMLError::DownloadFailed(format!(
                "HTTP {}",
                response.status()
            )));
        }

        let content = response.bytes().await?;
        fs::write(dest, content).await?;

        Ok(())
    }

    async fn merge_chunks(&self, chunk_files: &[PathBuf], dest: &Path) -> Result<()> {
        let mut file = fs::File::create(dest).await?;

        for chunk_file in chunk_files {
            let content = fs::read(chunk_file).await?;
            file.write_all(&content).await?;
        }

        file.flush().await?;
        Ok(())
    }

    fn get_mirror_urls(&self, original_url: &str) -> Vec<String> {
        let mut urls = Vec::new();

        // Add original URL first
        urls.push(original_url.to_string());

        // Add mirrored URLs
        for mirror in &self.mirrors {
            if let Some(mirrored) = mirror.mirror_url(original_url) {
                if !urls.contains(&mirrored) {
                    urls.push(mirrored);
                }
            }
        }

        urls
    }

    async fn get_file_size(&self, url: &str) -> Result<u64> {
        let response = self.client.head(url).send().await?;
        
        if !response.status().is_success() {
            return Err(NMLError::DownloadFailed(format!(
                "Failed to get file size: HTTP {}",
                response.status()
            )));
        }

        response
            .content_length()
            .ok_or_else(|| NMLError::DownloadFailed("No content length".to_string()))
    }

    async fn calculate_sha1(&self, path: &Path) -> Result<String> {
        use sha1::{Digest, Sha1};

        let content = fs::read(path).await?;
        let hash = Sha1::digest(&content);
        Ok(format!("{:x}", hash))
    }

    fn default_mirrors() -> Vec<Mirror> {
        vec![
            Mirror {
                name: "BMCLAPI".to_string(),
                base_url: "https://bmclapi2.bangbang93.com".to_string(),
                mappings: vec![
                    ("https://piston-meta.mojang.com".to_string(), "".to_string()),
                    ("https://piston-data.mojang.com".to_string(), "".to_string()),
                    ("https://launchermeta.mojang.com".to_string(), "".to_string()),
                    ("https://launcher.mojang.com".to_string(), "".to_string()),
                    ("https://libraries.minecraft.net".to_string(), "/maven".to_string()),
                    ("https://resources.download.minecraft.net".to_string(), "/objects".to_string()),
                ],
            },
            Mirror {
                name: "MCBBS".to_string(),
                base_url: "https://download.mcbbs.net".to_string(),
                mappings: vec![
                    ("https://piston-meta.mojang.com".to_string(), "".to_string()),
                    ("https://piston-data.mojang.com".to_string(), "".to_string()),
                    ("https://launchermeta.mojang.com".to_string(), "".to_string()),
                    ("https://launcher.mojang.com".to_string(), "".to_string()),
                    ("https://libraries.minecraft.net".to_string(), "/maven".to_string()),
                    ("https://resources.download.minecraft.net".to_string(), "/objects".to_string()),
                ],
            },
        ]
    }
}

impl Default for DownloadEngine {
    fn default() -> Self {
        Self::new()
    }
}
