use anyhow::Result;
use reqwest::Client;
use std::time::Duration;
use tracing::{debug, error};

/// Status options for Kuma push
#[derive(Debug, Clone, Copy)]
pub enum KumaStatus {
    /// Service is up and running
    Up,
    /// Service is down or experiencing issues
    Down,
}

impl KumaStatus {
    /// Convert to string representation for the API
    fn as_str(&self) -> &'static str {
        match self {
            KumaStatus::Up => "up",
            KumaStatus::Down => "down",
        }
    }
}

/// Client for sending status updates to Kuma push
#[derive(Clone)]
pub struct KumaPushClient {
    client: Client,
    base_url: reqwest::Url,
}

impl KumaPushClient {
    /// Create a new Kuma push client
    pub fn new(base_url: reqwest::Url) -> Self {
        let client = Client::builder()
            .timeout(Duration::from_secs(5))
            .build()
            .expect("Failed to create HTTP client");

        Self { client, base_url }
    }

    /// Send a status update to the Kuma push endpoint
    ///
    /// # Arguments
    ///
    /// * `id` - The Kuma push ID
    /// * `status` - The status (up or down)
    /// * `msg` - Optional message to include with the status update
    ///
    /// # Example
    ///
    /// ```
    /// use hyperion_dex_bot::kuma::{KumaPushClient, KumaStatus};
    /// use reqwest::Url;
    ///
    /// async fn example() {
    ///     let base_url = Url::parse("http://example.com").unwrap();
    ///     let client = KumaPushClient::new(base_url);
    ///     client.push("pushID", KumaStatus::Up, Some("Bot is running")).await.unwrap();
    /// }
    /// ```
    pub async fn push(&self, id: &str, status: KumaStatus, msg: Option<&str>) -> Result<()> {
        // Build the URL with the ID and query parameters
        let mut url = self.base_url.join(&format!("api/push/{}", id))?;

        // Add status parameter
        url.query_pairs_mut().append_pair("status", status.as_str());

        // Add message parameter if provided
        if let Some(message) = msg {
            url.query_pairs_mut().append_pair("msg", message);
        }

        // Add empty ping parameter
        url.query_pairs_mut().append_pair("ping", "");

        let response = self.client.get(url).send().await?;

        if response.status().is_success() {
            debug!("Kuma push successful for ID: {}", id);
            Ok(())
        } else {
            let status = response.status();
            error!("Kuma push failed with status {} for ID: {}", status, id);
            Err(anyhow::anyhow!("Kuma push failed with status {}", status))
        }
    }
}
