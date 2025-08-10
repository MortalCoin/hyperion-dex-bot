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
        function getAmountsOut(uint amountIn, address[] memory path) public view returns (uint[] memory amounts);
    }
}

sol!(
    #[sol(rpc)]
    contract IERC20 {
        function balanceOf(address target) returns (uint256);
        function approve(address spender, uint256 amount) external returns (bool);
        function allowance(address owner, address spender) external view returns (uint256);
        function decimals() external view returns (uint8);
        function name() external view returns (string memory);
        function symbol() external view returns (string memory);
    }
);

sol!(
    #[sol(rpc)]
    contract GameContract {
        function activeGames() external view returns (uint256);
    }
);

sol!(
    #[sol(rpc)]
    contract IUniswapV2Pair {
        function getReserves() external view returns (uint112 reserve0, uint112 reserve1, uint32 blockTimestampLast);
        function token0() external view returns (address);
        function token1() external view returns (address);
    }
);
