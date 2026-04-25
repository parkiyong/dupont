use async_trait::async_trait;
use reqwest::Client;
use serde::Deserialize;
use serde_json;
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};
use std::time::Duration;

use crate::domain::entities::Wallpaper;
use crate::domain::errors::SourceError;
use crate::domain::traits::Source;

/// Microsoft Spotlight API (fd.api.iris.microsoft.com) response
#[derive(Debug, Deserialize)]
struct SpotlightResponse {
    batchrsp: SpotlightBatch,
}

#[derive(Debug, Deserialize)]
struct SpotlightBatch {
    #[serde(default)]
    errors: Vec<SpotlightError>,
    #[serde(default)]
    items: Vec<SpotlightItem>,
}

#[derive(Debug, Deserialize)]
struct SpotlightError {
    msg: String,
}

/// items[0].item is a JSON string, not a nested object
#[derive(Debug, Deserialize)]
struct SpotlightItem {
    item: String,
}

/// Wrapper for the inner JSON string (items[0].item)
#[derive(Debug, Deserialize)]
struct SpotlightInner {
    ad: SpotlightAd,
}

/// Inner ad structure (nested under "ad" in the inner JSON)
#[derive(Debug, Deserialize)]
struct SpotlightAd {
    #[serde(rename = "landscapeImage", default)]
    landscape_image: Option<SpotlightImage>,
    #[serde(default)]
    title: Option<String>,
    #[serde(rename = "iconHoverText", default)]
    icon_hover_text: Option<String>,
    #[serde(default)]
    copyright: Option<String>,
}

#[derive(Debug, Deserialize)]
struct SpotlightImage {
    asset: String,
}

/// Microsoft Spotlight source
///
/// Fetches wallpapers from Microsoft's Spotlight delivery API (fd.api.iris.microsoft.com).
pub struct SpotlightSource {
    client: Client,
    locale: String,
}

impl SpotlightSource {
    /// Create a new SpotlightSource with default settings (en-US)
    pub fn new() -> Self {
        Self::with_locale("en-US".to_string())
    }

    /// Create a new SpotlightSource with specified locale (e.g. "en-US", "ja-JP")
    pub fn with_locale(locale: String) -> Self {
        Self {
            client: Client::builder()
                .timeout(Duration::from_secs(10))
                .build()
                .expect("Failed to create HTTP client"),
            locale,
        }
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
        // Parse locale to get country code (e.g. "en-US" -> "US")
        let country = self
            .locale
            .split('-')
            .nth(1)
            .unwrap_or(&self.locale)
            .to_uppercase();

        let url = format!(
            "https://fd.api.iris.microsoft.com/v4/api/selection?placement=88000820&bcnt=1&country={}&locale={}&fmt=json",
            country, self.locale
        );

        let response = self.fetch_with_retry(&url, 3).await?;

        let body = response.text().await.map_err(|e| {
            SourceError::ParseError(format!("Failed to read Spotlight response body: {}", e))
        })?;

        // Check for API error responses
        let outer: SpotlightResponse = serde_json::from_str(&body).map_err(|e| {
            SourceError::ParseError(format!("Failed to parse Spotlight API response: {}", e))
        })?;

        if let Some(err) = outer.batchrsp.errors.first() {
            return Err(SourceError::Unavailable {
                source_name: format!("Spotlight: {}", err.msg),
            });
        }

        // Extract the inner JSON string from items[0].item
        let item_json = outer
            .batchrsp
            .items
            .into_iter()
            .next()
            .ok_or(SourceError::NoWallpaperFound)?
            .item;

        // Parse the inner JSON (contains "f", "v", "rdr", "ad" fields)
        let inner: SpotlightInner = serde_json::from_str(&item_json).map_err(|e| {
            SourceError::ParseError(format!("Failed to parse Spotlight ad data: {}", e))
        })?;
        let ad = inner.ad;

        let image = ad.landscape_image.ok_or(SourceError::NoWallpaperFound)?;
        let image_url = image.asset;

        let title = ad.title.unwrap_or_default();
        let description = ad
            .icon_hover_text
            .as_ref()
            .and_then(|text| text.split('\n').next())
            .map(|s| s.trim().to_string())
            .unwrap_or_default();
        let attribution = ad.copyright.unwrap_or_else(|| "Microsoft".to_string());

        let id = format!("spotlight-{}", hash_url(&image_url));

        Ok(Wallpaper::new(
            id,
            image_url,
            title,
            description,
            attribution,
            "spotlight".to_string(),
        ))
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

            if status.as_u16() == 429 {
                retries += 1;
                if retries >= max_retries {
                    return Err(SourceError::RateLimited {
                        source_name: "Spotlight".to_string(),
                    });
                }

                let backoff = Duration::from_secs(2u64.pow(retries));
                tokio::time::sleep(backoff).await;
                continue;
            }

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
