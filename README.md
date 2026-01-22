# Massa MRC20 Token Contract (Rust)

This project implements a fully MRC20-compatible fungible token smart contract in Rust for the Massa blockchain using the [vibe_massa_rust_sdk](https://github.com/damip/vibe_massa_rust_sdk).

## MRC20 Compatibility

This contract is **fully compatible** with the [MRC20 standard](https://github.com/massalabs/massa-standards) (Massa's ERC20 equivalent):

- **Storage format**: Identical key/value encoding as AssemblyScript implementation
- **Function signatures**: Same prototypes, arguments, and return values
- **Events**: Same event names and formats (`TRANSFER SUCCESS`, `APPROVAL SUCCESS`, `MINT SUCCESS`, `BURN_SUCCESS`, `CHANGE_OWNER`)
- **Deployer**: Compatible with Massa's standard deployment pipeline
- **U256 amounts**: Uses proper 256-bit integers from `massa-types` crate with safe arithmetic

### Storage Keys (matching AS implementation)
| Key | Format | Description |
|-----|--------|-------------|
| `NAME` | raw bytes | Token name |
| `SYMBOL` | raw bytes | Token symbol |
| `DECIMALS` | 1 byte | Number of decimals |
| `TOTAL_SUPPLY` | 32 bytes (U256 LE) | Total supply |
| `BALANCE{address}` | 32 bytes (U256 LE) | Balance for address |
| `ALLOWANCE{owner}{spender}` | 32 bytes (U256 LE) | Allowance |
| `OWNER` | raw string bytes | Contract owner |

## Deployed on Buildnet

**Contract Address**: `AS1pZsJGd49trYhTGo4cDfpMruLDebgQ3YFcLHFaaZ7EsKg3YN26`

- Token Name: MassaRustToken
- Token Symbol: MRT
- Decimals: 18
- Initial Supply: 10^24 (1,000,000 tokens with 18 decimals)
- Owner with tokens: `AU1VCcJHYjR2cnQ3yembLo7dESX9x14esJxL1qzPJg2Shm7FV3MG`

## Project Structure

```
.
├── Cargo.toml                      # Workspace configuration
├── .cargo/config.toml              # WASM build configuration
├── contracts/
│   └── erc20-token/
│       ├── Cargo.toml
│       └── src/lib.rs              # MRC20 contract implementation
├── tests/
│   └── erc20-tests/
│       ├── Cargo.toml
│       └── src/lib.rs              # 13 comprehensive tests
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

```bash
cargo build -p erc20-token --release --target wasm32v1-none
```

The compiled WASM will be in `target/wasm32v1-none/release/erc20_token.wasm`.

## Testing

```bash
# Build the contract first
cargo build -p erc20-token --release --target wasm32v1-none

# Run all tests
cargo test -p erc20-tests -- --nocapture
```

## Contract Interface

### Constructor
```
constructor(name: string, symbol: string, decimals: u8, totalSupply: U256)
```
Initializes the token. The caller becomes the owner and receives the initial supply.

### Token Attributes (read-only, return raw bytes)
- `version()` → bytes ("0.0.1")
- `name()` → bytes (token name)
- `symbol()` → bytes (token symbol)
- `decimals()` → bytes ([u8])
- `totalSupply()` → bytes (U256, 32 bytes LE)
- `balanceOf(address: string)` → bytes (U256, 32 bytes LE)
- `allowance(owner: string, spender: string)` → bytes (U256, 32 bytes LE)

### Transfer Functions
- `transfer(to: string, amount: U256)` → emits `TRANSFER SUCCESS`
- `transferFrom(owner: string, recipient: string, amount: U256)` → emits `TRANSFER SUCCESS`

### Allowance Functions
- `increaseAllowance(spender: string, amount: U256)` → emits `APPROVAL SUCCESS`
- `decreaseAllowance(spender: string, amount: U256)` → emits `APPROVAL SUCCESS`

### Mintable (owner only)
- `mint(recipient: string, amount: U256)` → emits `MINT SUCCESS`

### Burnable
- `burn(amount: U256)` → emits `BURN_SUCCESS`
- `burnFrom(owner: string, amount: U256)` → emits `BURN_SUCCESS`

### Ownership
- `setOwner(newOwner: string)` → emits `CHANGE_OWNER:newOwner`
- `ownerAddress()` → bytes (owner address)
- `isOwner(address: string)` → bytes ([0] or [1])

## U256 Type

The contract uses the proper `U256` type from `massa-types` crate which provides:

- Full 256-bit arithmetic with safe operations (`checked_add`, `checked_sub`, etc.)
- Saturating arithmetic (`saturating_add`, `saturating_sub`)
- Little-endian byte serialization compatible with AssemblyScript's `as-bignum`
- Integration with `Args` for serialization (`add_u256`, `next_u256`)

Example usage:
```rust
use massa_sc_sdk::{Args, U256};

let a = U256::from(1_000_000u64);
let b = U256::from(2_000_000u64);
let sum = a.checked_add(b).expect("overflow");

// With Args
let mut args = Args::new();
args.add_u256(sum);
```

## License

MIT OR Apache-2.0

## Original prompt

Single prompt used to build this:

```
use this tools  https://github.com/damip/vibe_massa_rust_sdk (feel free to clone it separately as a temporary reference, delete it after, don' build the project in there)
Check massa docs: https://docs.massa.net

Create a new project,
Implement an ERC20 contract in rust, test it, deploy it on buildnet, call some of its functions directly on buildnet to attempt a coin transfer
```
