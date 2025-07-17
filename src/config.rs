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
    /// Address of the UniswapV2 pair contract
    pub address: Address,
    /// Name or description of the pool (optional)
    #[serde(default)]
    pub name: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    /// Ethereum RPC URL
    pub rpc_url: String,
    /// Private key for the trading account (hex string without 0x prefix)
    pub private_key: String,
    /// List of UniswapV2 pools to trade on
    pub pools: Vec<PoolConfig>,
}

impl Config {
    /// Load configuration from a TOML file
    pub fn from_file<P: AsRef<Path>>(path: P) -> Result<Self, ConfigError> {
        let content = fs::read_to_string(path)?;
        let config: Config = toml::from_str(&content)?;
        
        // Validate configuration
        if config.rpc_url.is_empty() {
            return Err(ConfigError::ValidationError("RPC URL cannot be empty".to_string()));
        }
        
        if config.private_key.is_empty() {
            return Err(ConfigError::ValidationError("Private key cannot be empty".to_string()));
        }
        
        if config.pools.is_empty() {
            return Err(ConfigError::ValidationError("At least one pool must be specified".to_string()));
        }
        
        Ok(config)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_parse_valid_config() {
        let config_str = r#"
            rpc_url = "https://eth-mainnet.alchemyapi.io/v2/your-api-key"
            private_key = "abcdef1234567890abcdef1234567890abcdef1234567890abcdef1234567890"
            
            [[pools]]
            address = "0x0000000000000000000000000000000000000001"
            name = "ETH-USDC"
            
            [[pools]]
            address = "0x0000000000000000000000000000000000000002"
        "#;
        
        let config: Config = toml::from_str(config_str).unwrap();
        assert_eq!(config.pools.len(), 2);
        assert_eq!(config.pools[0].name, Some("ETH-USDC".to_string()));
        assert!(config.pools[1].name.is_none());
    }
}