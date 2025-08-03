use alloy::primitives::Address;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum ConfigError {
    #[error("Failed to read config file: {0}")]
    IoError(#[from] std::io::Error),
    #[error("Failed to parse config file: {0}")]
    ParseError(#[from] toml::de::Error),
    #[error("Invalid configuration: {0}")]
    ValidationError(String),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PoolConfig {
    pub name: String,
    pub token0: Address,
    pub min_balance0: u64,
    pub token1: Address,
    pub min_balance1: u64,
    pub kuma_push_id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    /// Ethereum RPC URL
    pub rpc_url: String,
    /// Address of Uniswap V2 router
    pub uniswap_v2_router: Address,
    /// Address of Mortal Coin game contract
    pub game_contract: Address,
    /// Base URL for Kuma Push service
    pub base_kuma_url: String,
    /// General Kuma Push ID for monitoring the bot
    pub general_push_id: String,
    /// Private key for the trading account (hex string without 0x prefix)
    pub private_key: String,
    /// List of pairs to trade
    pub pairs: Vec<PoolConfig>,
}

impl Config {
    /// Load configuration from a TOML file
    pub fn from_file<P: AsRef<Path>>(path: P) -> Result<Self, ConfigError> {
        let content = fs::read_to_string(path)?;
        let config: Config = toml::from_str(&content)?;

        // Validate configuration
        if config.rpc_url.is_empty() {
            return Err(ConfigError::ValidationError(
                "RPC URL cannot be empty".to_string(),
            ));
        }

        if config.private_key.is_empty() {
            return Err(ConfigError::ValidationError(
                "Private key cannot be empty".to_string(),
            ));
        }

        if config.pairs.is_empty() {
            return Err(ConfigError::ValidationError(
                "At least one pool must be specified".to_string(),
            ));
        }

        Ok(config)
    }
}
