# Hyperion DEX Bot

A trading bot for UniswapV2 pairs that automatically executes trades on configured pools.

## Features

- Trades on multiple UniswapV2 pools
- Executes trades every second
- Uses random percentage of available balance for each trade
- Alternates between token0 and token1 for each pool
- Configurable via TOML configuration file
- Built with Rust and the alloy crate for Ethereum interaction
- Integrated with Kuma Push for uptime monitoring and status updates

## Installation

### Prerequisites

- Rust and Cargo (latest stable version)
- An Ethereum RPC endpoint (e.g., from Alchemy, Infura, or your own node)
- A wallet with some ETH for gas and tokens for trading

### Building from source

```bash
# Clone the repository
git clone https://github.com/yourusername/hyperion_dex_bot.git
cd hyperion_dex_bot

# Build the project
cargo build --release

# The binary will be available at target/release/hyperion_dex_bot
```

## Configuration

Create a configuration file based on the provided example:

```bash
cp config.example.toml config.toml
```

Edit the `config.toml` file to include your:
- Ethereum RPC URL
- Private key (without 0x prefix)
- General Kuma Push URL for bot uptime monitoring
- List of UniswapV2 pools to trade on, each with its own Kuma Push URL

Example configuration:

```toml
# Ethereum RPC URL
rpc_url = "https://eth-mainnet.g.alchemy.com/v2/your-api-key"

# Private key for the trading account (hex string without 0x prefix)
private_key = "your-private-key-here"

# Push URL used to track that bot is actually running
general_kuma_push = "http://kuma.example.com/api/push/your-push-id"

# List of pairs to trade, need to have a direct UniswapV2Pair
# Contains Kuma push URL to monitor status of specific pair
[[pairs]]
name = "USDT-WETH"
token0 = "0x3c099E287eC71b4AA61A7110287D715389329237"
token1 = "0x9AB236Ec38492099a4d35552e6dC7D9442607f9A"
kuma_push = "http://kuma.example.com/api/push/pair1-push-id"

[[pairs]]
name = "USDC-WETH"
token0 = "0x1234567890abcdef1234567890abcdef12345678"
token1 = "0x9AB236Ec38492099a4d35552e6dC7D9442607f9A"
kuma_push = "http://kuma.example.com/api/push/pair2-push-id"
```

## Usage

Run the bot with the default configuration file:

```bash
./target/release/hyperion_dex_bot
```

Or specify a custom configuration file:

```bash
./target/release/hyperion_dex_bot --config my_config.toml
```

## Trading Strategy

The bot implements a simple trading strategy:

1. For each configured pool, the bot trades every second
2. The trade amount is a random percentage (1-10%) of the current token balance
3. If the previous trade was from token0 to token1, the next trade will be from token1 to token0, and vice versa

## Monitoring with Kuma Push

The bot integrates with [Kuma Push](http://kuma.example.com) for uptime monitoring and status updates:

1. **Bot-level monitoring**: When the bot starts, it sends an "up" status to the general Kuma Push URL configured in `general_kuma_push`.

2. **Pair-level monitoring**: For each trading pair, the bot sends status updates to the pair-specific Kuma Push URL configured in `kuma_push`:
   - When pair monitoring starts: Sends an "up" status with message "Pair monitoring started"
   - After successful swaps: Sends an "up" status with the transaction hash
   - After failed swaps: Sends a "down" status with the error message

This allows you to monitor both the overall bot status and the status of individual trading pairs in real-time.

## Security Considerations

- **NEVER** commit your private key to version control
- Consider using environment variables or a secure secret manager for production use
- Ensure your wallet has sufficient ETH for gas fees
- Be aware of the risks associated with automated trading

## License

MIT

## Disclaimer

This software is provided for educational purposes only. Use at your own risk. The authors are not responsible for any financial losses incurred by using this software.