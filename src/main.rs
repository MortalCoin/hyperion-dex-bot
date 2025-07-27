use anyhow::{Context, Result};
use clap::Parser;
use hyperion_dex_bot::{Config, KumaPushClient, KumaStatus, TradingBot};
use std::path::PathBuf;
use std::sync::Arc;
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
    let _subscriber = FmtSubscriber::builder()
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

    let base_kuma_url = reqwest::Url::parse(&config.base_kuma_url)?;
    let kuma_push_client = Arc::new(KumaPushClient::new(base_kuma_url));

    tokio::spawn({
        let kuma_push_client = kuma_push_client.clone();
        let push_id = config.general_push_id.clone();
        async move {
            loop {
                if let Err(e) = kuma_push_client
                    .push(&push_id, KumaStatus::Up, Some("Bot is running"))
                    .await
                {
                    error!("Failed to send status update to Kuma push: {}", e);
                }

                sleep(Duration::from_secs(55)).await;
            }
        }
    });

    // Create and run the trading bot
    info!("Initializing trading bot");
    let bot = TradingBot::new(config, kuma_push_client).await?;

    info!("Running trading bot");
    bot.run().await?;

    Ok(())
}
