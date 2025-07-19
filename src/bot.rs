use crate::config::Config;
use crate::uniswap::{IERC20, IUniswapV2Router};
use alloy::primitives::{Address, U256};
use alloy::providers::ProviderBuilder;
use alloy::signers::local::PrivateKeySigner;
use anyhow::Result;
use futures::future::join_all;
use rand::prelude::SliceRandom;
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
    pub async fn new(config: Config) -> Result<Self> {
        // Parse private key and create wallet
        let wallet: PrivateKeySigner = config.private_key.parse()?;
        let wallet_address = wallet.address();

        // Create provider
        let provider = ProviderBuilder::new()
            .wallet(wallet)
            .connect_http(config.rpc_url.parse()?);

        // Uniswap V2 Router address (Hyperion)
        let router: Address = "0xa1cF48c109f8B5eEe38B406591FE27f11f685a1f".parse()?;

        let router_contract = IUniswapV2Router::new(router, provider.clone());
        let mut handles = Vec::with_capacity(config.pairs.len());

        for pair in config.pairs {
            let provider = provider.clone();
            let router_contract = router_contract.clone();

            let handle = tokio::spawn(async move {
                let token0_contract = IERC20::new(pair.token0, provider.clone());
                let token1_contract = IERC20::new(pair.token1, provider.clone());
                let mut tokens = [token0_contract, token1_contract];

                loop {
                    tokens.shuffle(&mut rand::rng());

                    let input_balance = tokens[0].balanceOf(wallet_address).call().await.unwrap();
                    let allowance = tokens[0]
                        .allowance(wallet_address, router)
                        .call()
                        .await
                        .unwrap();
                    if allowance < input_balance {
                        if let Err(e) = tokens[0]
                            .approve(router, U256::MAX)
                            .max_priority_fee_per_gas(3_000_000_000u128)
                            .send()
                            .await
                        {
                            error!("Approve tx error: {}", e);
                        }
                    }

                    let expiration = U256::MAX;
                    match router_contract
                        .swapExactTokensForTokens(
                            input_balance / U256::from(10u64),
                            U256::ZERO,
                            vec![*tokens[0].address(), *tokens[1].address()],
                            wallet_address,
                            expiration,
                        )
                        .max_priority_fee_per_gas(3_000_000_000u128)
                        .send()
                        .await
                    {
                        Ok(pending) => {
                            info!("Swap tx hash: {}", pending.tx_hash());
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
