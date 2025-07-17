# Hyperion DEX Bot

A trading bot for UniswapV2 pairs that automatically executes trades on configured pools.

## Features

- Trades on multiple UniswapV2 pools
- Executes trades every second
- Uses random percentage of available balance for each trade
- Alternates between token0 and token1 for each pool
- Configurable via TOML configuration file
- Built with Rust and the alloy crate for Ethereum interaction

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
- List of UniswapV2 pools to trade on

Example configuration:

```toml
# Ethereum RPC URL
rpc_url = "https://eth-mainnet.g.alchemy.com/v2/your-api-key"

# Private key for the trading account (hex string without 0x prefix)
private_key = "your-private-key-here"

# List of UniswapV2 pools to trade on
[[pools]]
address = "0xB4e16d0168e52d35CaCD2c6185b44281Ec28C9Dc"
name = "ETH-USDC"

[[pools]]
address = "0x0d4a11d5EEaaC28EC3F61d100daF4d40471f1852"
name = "ETH-USDT"
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

## Security Considerations

- **NEVER** commit your private key to version control
- Consider using environment variables or a secure secret manager for production use
- Ensure your wallet has sufficient ETH for gas fees
- Be aware of the risks associated with automated trading

## License

MIT

## Disclaimer

This software is provided for educational purposes only. Use at your own risk. The authors are not responsible for any financial losses incurred by using this software.