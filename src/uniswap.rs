use alloy::sol_types::sol;

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
