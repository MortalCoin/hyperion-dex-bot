use alloy::primitives::{Address, Bytes, U256};
use alloy::providers::{Provider, ProviderBuilder, WalletProvider};
use alloy::signers::local::PrivateKeySigner;
use alloy::sol_types::sol;
use anyhow::Result;
use std::ops::Div;
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
