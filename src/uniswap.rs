use std::ops::Div;
use alloy::primitives::{Address, Bytes, U256};
use alloy::providers::{Provider, ProviderBuilder, WalletProvider};
use alloy::signers::local::PrivateKeySigner;
use alloy::sol_types::{sol};
use anyhow::{Result};
use std::sync::Arc;

pub struct UniswapClient {
    provider: Arc<dyn Provider>,
    router: Address,
}

sol! {
    #[sol(rpc)]
    contract IUniswapV2Router {
        function swapExactTokensForTokens(
          uint amountIn,
          uint amountOutMin,
          address[] calldata path,
          address to,
          uint deadline
        ) external returns (uint[] memory amounts);
    }
}

sol!(
    #[sol(rpc)]
    contract IERC20 {
        function balanceOf(address target) returns (uint256);
        function approve(address spender, uint256 amount) external returns (bool);
    }
);

impl UniswapClient {
    /// Create a new UniswapClient
    pub async fn new(rpc_url: &str, private_key: &str) -> Result<Self> {
        // Parse private key and create wallet
        let wallet: PrivateKeySigner = private_key.parse()?;
        
        // Create provider
        let provider = ProviderBuilder::new()
            .wallet(wallet)
            .connect_http(rpc_url.parse()?);
        
        // Uniswap V2 Router address (Hyperion)
        let router: Address = "0xa1cF48c109f8B5eEe38B406591FE27f11f685a1f".parse()?;

        let usdt_weth_pool: Address = "0x37f8084c6ed4228378A7beC5819872b595B00223".parse()?;
        let router_contract = IUniswapV2Router::new(router, provider.clone());

        let usdt_address: Address = "0x3c099E287eC71b4AA61A7110287D715389329237".parse()?;
        let usdt = IERC20::new(usdt_address, provider.clone());
        let approve_receipt = usdt.approve(router, U256::MAX).send().await?.get_receipt().await?;
        println!("Approve receipt: {:?}", approve_receipt);
        
        let address = provider.default_signer_address();
        let weth_address: Address = "0x9AB236Ec38492099a4d35552e6dC7D9442607f9A".parse()?;
        let usdt_balance = usdt.balanceOf(address).call().await?;

        println!("USDT balance: {}", usdt_balance);

        let expiration = U256::MAX;
        let router_call_receipt = router_contract.swapExactTokensForTokens(usdt_balance.div(U256::from(10u64)), U256::ZERO, vec![usdt_address, weth_address], address, expiration).send().await?.get_receipt().await?;

        println!("Router call receipt: {:?}", router_call_receipt);

        Ok(Self {
            provider: Arc::new(provider),
            router,
        })
    }
}
