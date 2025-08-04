use crate::config::Config;
use crate::contracts::IERC20::IERC20Instance;
use crate::contracts::{GameContract, IERC20, IUniswapV2Router};
use crate::kuma::{KumaPushClient, KumaStatus};
use alloy::primitives::U256;
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

struct Token<T> {
    contract: IERC20Instance<T>,
    min_balance: U256,
}

impl TradingBot {
    /// Create a new trading bot
    pub async fn new(config: Config, kuma_push_client: Arc<KumaPushClient>) -> Result<Self> {
        // Parse private key and create wallet
        let wallet: PrivateKeySigner = config.private_key.parse()?;
        let wallet_address = wallet.address();

        // Create provider
        let provider = ProviderBuilder::new()
            .wallet(wallet)
            .connect_http(config.rpc_url.parse()?);

        let router_contract = IUniswapV2Router::new(config.uniswap_v2_router, provider.clone());
        let mut handles = Vec::with_capacity(config.pairs.len());

        let game_contract = GameContract::new(config.game_contract, provider.clone());

        for pair in config.pairs {
            let provider = provider.clone();
            let router_contract = router_contract.clone();
            let kuma_push_id = pair.kuma_push_id.clone();
            let kuma_client = kuma_push_client.clone();
            let game_contract = game_contract.clone();

            let handle = tokio::spawn(async move {
                let token0_contract = IERC20::new(pair.token0, provider.clone());
                let token1_contract = IERC20::new(pair.token1, provider.clone());

                let mut tokens = [
                    Token {
                        contract: token0_contract,
                        min_balance: U256::from(pair.min_balance0),
                    },
                    Token {
                        contract: token1_contract,
                        min_balance: U256::from(pair.min_balance1),
                    },
                ];

                loop {
                    tokens.shuffle(&mut rand::rng());

                    let input_balance = tokens[0]
                        .contract
                        .balanceOf(wallet_address)
                        .call()
                        .await
                        .unwrap();

                    let threshold = tokens[0].min_balance;

                    if input_balance < threshold {
                        let symbol = tokens[0].contract.symbol().call().await.unwrap();
                        let msg = format!(
                            "Insufficient {} balance. Top up address {}",
                            symbol, wallet_address
                        );
                        error!("{msg}");
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
                        .push(&kuma_push_id, KumaStatus::Up, Some("Pair is up"))
                        .await
                    {
                        error!("Failed to send status update to Kuma push: {}", e);
                    }

                    let active_games = game_contract.activeGames().call().await.unwrap();
                    if active_games == U256::ZERO {
                        info!("No active games");
                        sleep(Duration::from_secs(1)).await;
                        continue;
                    }

                    let allowance = tokens[0]
                        .contract
                        .allowance(wallet_address, *router_contract.address())
                        .call()
                        .await
                        .unwrap();

                    if allowance < input_balance {
                        if let Err(e) = tokens[0]
                            .contract
                            .approve(*router_contract.address(), U256::MAX)
                            .send()
                            .await
                        {
                            error!("Approve tx error: {}", e);
                        }
                    }

                    let amount_in = input_balance / U256::from(10u64);

                    let path = vec![*tokens[0].contract.address(), *tokens[1].contract.address()];
                    let amounts_out = match router_contract
                        .getAmountsOut(amount_in, path.clone())
                        .call()
                        .await
                    {
                        Ok(amounts_out) => amounts_out,
                        Err(e) => {
                            error!("Get amounts out error: {}", e);
                            sleep(Duration::from_secs(1)).await;
                            continue;
                        }
                    };

                    let expiration = U256::MAX;
                    let expected_output = amounts_out[1];
                    let slippage_tolerance = U256::from(99u64); // 99% (1% slippage)
                    let amount_out_min =
                        (expected_output * slippage_tolerance) / U256::from(100u64);

                    match router_contract
                        .swapExactTokensForTokens(
                            amount_in,
                            amount_out_min,
                            path,
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

                    sleep(Duration::from_secs(3)).await;
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
