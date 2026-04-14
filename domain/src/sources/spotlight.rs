use async_trait::async_trait;
use reqwest::Client;
use serde::Deserialize;
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};
use std::time::Duration;

use crate::error::SourceError;
use crate::source::Source;
use crate::wallpaper::Wallpaper;

/// Microsoft Spotlight API response structure (deeply nested)
#[derive(Debug, Deserialize)]
struct SpotlightResponse {
    batchrsp: SpotlightBatch,
}

#[derive(Debug, Deserialize)]
struct SpotlightBatch {
    items: Vec<SpotlightItem>,
}

#[derive(Debug, Deserialize)]
struct SpotlightItem {
    item: Vec<SpotlightItemData>,
}

#[derive(Debug, Deserialize)]
struct SpotlightItemData {
    #[serde(rename = "ad")]
    ad: SpotlightAd,
}

#[derive(Debug, Deserialize)]
struct SpotlightAd {
    #[serde(rename = "imageFullscreen001")]
    image: SpotlightImage,
}

#[derive(Debug, Deserialize)]
#[allow(dead_code)]
struct SpotlightImage {
    #[serde(rename = "u")]
    url: String,
    #[serde(rename = "t")]
    title: String,
    #[serde(rename = "desc")]
    description: String,
    #[serde(rename = "l")]
    attribution_url: String,
}

/// Microsoft Spotlight source
///
/// Fetches wallpapers from Microsoft's Spotlight delivery API.
pub struct SpotlightSource {
    client: Client,
    locale: String,
    resolution: (u32, u32),
    base_url: &'static str,
}

impl SpotlightSource {
    /// Create a new SpotlightSource with default settings (en-US, 1920x1080)
    pub fn new() -> Self {
        Self::with_locale("80217".to_string()) // 80217 = en-US
    }

    /// Create a new SpotlightSource with specified locale code
    pub fn with_locale(locale: String) -> Self {
        Self {
            client: Client::builder()
                .timeout(Duration::from_secs(10))
                .build()
                .expect("Failed to create HTTP client"),
            locale,
            resolution: (1920, 1080), // Default Full HD resolution
            base_url: "https://arc.msn.com/v3/Delivery/Cache",
        }
    }

    /// Set resolution for fetched wallpapers (builder pattern)
    pub fn with_resolution(mut self, width: u32, height: u32) -> Self {
        self.resolution = (width, height);
        self
    }
}

impl Default for SpotlightSource {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl Source for SpotlightSource {
    async fn fetch(&self) -> Result<Wallpaper, SourceError> {
        // Build API URL with parameters
        let url = format!(
            "{}?pid=209567&fmt=json&rafb=0&ua=WindowsShellClient%2F0&disphorzres={}&dispvertres={}&lo={}",
            self.base_url, self.resolution.0, self.resolution.1, self.locale
        );

        // Fetch from Spotlight API with retry for rate limiting
        let response = self.fetch_with_retry(&url, 3).await?;

        // Parse JSON response
        let spotlight_response: SpotlightResponse = response
            .json()
            .await
            .map_err(|e| {
                SourceError::ParseError(format!("Failed to parse Spotlight API response: {}", e))
            })?;

        // Extract image from deeply nested structure
        let image = spotlight_response
            .batchrsp
            .items
            .into_iter()
            .filter_map(|item| item.item.into_iter().next())
            .map(|data| data.ad.image)
            .next()
            .ok_or(SourceError::NoWallpaperFound)?;

        // Create ID from URL hash
        let id = format!("spotlight-{}", hash_url(&image.url));

        // Extract attribution from description (format: "Description (© Copyright)")
        let attribution = extract_attribution(&image.description);

        Ok(Wallpaper::new(
            id,
            image.url,
            image.title,
            image.description,
            attribution,
            "spotlight".to_string(),
        ))
    }

    fn id(&self) -> &'static str {
        "spotlight"
    }

    fn name(&self) -> &'static str {
        "Microsoft Spotlight"
    }
}

impl SpotlightSource {
    /// Fetch a URL with exponential backoff retry on rate limiting (HTTP 429).
    async fn fetch_with_retry(
        &self,
        url: &str,
        max_retries: u32,
    ) -> Result<reqwest::Response, SourceError> {
        let mut retries = 0;

        loop {
            let response = self.client.get(url).send().await?;

            let status = response.status();

            // Check for rate limiting
            if status.as_u16() == 429 {
                retries += 1;
                if retries >= max_retries {
                    return Err(SourceError::RateLimited {
                        source_name: "Spotlight".to_string(),
                    });
                }

                // Exponential backoff: 2s, 4s, 8s
                let backoff = Duration::from_secs(2u64.pow(retries));
                tokio::time::sleep(backoff).await;
                continue;
            }

            // Return response for non-rate-limit status codes
            if !status.is_success() {
                return Err(SourceError::Unavailable {
                    source_name: format!("Spotlight API returned status {}", status),
                });
            }

            return Ok(response);
        }
    }
}

/// Simple hash function to generate a short ID from a URL.
fn hash_url(url: &str) -> String {
    let mut hasher = DefaultHasher::new();
    url.hash(&mut hasher);
    format!("{:x}", hasher.finish())
}

/// Extract attribution from description string.
///
/// Expected format: "Description text (© Copyright Holder)"
fn extract_attribution(description: &str) -> String {
    description
        .split('©')
        .nth(1)
        .and_then(|s| s.split(')').next())
        .map(|s| s.trim().to_string())
        .unwrap_or_else(|| "Microsoft".to_string())
}
