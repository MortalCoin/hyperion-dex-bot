use crate::config::Config;
use crate::contracts::IERC20::IERC20Instance;
use crate::contracts::{IERC20, IUniswapV2Pair, IUniswapV2Router};
use crate::kraken::KrakenClient;
use crate::kuma::{KumaPushClient, KumaStatus};
use alloy::primitives::U256;
use alloy::providers::{Provider, ProviderBuilder};
use alloy::signers::local::PrivateKeySigner;
use anyhow::Result;
use futures::future::join_all;
use rust_decimal::prelude::ToPrimitive;
use rust_decimal::{Decimal, MathematicalOps, dec};
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
    decimals: u8,
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

        for pair in config.pairs {
            let provider = provider.clone();
            let router_contract = router_contract.clone();
            let kuma_push_id = pair.kuma_push_id.clone();
            let kuma_client = kuma_push_client.clone();
            let kraken_pair = pair.kraken_pair.clone();
            let kraken_client = KrakenClient::new();

            let handle = tokio::spawn(async move {
                let token0_contract = IERC20::new(pair.token0, provider.clone());
                // should fail early here if we can't fetch decimals
                let decimals0 = token0_contract.decimals().call().await.unwrap();

                let token1_contract = IERC20::new(pair.token1, provider.clone());
                // should fail early here if we can't fetch decimals
                let decimals1 = token1_contract.decimals().call().await.unwrap();

                let token0 = Token {
                    contract: token0_contract,
                    min_balance: U256::from(pair.min_balance0),
                    decimals: decimals0,
                };
                let token1 = Token {
                    contract: token1_contract,
                    min_balance: U256::from(pair.min_balance1),
                    decimals: decimals1,
                };

                let pair_contract = IUniswapV2Pair::new(pair.pair_address, provider.clone());

                loop {
                    // Compute current pool price using reserves: price = (r1 * 10^d0) / (r0 * 10^d1)
                    let (reserve0, reserve1): (u128, u128) =
                        match pair_contract.getReserves().call().await {
                            Ok(res) => (res.reserve0.to(), res.reserve1.to()),
                            Err(e) => {
                                error!("Failed to fetch reserves: {}", e);
                                sleep(Duration::from_secs(1)).await;
                                continue;
                            }
                        };
                    let reserve0_dec =
                        Decimal::from_i128_with_scale(reserve0 as i128, decimals0 as u32);
                    let reserve1_dec =
                        Decimal::from_i128_with_scale(reserve1 as i128, decimals1 as u32);

                    let pool_price = reserve1_dec / reserve0_dec;

                    let kraken_price = match kraken_client
                        .get_price(&kraken_pair, pair.reverse_kraken_pair)
                        .await
                    {
                        Ok(kr_price) => kr_price,
                        Err(e) => {
                            error!("Failed to fetch Kraken price: {}", e);
                            sleep(Duration::from_secs(1)).await;
                            continue;
                        }
                    };

                    let (input_token, input_amount_dec, output_token) = if kraken_price > pool_price
                    {
                        let target_price = kraken_price * dec!(1.0001);
                        let delta_token1 =
                            (reserve0_dec * reserve1_dec * target_price).sqrt().unwrap()
                                - reserve1_dec;
                        (&token1, delta_token1, &token0)
                    } else {
                        let target_price = kraken_price * dec!(0.9999);
                        let delta_token0 =
                            (reserve0_dec * reserve1_dec / target_price).sqrt().unwrap()
                                - reserve0_dec;
                        (&token0, delta_token0, &token1)
                    };

                    let input_balance =
                        match input_token.contract.balanceOf(wallet_address).call().await {
                            Ok(balance) => balance,
                            Err(e) => {
                                error!("Failed to get balance: {}", e);
                                sleep(Duration::from_secs(1)).await;
                                continue;
                            }
                        };

                    let threshold = input_token.min_balance;

                    if input_balance < threshold {
                        let symbol = match input_token.contract.symbol().call().await {
                            Ok(symbol) => symbol,
                            Err(e) => {
                                error!("Failed to get symbol: {}", e);
                                sleep(Duration::from_secs(1)).await;
                                continue;
                            }
                        };
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

                    let gas_balance = match provider.get_balance(wallet_address).await {
                        Ok(balance) => balance,
                        Err(e) => {
                            error!("Failed to get gas balance: {}", e);
                            sleep(Duration::from_secs(1)).await;
                            continue;
                        }
                    };
                    let gas_threshold = U256::from(10).pow(U256::from(17));
                    if gas_balance < gas_threshold {
                        let msg = format!(
                            "Insufficient gas balance. Top up address {}",
                            wallet_address
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

                    let allowance = match input_token
                        .contract
                        .allowance(wallet_address, *router_contract.address())
                        .call()
                        .await
                    {
                        Ok(allowance) => allowance,
                        Err(e) => {
                            error!("Failed to get allowance: {}", e);
                            sleep(Duration::from_secs(1)).await;
                            continue;
                        }
                    };

                    if allowance < input_balance {
                        if let Err(e) = input_token
                            .contract
                            .approve(*router_contract.address(), U256::MAX)
                            .send()
                            .await
                        {
                            error!("Approve tx error: {}", e);
                        }
                    }

                    let path = vec![
                        *input_token.contract.address(),
                        *output_token.contract.address(),
                    ];

                    let expiration = U256::MAX;

                    let pow = 10u64.pow(input_token.decimals as u32);
                    let input_amount = (input_amount_dec * Decimal::from(pow))
                        .trunc()
                        .to_u128()
                        .unwrap();

                    match router_contract
                        .swapExactTokensForTokens(
                            U256::from(input_amount),
                            U256::ZERO,
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

                    sleep(Duration::from_secs(9)).await;
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
