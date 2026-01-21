# Massa ERC20 Token Contract

This project implements an ERC20-like token contract in Rust for the Massa blockchain using the [vibe_massa_rust_sdk](https://github.com/damip/vibe_massa_rust_sdk).

## Features

- Full ERC20 implementation:
  - `name()`, `symbol()`, `decimals()`, `totalSupply()`
  - `balanceOf(address)`
  - `transfer(to, amount)`
  - `approve(spender, amount)`
  - `allowance(owner, spender)`
  - `transferFrom(from, to, amount)`
- Additional functions:
  - `mint(to, amount)` - Mint new tokens
  - `burn(amount)` - Burn tokens from caller

## Project Structure

```
.
├── Cargo.toml                      # Workspace configuration
├── .cargo/config.toml              # WASM build configuration
├── contracts/
│   └── erc20-token/
│       ├── Cargo.toml
│       └── src/lib.rs              # ERC20 contract implementation
├── tests/
│   └── erc20-tests/
│       ├── Cargo.toml
│       └── src/lib.rs              # Contract tests
└── README.md
```

## Prerequisites

1. Install Rust:
   ```bash
   curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
   ```

2. Add the WASM target:
   ```bash
   rustup target add wasm32v1-none
   ```

## Building

Build the smart contract:

```bash
cargo build -p erc20-token --release --target wasm32v1-none
```

The compiled WASM will be in `target/wasm32v1-none/release/erc20_token.wasm`.

## Testing

First build the contract, then run tests:

```bash
# Build the contract
cargo build -p erc20-token --release --target wasm32v1-none

# Run tests
cargo test -p erc20-tests -- --nocapture
```

## Deploying to Buildnet

### 1. Get test MAS from faucet

Go to Massa Discord and use the buildnet faucet to get test tokens.

### 2. Prepare constructor arguments

```bash
# Using massa-cli from the SDK
cargo run -p massa-cli -- args \
  --arg string:MassaToken \
  --arg string:MTK \
  --arg u8:18 \
  --arg u64:1000000000000000000 \
  --out ./constructor.args
```

### 3. Deploy the contract

```bash
cargo run -p massa-cli -- deploy \
  --rpc https://buildnet.massa.net/api/v2 \
  --private-key <YOUR_PRIVATE_KEY> \
  --bytecode target/wasm32v1-none/release/erc20_token.wasm \
  --args-file ./constructor.args \
  --coins 1 \
  --wait-final
```

### 4. Call contract functions

**Transfer tokens:**
```bash
# Prepare transfer args (to_address, amount)
cargo run -p massa-cli -- args \
  --arg string:AU1... \
  --arg u64:1000000 \
  --out ./transfer.args

# Execute transfer
cargo run -p massa-cli -- call \
  --rpc https://buildnet.massa.net/api/v2 \
  --private-key <YOUR_PRIVATE_KEY> \
  --target <CONTRACT_ADDRESS> \
  --function transfer \
  --args-file ./transfer.args \
  --coins 0 \
  --wait-final
```

**Check balance:**
```bash
# Prepare balanceOf args
cargo run -p massa-cli -- args \
  --arg string:AU1... \
  --out ./balance.args

# Call balanceOf (read-only call)
cargo run -p massa-cli -- call \
  --rpc https://buildnet.massa.net/api/v2 \
  --private-key <YOUR_PRIVATE_KEY> \
  --target <CONTRACT_ADDRESS> \
  --function balanceOf \
  --args-file ./balance.args \
  --coins 0
```

### 5. Read events

```bash
cargo run -p massa-cli -- events \
  --rpc https://buildnet.massa.net/api/v2 \
  --operation-id <OPERATION_ID> \
  --final-only
```

## Contract Interface

### Constructor
```
constructor(name: string, symbol: string, decimals: u8, initial_supply: u64)
```
Initializes the token with the given parameters and mints initial supply to deployer.

### Read Functions
- `name() -> string` - Returns token name
- `symbol() -> string` - Returns token symbol
- `decimals() -> u8` - Returns token decimals
- `totalSupply() -> u64` - Returns total supply
- `balanceOf(address: string) -> u64` - Returns balance of address
- `allowance(owner: string, spender: string) -> u64` - Returns allowance

### Write Functions
- `transfer(to: string, amount: u64) -> bool` - Transfer tokens
- `approve(spender: string, amount: u64) -> bool` - Approve allowance
- `transferFrom(from: string, to: string, amount: u64) -> bool` - Transfer using allowance
- `mint(to: string, amount: u64) -> bool` - Mint new tokens
- `burn(amount: u64) -> bool` - Burn tokens

### Events
- `ERC20_DEPLOYED:{name}:{symbol}:{decimals}:{initial_supply}:{deployer}`
- `TRANSFER:{from}:{to}:{amount}`
- `APPROVAL:{owner}:{spender}:{amount}`
- `ERROR:{message}`

## License

MIT OR Apache-2.0

## Original prompt

```
use this tools  https://github.com/damip/vibe_massa_rust_sdk (feel free to clone it separately as a temporary reference, delete it after, don' build the project in there)
Check massa docs: https://docs.massa.net

Create a new project,
Implement an ERC20 contract in rust, test it, deploy it on buildnet, call some of its functions directly on buildnet to attempt a coin transfer
```
