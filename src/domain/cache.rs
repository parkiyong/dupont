use std::io::Cursor;
use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};

use image::{ImageFormat, ImageReader, Limits};

use crate::domain::entities::Wallpaper;
use crate::domain::errors::CacheError;
use crate::domain::value_objects::CacheConfig;

/// Cache entry metadata
#[derive(Debug, Clone)]
struct CacheEntry {
    id: String,
    path: PathBuf,
    size_bytes: u64,
    accessed_at: u64,
    created_at: u64,
}

/// Image cache manager with LRU eviction
///
/// Stores downloaded wallpapers in `~/.cache/dupont/` with configurable
/// size, count, and age limits. Enforces LRU eviction when limits are exceeded.
pub struct Cache {
    cache_dir: PathBuf,
    config: CacheConfig,
    entries: Vec<CacheEntry>,
}

impl Cache {
    /// Create a new cache manager with the given configuration
    ///
    /// Creates cache directory at `~/.cache/dupont/` if it doesn't exist.
    pub fn new(config: CacheConfig) -> Result<Self, CacheError> {
        let cache_dir = dirs::cache_dir()
            .ok_or(CacheError::NotAccessible)?
            .join("dupont");

        std::fs::create_dir_all(&cache_dir).map_err(CacheError::IoError)?;

        Ok(Self {
            cache_dir,
            config,
            entries: Vec::new(),
        })
    }

    /// Create a new cache manager with default configuration
    ///
    /// Defaults: 500MB max size, 50 max images, 30 days max age.
    pub fn with_defaults() -> Result<Self, CacheError> {
        Self::new(CacheConfig::default())
    }

    /// Load existing cache entries from disk
    ///
    /// Scans the cache directory and builds an in-memory index of all
    /// cached files with their metadata (size, timestamps).
    pub async fn load_entries(&mut self) -> Result<(), CacheError> {
        self.entries.clear();

        let mut read_dir = tokio::fs::read_dir(&self.cache_dir)
            .await
            .map_err(CacheError::IoError)?;

        while let Some(entry) = read_dir.next_entry().await.map_err(CacheError::IoError)? {
            let path = entry.path();

            // Only process files (not directories)
            if path.is_file() {
                let metadata = tokio::fs::metadata(&path)
                    .await
                    .map_err(CacheError::IoError)?;

                let modified = metadata.modified().map_err(CacheError::IoError)?;
                // created() may fail on Linux filesystems — fall back to modified time
                let created = metadata.created().unwrap_or(modified);

                let size_bytes = metadata.len();

                // Extract ID from filename (remove extension)
                let id = path
                    .file_stem()
                    .and_then(|s| s.to_str())
                    .ok_or(CacheError::NotAccessible)?
                    .to_string();

                self.entries.push(CacheEntry {
                    id,
                    path,
                    size_bytes,
                    accessed_at: to_unix_secs(modified)?,
                    created_at: to_unix_secs(created)?,
                });
            }
        }

        Ok(())
    }

    /// Get or download a wallpaper
    ///
    /// Returns local file path if already cached, downloads and caches otherwise.
    /// Automatically loads cache entries on first call if not yet loaded.
    pub async fn get_or_download(&mut self, wallpaper: &Wallpaper) -> Result<PathBuf, CacheError> {
        // Load entries if not already loaded
        if self.entries.is_empty() {
            self.load_entries().await?;
        }

        // Check if already cached — clone path before mutable borrow
        if self.find_entry(&wallpaper.id).is_some() {
            let path = self
                .find_entry(&wallpaper.id)
                .expect("entry exists")
                .path
                .clone();
            // Update access time (LRU tracking)
            self.update_access_time(&wallpaper.id);
            return Ok(path);
        }

        // Download and cache
        self.download_and_cache(wallpaper).await
    }

    /// Find a cache entry by ID
    fn find_entry(&self, id: &str) -> Option<&CacheEntry> {
        self.entries.iter().find(|e| e.id == id)
    }

    /// Update access time for an entry (LRU tracking)
    fn update_access_time(&mut self, id: &str) {
        if let Some(entry) = self.entries.iter_mut().find(|e| e.id == id) {
            entry.accessed_at = now_secs();
        }
    }

    /// Download a wallpaper image and cache it locally
    async fn download_and_cache(&mut self, wallpaper: &Wallpaper) -> Result<PathBuf, CacheError> {
        // Download image bytes
        let client = reqwest::Client::new();
        let response = client.get(&wallpaper.url).send().await.map_err(|e| {
            CacheError::IoError(std::io::Error::other(format!("HTTP request failed: {}", e)))
        })?;

        if !response.status().is_success() {
            return Err(CacheError::IoError(std::io::Error::other(format!(
                "HTTP error: {}",
                response.status()
            ))));
        }

        let image_bytes = response.bytes().await.map_err(|e| {
            CacheError::IoError(std::io::Error::other(format!(
                "Failed to download image: {}",
                e
            )))
        })?;

        // Validate image format and detect file extension in a single pass
        let extension = self.validate_and_detect(image_bytes.as_ref())?;

        // Generate cache file path
        let cache_path = self
            .cache_dir
            .join(format!("{}.{}", wallpaper.id, extension));

        // Write image bytes to cache
        tokio::fs::write(&cache_path, &image_bytes)
            .await
            .map_err(CacheError::IoError)?;

        // Get file metadata for size and timestamps
        let metadata = tokio::fs::metadata(&cache_path)
            .await
            .map_err(CacheError::IoError)?;

        let created = metadata.created().unwrap_or(SystemTime::now());
        let accessed = metadata.modified().unwrap_or(SystemTime::now());

        // Create cache entry
        self.entries.push(CacheEntry {
            id: wallpaper.id.clone(),
            path: cache_path.clone(),
            size_bytes: metadata.len(),
            accessed_at: to_unix_secs(accessed)?,
            created_at: to_unix_secs(created)?,
        });

        // Evict if over limits
        self.evict_if_needed().await?;

        Ok(cache_path)
    }

    /// Validate image format with OOM protection and detect the file extension
    ///
    /// Uses `image::ImageReader` with custom limits to prevent memory exhaustion
    /// from large or corrupted images. Returns the appropriate file extension
    /// (jpg, png, webp) based on detected format.
    ///
    /// Supported formats: JPEG, PNG, WebP. HEIC is not supported in MVP.
    fn validate_and_detect(&self, image_bytes: &[u8]) -> Result<&'static str, CacheError> {
        // Set limits to prevent OOM on large or corrupted images
        let mut limits = Limits::default();
        limits.max_image_width = Some(8192);
        limits.max_image_height = Some(8192);
        limits.max_alloc = Some(50 * 1024 * 1024); // 50MB

        let cursor = Cursor::new(image_bytes);
        let mut reader = ImageReader::new(cursor)
            .with_guessed_format()
            .map_err(image::ImageError::IoError)?;
        reader.limits(limits);

        // Get format before decoding (decode consumes self)
        let format = reader.format();

        // Decode to validate the image is not corrupted
        reader.decode().map_err(CacheError::ImageError)?;

        // Determine file extension from detected format
        match format {
            Some(ImageFormat::Jpeg) => Ok("jpg"),
            Some(ImageFormat::Png) => Ok("png"),
            Some(ImageFormat::WebP) => Ok("webp"),
            _ => Err(CacheError::ImageError(image::ImageError::IoError(
                std::io::Error::new(
                    std::io::ErrorKind::InvalidData,
                    "Unsupported image format (HEIC not supported in MVP)",
                ),
            ))),
        }
    }

    /// Evict cache entries if over configured limits
    ///
    /// First removes entries older than `max_age_seconds`, then evicts
    /// least recently accessed (LRU) entries until within size and count limits.
    async fn evict_if_needed(&mut self) -> Result<(), CacheError> {
        let total_size: u64 = self.entries.iter().map(|e| e.size_bytes).sum();
        let count = self.entries.len();

        let needs_eviction =
            total_size > self.config.max_size_bytes || count > self.config.max_count;
        if !needs_eviction {
            return Ok(());
        }

        // First, remove entries older than max_age_seconds
        let now = now_secs();
        let old_entries: Vec<String> = self
            .entries
            .iter()
            .filter(|e| now - e.created_at > self.config.max_age_seconds)
            .map(|e| e.id.clone())
            .collect();

        for entry_id in old_entries {
            self.remove_entry(&entry_id).await?;
        }

        // If still over limits, remove least recently accessed (LRU)
        let total_size: u64 = self.entries.iter().map(|e| e.size_bytes).sum();
        let count = self.entries.len();
        if total_size > self.config.max_size_bytes || count > self.config.max_count {
            self.evict_lru_until_within_limits().await?;
        }

        Ok(())
    }

    /// Remove least recently accessed entries until within limits
    async fn evict_lru_until_within_limits(&mut self) -> Result<(), CacheError> {
        loop {
            let total_size: u64 = self.entries.iter().map(|e| e.size_bytes).sum();
            let count = self.entries.len();

            if total_size <= self.config.max_size_bytes && count <= self.config.max_count {
                break;
            }

            // Find and remove least recently accessed entry
            // Clone the ID to avoid borrow conflict with mutable remove_entry
            let lru_id = self
                .entries
                .iter()
                .min_by_key(|e| e.accessed_at)
                .map(|e| e.id.clone());

            if let Some(id) = lru_id {
                self.remove_entry(&id).await?;
            } else {
                break;
            }
        }

        Ok(())
    }

    /// Remove a cache entry by ID, deleting its file from disk
    async fn remove_entry(&mut self, id: &str) -> Result<(), CacheError> {
        if let Some(pos) = self.entries.iter().position(|e| e.id == id) {
            let entry = self.entries.remove(pos);
            tokio::fs::remove_file(&entry.path)
                .await
                .map_err(CacheError::IoError)?;
        }
        Ok(())
    }
}

/// Convert a `SystemTime` to Unix timestamp in seconds
fn to_unix_secs(time: SystemTime) -> Result<u64, CacheError> {
    Ok(time
        .duration_since(UNIX_EPOCH)
        .map_err(std::io::Error::other)?
        .as_secs())
}

/// Get the current Unix timestamp in seconds
fn now_secs() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs()
}
