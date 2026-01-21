# Massa MRC20 Token Contract (Rust)

This project implements a fully MRC20-compatible fungible token smart contract in Rust for the Massa blockchain using the [vibe_massa_rust_sdk](https://github.com/damip/vibe_massa_rust_sdk).

## MRC20 Compatibility

This contract is **fully compatible** with the [MRC20 standard](https://github.com/massalabs/massa-standards) (Massa's ERC20 equivalent):

- **Storage format**: Identical key/value encoding as AssemblyScript implementation
- **Function signatures**: Same prototypes, arguments, and return values
- **Events**: Same event names and formats (`TRANSFER SUCCESS`, `APPROVAL SUCCESS`, `MINT SUCCESS`, `BURN_SUCCESS`, `CHANGE_OWNER`)
- **Deployer**: Compatible with Massa's standard deployment pipeline
- **u256 amounts**: Uses 256-bit integers for all token amounts (32-byte little-endian)

### Storage Keys (matching AS implementation)
| Key | Format | Description |
|-----|--------|-------------|
| `NAME` | raw bytes | Token name |
| `SYMBOL` | raw bytes | Token symbol |
| `DECIMALS` | 1 byte | Number of decimals |
| `TOTAL_SUPPLY` | 32 bytes (u256 LE) | Total supply |
| `BALANCE{address}` | 32 bytes (u256 LE) | Balance for address |
| `ALLOWANCE{owner}{spender}` | 32 bytes (u256 LE) | Allowance |
| `OWNER` | raw string bytes | Contract owner |

## Deployed on Buildnet

**Contract Address**: `AS12EL3KM4JT5tJsJSWZRqWj11kafR2sCyK9RXAotbuTHmJrdjTHc`

- Token Name: MassaTestToken
- Token Symbol: MTT
- Decimals: 18
- Initial Supply: 10^24 (1,000,000 tokens with 18 decimals)
- Owner: `AU1VCcJHYjR2cnQ3yembLo7dESX9x14esJxL1qzPJg2Shm7FV3MG`

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

```bash
cargo build -p erc20-token --release --target wasm32v1-none
```

The compiled WASM will be in `target/wasm32v1-none/release/erc20_token.wasm`.

## Contract Interface

### Constructor
```
constructor(name: string, symbol: string, decimals: u8, totalSupply: u256)
```
Initializes the token. The caller becomes the owner and receives the initial supply.

### Token Attributes (read-only, return raw bytes)
- `version()` → bytes ("0.0.1")
- `name()` → bytes (token name)
- `symbol()` → bytes (token symbol)
- `decimals()` → bytes ([u8])
- `totalSupply()` → bytes (u256, 32 bytes LE)
- `balanceOf(address: string)` → bytes (u256, 32 bytes LE)
- `allowance(owner: string, spender: string)` → bytes (u256, 32 bytes LE)

### Transfer Functions
- `transfer(to: string, amount: u256)` → emits `TRANSFER SUCCESS`
- `transferFrom(owner: string, recipient: string, amount: u256)` → emits `TRANSFER SUCCESS`

### Allowance Functions
- `increaseAllowance(spender: string, amount: u256)` → emits `APPROVAL SUCCESS`
- `decreaseAllowance(spender: string, amount: u256)` → emits `APPROVAL SUCCESS`

### Mintable (owner only)
- `mint(recipient: string, amount: u256)` → emits `MINT SUCCESS`

### Burnable
- `burn(amount: u256)` → emits `BURN_SUCCESS`
- `burnFrom(owner: string, amount: u256)` → emits `BURN_SUCCESS`

### Ownership
- `setOwner(newOwner: string)` → emits `CHANGE_OWNER:newOwner`
- `ownerAddress()` → bytes (owner address)
- `isOwner(address: string)` → bytes ([0] or [1])

## Deploying to Buildnet

### 1. Prepare constructor arguments

Create a file with Args-encoded constructor parameters:
- name (string)
- symbol (string)
- decimals (u8)
- totalSupply (u256 as length-prefixed bytes)

### 2. Deploy

```bash
cargo run -p massa-cli -- deploy \
  --rpc https://buildnet.massa.net/api/v2 \
  --private-key <YOUR_PRIVATE_KEY> \
  --bytecode target/wasm32v1-none/release/erc20_token.wasm \
  --args-file ./constructor.args \
  --coins 1 \
  --wait-final
```

### 3. Interact

```bash
# Transfer tokens
cargo run -p massa-cli -- call \
  --rpc https://buildnet.massa.net/api/v2 \
  --private-key <YOUR_PRIVATE_KEY> \
  --target <CONTRACT_ADDRESS> \
  --function transfer \
  --args-file ./transfer.args \
  --coins 0
```

## u256 Encoding

Token amounts use 256-bit unsigned integers encoded as 32-byte little-endian arrays, compatible with AssemblyScript's `as-bignum` library.

Example in Python:
```python
# Encode 1 million tokens (with 18 decimals) as u256
amount = 10**24  # 1,000,000 * 10^18
u256_bytes = amount.to_bytes(32, byteorder='little')
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

Then made MRC20-compatible with:
```
now let's make it fully compatible with the MRC20 (fungible token) spec that is here: https://github.com/massalabs/massa-standards
```
