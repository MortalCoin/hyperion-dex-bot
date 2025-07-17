use anyhow::{Context, Result};
use clap::Parser;
use hyperion_dex_bot::{Config, TradingBot};
use std::path::PathBuf;
use std::time::Duration;
use tokio::time::sleep;
use tracing::{error, info};
use tracing_subscriber::FmtSubscriber;

/// UniswapV2 trading bot
#[derive(Parser, Debug)]
#[clap(author, version, about)]
struct Args {
    /// Path to the configuration file
    #[clap(short, long, value_parser, default_value = "config.toml")]
    config: PathBuf,
}

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize logging
    let subscriber = FmtSubscriber::builder()
        .with_max_level(tracing::Level::INFO)
        .finish();
    tracing_subscriber::fmt::init();

    info!("Starting Hyperion DEX Bot");

    // Parse command-line arguments
    let args = Args::parse();

    // Load configuration
    info!("Loading configuration from {}", args.config.display());
    let config = Config::from_file(&args.config).with_context(|| {
        format!(
            "Failed to load configuration from {}",
            args.config.display()
        )
    })?;

    info!("Configuration loaded successfully");
    info!("RPC URL: {}", config.rpc_url);
    info!("Number of pairs: {}", config.pairs.len());

    // Create and run the trading bot
    info!("Initializing trading bot");
    let bot = TradingBot::new(config).await?;

    info!("Running trading bot");
    bot.run().await?;

    Ok(())
}
