use crate::config::Config;
use crate::contracts::{IERC20, IUniswapV2Router};
use crate::kuma::{KumaPushClient, KumaStatus};
use alloy::primitives::{Address, U256};
use alloy::providers::ProviderBuilder;
use alloy::signers::local::PrivateKeySigner;
use anyhow::Result;
use futures::future::join_all;
use rand::prelude::SliceRandom;
use std::sync::Arc;
use std::time::Duration;
use tokio::task::JoinHandle;
use tokio::time::sleep;
use tracing::{error, info};

/// The trading bot
pub struct TradingBot {
    handles: Vec<JoinHandle<()>>,
}

impl TradingBot {
    /// Create a new trading bot
    pub async fn new(config: Config, kuma_push_client: Arc<KumaPushClient>) -> Result<Self> {
        // Parse private key and create wallet
        let wallet: PrivateKeySigner = config.private_key.parse()?;
        let wallet_address = wallet.address();

        // Create provider
        let provider = ProviderBuilder::new()
            .with_cached_nonce_management()
            .wallet(wallet)
            .connect_http(config.rpc_url.parse()?);

        // Uniswap V2 Router address (Hyperion)
        let router: Address = "0xa1cF48c109f8B5eEe38B406591FE27f11f685a1f".parse()?;

        let router_contract = IUniswapV2Router::new(router, provider.clone());
        let mut handles = Vec::with_capacity(config.pairs.len());

        for pair in config.pairs {
            let provider = provider.clone();
            let router_contract = router_contract.clone();
            let kuma_push_id = pair.kuma_push_id.clone();
            let kuma_client = kuma_push_client.clone();

            let handle = tokio::spawn(async move {
                let token0_contract = IERC20::new(pair.token0, provider.clone());
                let token0_decimals = token0_contract.decimals().call().await.unwrap();

                let token1_contract = IERC20::new(pair.token1, provider.clone());
                let token1_decimals = token1_contract.decimals().call().await.unwrap();

                let mut tokens = [
                    (token0_contract, token0_decimals),
                    (token1_contract, token1_decimals),
                ];

                loop {
                    tokens.shuffle(&mut rand::rng());

                    let input_balance = tokens[0].0.balanceOf(wallet_address).call().await.unwrap();
                    let allowance = tokens[0]
                        .0
                        .allowance(wallet_address, router)
                        .call()
                        .await
                        .unwrap();

                    if allowance < input_balance {
                        if let Err(e) = tokens[0].0.approve(router, U256::MAX).send().await {
                            error!("Approve tx error: {}", e);
                        }
                    }

                    let decimals_diff = tokens[0].1 / 3;
                    let threshold = U256::from(10u64).pow(U256::from(tokens[0].1 - decimals_diff));
                    if input_balance < threshold {
                        let symbol = tokens[0].0.symbol().call().await.unwrap();
                        let msg = format!(
                            "Insufficient {} balance. Top up address {}",
                            symbol, wallet_address
                        );
                        if let Err(e) = kuma_client
                            .push(&kuma_push_id, KumaStatus::Down, Some(&msg))
                            .await
                        {
                            error!("Failed to send status update to Kuma push: {}", e);
                        }
                        sleep(Duration::from_secs(30)).await;
                        continue;
                    }

                    if let Err(e) = kuma_client
                        .push(&kuma_push_id, KumaStatus::Up, Some("Pair monitoring is up"))
                        .await
                    {
                        error!("Failed to send status update to Kuma push: {}", e);
                    }

                    let expiration = U256::MAX;
                    match router_contract
                        .swapExactTokensForTokens(
                            input_balance / U256::from(10u64),
                            U256::ZERO,
                            vec![*tokens[0].0.address(), *tokens[1].0.address()],
                            wallet_address,
                            expiration,
                        )
                        .send()
                        .await
                    {
                        Ok(pending) => {
                            let tx_hash = pending.tx_hash();
                            info!("Swap tx hash: {}", tx_hash);
                        }
                        Err(e) => {
                            error!("Swap tx error: {}", e);
                            sleep(Duration::from_secs(1)).await;
                            continue;
                        }
                    };

                    sleep(Duration::from_secs(1)).await;
                }
            });

            handles.push(handle);
        }

        Ok(Self { handles })
    }

    /// Run the trading bot
    pub async fn run(self) -> Result<()> {
        join_all(self.handles).await;
        Ok(())
    }
}
