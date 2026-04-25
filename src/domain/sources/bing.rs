use async_trait::async_trait;
use reqwest::Client;
use serde::Deserialize;
use std::time::Duration;

use crate::domain::entities::Wallpaper;
use crate::domain::errors::SourceError;
use crate::domain::traits::Source;

/// Bing Wallpaper of the Day API response structure
#[derive(Debug, Deserialize)]
struct BingResponse {
    images: Vec<BingImage>,
}

#[derive(Debug, Deserialize)]
struct BingImage {
    #[serde(rename = "url")]
    url_path: String,
    #[serde(rename = "copyright")]
    description: String,
    #[serde(rename = "title")]
    title: String,
    #[serde(rename = "startdate")]
    date: String,
}

/// Bing Wallpaper of the Day source
///
/// Fetches the daily wallpaper from Microsoft Bing's Wallpaper of the Day API.
pub struct BingSource {
    client: Client,
    market: String,
    base_url: &'static str,
}

impl BingSource {
    /// Create a new BingSource with default market (en-US)
    pub fn new() -> Self {
        Self::with_market("en-US".to_string())
    }

    /// Create a new BingSource with specified market
    pub fn with_market(market: String) -> Self {
        Self {
            client: Client::builder()
                .timeout(Duration::from_secs(10))
                .build()
                .expect("Failed to create HTTP client"),
            market,
            base_url: "https://www.bing.com/HPImageArchive.aspx",
        }
    }
}

impl Default for BingSource {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl Source for BingSource {
    async fn fetch(&self) -> Result<Wallpaper, SourceError> {
        // Build API URL with parameters
        let url = format!("{}?format=js&idx=0&n=1&mkt={}", self.base_url, self.market);

        // Fetch from Bing API with retry for rate limiting
        let response = self.fetch_with_retry(&url, 3).await?;

        // Parse JSON response
        let bing_response: BingResponse = response.json().await.map_err(|e| {
            SourceError::ParseError(format!("Failed to parse Bing API response: {}", e))
        })?;

        // Extract first image
        let image = bing_response
            .images
            .into_iter()
            .next()
            .ok_or(SourceError::NoWallpaperFound)?;

        // Construct full image URL
        let full_url = format!("https://www.bing.com{}", image.url_path);

        // Extract attribution (copyright holder from copyright text)
        let attribution = extract_attribution(&image.description);

        // Create ID from date + market to avoid cache collision across markets
        let id = format!("bing-{}-{}", self.market, image.date);

        Ok(Wallpaper::new(
            id,
            full_url,
            image.title,
            image.description,
            attribution,
            "bing".to_string(),
        ))
    }
}

impl BingSource {
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
                        source_name: "Bing".to_string(),
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
                    source_name: format!("Bing API returned status {}", status),
                });
            }

            return Ok(response);
        }
    }
}

/// Extract attribution (copyright holder) from copyright string.
///
/// Expected format: "Description text (© Copyright Holder)"
fn extract_attribution(copyright: &str) -> String {
    copyright
        .split('©')
        .nth(1)
        .and_then(|s| s.split(')').next())
        .map(|s| s.trim().to_string())
        .unwrap_or_else(|| "Microsoft Bing".to_string())
}
