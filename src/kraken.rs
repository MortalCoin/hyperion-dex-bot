use anyhow::{Result, anyhow};
use reqwest::Client;
use rust_decimal::{Decimal, dec};

#[derive(Clone)]
pub struct KrakenClient {
    client: Client,
}

impl KrakenClient {
    pub fn new() -> Self {
        Self {
            client: Client::new(),
        }
    }

    pub async fn get_price(&self, pair: &str, reverse: bool) -> Result<Decimal> {
        // Kraken public ticker endpoint
        let url = format!("https://api.kraken.com/0/public/Ticker?pair={}", pair);
        let resp = self.client.get(&url).send().await?;
        if !resp.status().is_success() {
            return Err(anyhow!("Kraken HTTP error: {}", resp.status()));
        }
        let json: serde_json::Value = resp.json().await?;
        if let Some(errors) = json.get("error").and_then(|e| e.as_array()) {
            if !errors.is_empty() {
                return Err(anyhow!("Kraken API error: {:?}", errors));
            }
        }
        let result = json
            .get("result")
            .ok_or_else(|| anyhow!("Missing result in Kraken response"))?;
        // The key inside result can be different from requested pair, so pick the first object value.
        let first_pair_obj = result
            .as_object()
            .and_then(|m| m.values().next())
            .ok_or_else(|| anyhow!("Missing pair data in Kraken result"))?;
        let last_trade = first_pair_obj
            .get("c")
            .and_then(|c| c.as_array())
            .and_then(|arr| arr.get(0))
            .and_then(|v| v.as_str())
            .ok_or_else(|| anyhow!("Missing last trade price in Kraken result"))?;
        let price = last_trade.parse()?;
        if reverse {
            Ok(dec!(1.0) / price)
        } else {
            Ok(price)
        }
    }
}
