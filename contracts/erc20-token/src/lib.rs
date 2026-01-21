//! MRC20 Fungible Token Implementation for Massa Blockchain
//!
//! This contract implements the MRC20 standard (Massa's ERC20 equivalent) with full
//! compatibility with the AssemblyScript reference implementation from massa-standards.
//!
//! # Compatibility
//! - Storage format matches AS implementation exactly
//! - Function signatures match AS implementation
//! - Event names and formats match AS implementation
//! - Can be deployed using the same deployer as AS contracts
//!
//! # Storage Keys
//! - `NAME`: Token name as raw bytes
//! - `SYMBOL`: Token symbol as raw bytes
//! - `DECIMALS`: Single byte [u8]
//! - `TOTAL_SUPPLY`: u256 as 32 bytes (little-endian)
//! - `BALANCE{address}`: Balance for address, value is u256
//! - `ALLOWANCE{owner}{spender}`: Allowance, value is u256
//! - `OWNER`: Owner address as raw string bytes

#![no_std]

extern crate alloc;

use alloc::string::String;
use alloc::vec::Vec;
use massa_export::massa_export;
use massa_sc_sdk::{abi, context, storage, Args};

// ============================================================================
// U256 Implementation (little-endian, compatible with AS as-bignum)
// ============================================================================

/// 256-bit unsigned integer, little-endian byte order
/// Compatible with AssemblyScript's as-bignum u256
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct U256(pub [u8; 32]);

impl U256 {
    pub const ZERO: U256 = U256([0u8; 32]);
    pub const MAX: U256 = U256([0xFF; 32]);

    pub fn from_le_bytes(bytes: [u8; 32]) -> Self {
        Self(bytes)
    }

    pub fn to_le_bytes(self) -> [u8; 32] {
        self.0
    }

    pub fn from_u64(value: u64) -> Self {
        let mut bytes = [0u8; 32];
        bytes[..8].copy_from_slice(&value.to_le_bytes());
        Self(bytes)
    }

    /// Checked addition, returns None on overflow
    pub fn checked_add(self, other: U256) -> Option<U256> {
        let mut result = [0u8; 32];
        let mut carry: u16 = 0;
        for i in 0..32 {
            let sum = self.0[i] as u16 + other.0[i] as u16 + carry;
            result[i] = sum as u8;
            carry = sum >> 8;
        }
        if carry != 0 {
            None // Overflow
        } else {
            Some(U256(result))
        }
    }

    /// Checked subtraction, returns None on underflow
    pub fn checked_sub(self, other: U256) -> Option<U256> {
        if self < other {
            return None;
        }
        let mut result = [0u8; 32];
        let mut borrow: i16 = 0;
        for i in 0..32 {
            let diff = self.0[i] as i16 - other.0[i] as i16 - borrow;
            if diff < 0 {
                result[i] = (diff + 256) as u8;
                borrow = 1;
            } else {
                result[i] = diff as u8;
                borrow = 0;
            }
        }
        Some(U256(result))
    }

    pub fn is_zero(&self) -> bool {
        self.0.iter().all(|&b| b == 0)
    }
}

impl PartialOrd for U256 {
    fn partial_cmp(&self, other: &Self) -> Option<core::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for U256 {
    fn cmp(&self, other: &Self) -> core::cmp::Ordering {
        // Compare from most significant byte to least significant
        for i in (0..32).rev() {
            match self.0[i].cmp(&other.0[i]) {
                core::cmp::Ordering::Equal => continue,
                other => return other,
            }
        }
        core::cmp::Ordering::Equal
    }
}

// ============================================================================
// Constants - Storage Keys (matching AS implementation exactly)
// ============================================================================

const VERSION: &[u8] = b"0.0.1";
const NAME_KEY: &[u8] = b"NAME";
const SYMBOL_KEY: &[u8] = b"SYMBOL";
const DECIMALS_KEY: &[u8] = b"DECIMALS";
const TOTAL_SUPPLY_KEY: &[u8] = b"TOTAL_SUPPLY";
const BALANCE_KEY_PREFIX: &[u8] = b"BALANCE";
const ALLOWANCE_KEY_PREFIX: &[u8] = b"ALLOWANCE";
const OWNER_KEY: &[u8] = b"OWNER";

// Event names (matching AS implementation exactly)
const TRANSFER_EVENT: &str = "TRANSFER SUCCESS";
const APPROVAL_EVENT: &str = "APPROVAL SUCCESS";
const MINT_EVENT: &str = "MINT SUCCESS";
const BURN_EVENT: &str = "BURN_SUCCESS";
const CHANGE_OWNER_EVENT: &str = "CHANGE_OWNER";

// ============================================================================
// Storage Key Builders
// ============================================================================

/// Build balance key: "BALANCE" + address
fn balance_key(address: &str) -> Vec<u8> {
    let mut key = BALANCE_KEY_PREFIX.to_vec();
    key.extend_from_slice(address.as_bytes());
    key
}

/// Build allowance key: "ALLOWANCE" + owner + spender
fn allowance_key(owner: &str, spender: &str) -> Vec<u8> {
    let mut key = ALLOWANCE_KEY_PREFIX.to_vec();
    key.extend_from_slice(owner.as_bytes());
    key.extend_from_slice(spender.as_bytes());
    key
}

// ============================================================================
// Internal Storage Helpers
// ============================================================================

fn get_balance(address: &str) -> U256 {
    let key = balance_key(address);
    if !storage::has(&key) {
        return U256::ZERO;
    }
    let data = storage::get(&key);
    if data.len() >= 32 {
        let mut bytes = [0u8; 32];
        bytes.copy_from_slice(&data[..32]);
        U256::from_le_bytes(bytes)
    } else {
        U256::ZERO
    }
}

fn set_balance(address: &str, amount: U256) {
    let key = balance_key(address);
    storage::set(&key, &amount.to_le_bytes());
}

fn get_allowance(owner: &str, spender: &str) -> U256 {
    let key = allowance_key(owner, spender);
    if !storage::has(&key) {
        return U256::ZERO;
    }
    let data = storage::get(&key);
    if data.len() >= 32 {
        let mut bytes = [0u8; 32];
        bytes.copy_from_slice(&data[..32]);
        U256::from_le_bytes(bytes)
    } else {
        U256::ZERO
    }
}

fn set_allowance(owner: &str, spender: &str, amount: U256) {
    let key = allowance_key(owner, spender);
    storage::set(&key, &amount.to_le_bytes());
}

fn get_total_supply() -> U256 {
    if !storage::has(TOTAL_SUPPLY_KEY) {
        return U256::ZERO;
    }
    let data = storage::get(TOTAL_SUPPLY_KEY);
    if data.len() >= 32 {
        let mut bytes = [0u8; 32];
        bytes.copy_from_slice(&data[..32]);
        U256::from_le_bytes(bytes)
    } else {
        U256::ZERO
    }
}

fn set_total_supply(amount: U256) {
    storage::set(TOTAL_SUPPLY_KEY, &amount.to_le_bytes());
}

fn get_owner() -> Option<String> {
    if !storage::has(OWNER_KEY) {
        return None;
    }
    let data = storage::get(OWNER_KEY);
    core::str::from_utf8(&data).ok().map(|s| String::from(s))
}

fn set_owner_internal(owner: &str) {
    storage::set(OWNER_KEY, owner.as_bytes());
}

fn only_owner() {
    let owner = get_owner();
    assert!(owner.is_some(), "Owner is not set");
    let caller = context::caller();
    assert!(
        caller == owner.unwrap(),
        "Caller is not the owner"
    );
}

fn is_owner_check(address: &str) -> bool {
    match get_owner() {
        Some(owner) => owner == address,
        None => false,
    }
}

// ============================================================================
// Constructor
// ============================================================================

/// Constructor - Initialize the MRC20 token.
///
/// # Arguments (Args serialized)
/// - `name`: Token name (string)
/// - `symbol`: Token symbol (string)
/// - `decimals`: Token decimals (u8)
/// - `totalSupply`: Initial supply as u256 (32 bytes)
///
/// The caller becomes the owner and receives all initial tokens.
#[massa_export]
pub fn constructor(binary_args: &[u8]) -> Vec<u8> {
    assert!(context::is_deploying_contract(), "Can only be called during deployment");

    let mut args = Args::from_bytes(binary_args.to_vec());
    let name = args.next_string().unwrap_or_else(|_| String::from("MassaToken"));
    let symbol = args.next_string().unwrap_or_else(|_| String::from("MT"));
    let decimals = args.next_u8().unwrap_or(18);
    
    // Read u256 as 32 bytes
    let total_supply = if let Ok(bytes) = args.next_bytes() {
        if bytes.len() >= 32 {
            let mut arr = [0u8; 32];
            arr.copy_from_slice(&bytes[..32]);
            U256::from_le_bytes(arr)
        } else {
            U256::from_u64(1_000_000_000_000_000_000) // Default 1 token with 18 decimals
        }
    } else {
        U256::from_u64(1_000_000_000_000_000_000)
    };

    // Store token metadata (raw bytes, matching AS format)
    storage::set(NAME_KEY, name.as_bytes());
    storage::set(SYMBOL_KEY, symbol.as_bytes());
    storage::set(DECIMALS_KEY, &[decimals]);
    set_total_supply(total_supply);

    // Set owner and mint initial supply to caller
    let caller = context::caller();
    set_owner_internal(&caller);
    set_balance(&caller, total_supply);

    // Emit CHANGE_OWNER event (matching AS format: "CHANGE_OWNER:address")
    abi::generate_event(&alloc::format!("{}:{}", CHANGE_OWNER_EVENT, caller));

    Vec::new()
}

// ============================================================================
// Token Attributes (read-only)
// ============================================================================

/// Returns the version of this smart contract.
#[massa_export]
pub fn version(_binary_args: &[u8]) -> Vec<u8> {
    VERSION.to_vec()
}

/// Returns the name of the token (raw bytes, not Args-wrapped).
#[massa_export]
pub fn name(_binary_args: &[u8]) -> Vec<u8> {
    storage::get(NAME_KEY)
}

/// Returns the symbol of the token (raw bytes, not Args-wrapped).
#[massa_export]
pub fn symbol(_binary_args: &[u8]) -> Vec<u8> {
    storage::get(SYMBOL_KEY)
}

/// Returns the decimals of the token (raw bytes, not Args-wrapped).
#[massa_export]
pub fn decimals(_binary_args: &[u8]) -> Vec<u8> {
    storage::get(DECIMALS_KEY)
}

/// Returns the total supply (raw u256 bytes, not Args-wrapped).
#[massa_export]
pub fn totalSupply(_binary_args: &[u8]) -> Vec<u8> {
    storage::get(TOTAL_SUPPLY_KEY)
}

// ============================================================================
// Balance
// ============================================================================

/// Returns the balance of an account (u256 bytes).
///
/// # Arguments
/// - `address`: Account address (string)
#[massa_export]
pub fn balanceOf(binary_args: &[u8]) -> Vec<u8> {
    let mut args = Args::from_bytes(binary_args.to_vec());
    let address = args.next_string().expect("Address argument is missing or invalid");
    let balance = get_balance(&address);
    balance.to_le_bytes().to_vec()
}

// ============================================================================
// Transfer
// ============================================================================

/// Transfers tokens from caller to recipient.
///
/// # Arguments
/// - `to`: Recipient address (string)
/// - `amount`: Amount to transfer (u256 as bytes)
///
/// # Events
/// - `TRANSFER SUCCESS`
#[massa_export]
pub fn transfer(binary_args: &[u8]) -> Vec<u8> {
    let mut args = Args::from_bytes(binary_args.to_vec());
    let to = args.next_string().expect("receiverAddress argument is missing or invalid");
    let amount_bytes = args.next_bytes().expect("amount argument is missing or invalid");
    
    let amount = if amount_bytes.len() >= 32 {
        let mut arr = [0u8; 32];
        arr.copy_from_slice(&amount_bytes[..32]);
        U256::from_le_bytes(arr)
    } else {
        panic!("amount argument is missing or invalid");
    };

    let from = context::caller();
    
    assert!(from != to, "Transfer failed: cannot send tokens to own account");

    let from_balance = get_balance(&from);
    let to_balance = get_balance(&to);
    
    assert!(from_balance >= amount, "Transfer failed: insufficient funds");
    
    let new_to_balance = to_balance.checked_add(amount).expect("Transfer failed: overflow");
    let new_from_balance = from_balance.checked_sub(amount).unwrap();
    
    set_balance(&from, new_from_balance);
    set_balance(&to, new_to_balance);

    abi::generate_event(TRANSFER_EVENT);

    Vec::new()
}

// ============================================================================
// Allowance
// ============================================================================

/// Returns the allowance for owner/spender (u256 bytes).
///
/// # Arguments
/// - `owner`: Owner address (string)
/// - `spender`: Spender address (string)
#[massa_export]
pub fn allowance(binary_args: &[u8]) -> Vec<u8> {
    let mut args = Args::from_bytes(binary_args.to_vec());
    let owner = args.next_string().expect("owner argument is missing or invalid");
    let spender = args.next_string().expect("spenderAddress argument is missing or invalid");
    
    let amount = get_allowance(&owner, &spender);
    amount.to_le_bytes().to_vec()
}

/// Increases the allowance of the spender on the caller's account.
///
/// # Arguments
/// - `spender`: Spender address (string)
/// - `amount`: Amount to increase (u256 as bytes)
///
/// # Events
/// - `APPROVAL SUCCESS`
#[massa_export]
pub fn increaseAllowance(binary_args: &[u8]) -> Vec<u8> {
    let mut args = Args::from_bytes(binary_args.to_vec());
    let spender = args.next_string().expect("spenderAddress argument is missing or invalid");
    let amount_bytes = args.next_bytes().expect("amount argument is missing or invalid");
    
    let amount = if amount_bytes.len() >= 32 {
        let mut arr = [0u8; 32];
        arr.copy_from_slice(&amount_bytes[..32]);
        U256::from_le_bytes(arr)
    } else {
        panic!("amount argument is missing or invalid");
    };

    let owner = context::caller();
    let current = get_allowance(&owner, &spender);
    
    // If overflow, set to max
    let new_allowance = current.checked_add(amount).unwrap_or(U256::MAX);
    
    set_allowance(&owner, &spender, new_allowance);

    abi::generate_event(APPROVAL_EVENT);

    Vec::new()
}

/// Decreases the allowance of the spender on the caller's account.
///
/// # Arguments
/// - `spender`: Spender address (string)
/// - `amount`: Amount to decrease (u256 as bytes)
///
/// # Events
/// - `APPROVAL SUCCESS`
#[massa_export]
pub fn decreaseAllowance(binary_args: &[u8]) -> Vec<u8> {
    let mut args = Args::from_bytes(binary_args.to_vec());
    let spender = args.next_string().expect("spenderAddress argument is missing or invalid");
    let amount_bytes = args.next_bytes().expect("amount argument is missing or invalid");
    
    let amount = if amount_bytes.len() >= 32 {
        let mut arr = [0u8; 32];
        arr.copy_from_slice(&amount_bytes[..32]);
        U256::from_le_bytes(arr)
    } else {
        panic!("amount argument is missing or invalid");
    };

    let owner = context::caller();
    let current = get_allowance(&owner, &spender);
    
    // If underflow, set to zero
    let new_allowance = if current > amount {
        current.checked_sub(amount).unwrap()
    } else {
        U256::ZERO
    };
    
    set_allowance(&owner, &spender, new_allowance);

    abi::generate_event(APPROVAL_EVENT);

    Vec::new()
}

/// Transfers tokens from owner to recipient using spender's allowance.
///
/// # Arguments
/// - `owner`: Owner address (string)
/// - `recipient`: Recipient address (string)
/// - `amount`: Amount to transfer (u256 as bytes)
///
/// # Events
/// - `TRANSFER SUCCESS`
#[massa_export]
pub fn transferFrom(binary_args: &[u8]) -> Vec<u8> {
    let mut args = Args::from_bytes(binary_args.to_vec());
    let owner = args.next_string().expect("ownerAddress argument is missing or invalid");
    let recipient = args.next_string().expect("recipientAddress argument is missing or invalid");
    let amount_bytes = args.next_bytes().expect("amount argument is missing or invalid");
    
    let amount = if amount_bytes.len() >= 32 {
        let mut arr = [0u8; 32];
        arr.copy_from_slice(&amount_bytes[..32]);
        U256::from_le_bytes(arr)
    } else {
        panic!("amount argument is missing or invalid");
    };

    let spender = context::caller();
    
    assert!(owner != recipient, "Transfer failed: cannot send tokens to own account");
    
    // Check allowance
    let spender_allowance = get_allowance(&owner, &spender);
    assert!(spender_allowance >= amount, "transferFrom failed: insufficient allowance");
    
    // Check balance
    let owner_balance = get_balance(&owner);
    let recipient_balance = get_balance(&recipient);
    
    assert!(owner_balance >= amount, "Transfer failed: insufficient funds");
    
    let new_recipient_balance = recipient_balance.checked_add(amount).expect("Transfer failed: overflow");
    let new_owner_balance = owner_balance.checked_sub(amount).unwrap();
    let new_allowance = spender_allowance.checked_sub(amount).unwrap();
    
    set_balance(&owner, new_owner_balance);
    set_balance(&recipient, new_recipient_balance);
    set_allowance(&owner, &spender, new_allowance);

    abi::generate_event(TRANSFER_EVENT);

    Vec::new()
}

// ============================================================================
// Mintable (owner only)
// ============================================================================

/// Mint tokens to recipient (owner only).
///
/// # Arguments
/// - `recipient`: Recipient address (string)
/// - `amount`: Amount to mint (u256 as bytes)
///
/// # Events
/// - `MINT SUCCESS`
#[massa_export]
pub fn mint(binary_args: &[u8]) -> Vec<u8> {
    only_owner();
    
    let mut args = Args::from_bytes(binary_args.to_vec());
    let recipient = args.next_string().expect("recipient argument is missing or invalid");
    let amount_bytes = args.next_bytes().expect("amount argument is missing or invalid");
    
    let amount = if amount_bytes.len() >= 32 {
        let mut arr = [0u8; 32];
        arr.copy_from_slice(&amount_bytes[..32]);
        U256::from_le_bytes(arr)
    } else {
        panic!("amount argument is missing or invalid");
    };

    // Increase total supply
    let old_supply = get_total_supply();
    let new_supply = old_supply.checked_add(amount).expect("Requested mint amount causes an overflow");
    set_total_supply(new_supply);
    
    // Increase recipient balance
    let old_balance = get_balance(&recipient);
    let new_balance = old_balance.checked_add(amount).expect("Requested mint amount causes an overflow");
    set_balance(&recipient, new_balance);

    abi::generate_event(MINT_EVENT);

    Vec::new()
}

// ============================================================================
// Burnable
// ============================================================================

/// Burn tokens from caller's balance.
///
/// # Arguments
/// - `amount`: Amount to burn (u256 as bytes)
///
/// # Events
/// - `BURN_SUCCESS`
#[massa_export]
pub fn burn(binary_args: &[u8]) -> Vec<u8> {
    let mut args = Args::from_bytes(binary_args.to_vec());
    let amount_bytes = args.next_bytes().expect("amount argument is missing or invalid");
    
    let amount = if amount_bytes.len() >= 32 {
        let mut arr = [0u8; 32];
        arr.copy_from_slice(&amount_bytes[..32]);
        U256::from_le_bytes(arr)
    } else {
        panic!("amount argument is missing or invalid");
    };

    let caller = context::caller();
    
    // Decrease total supply
    let old_supply = get_total_supply();
    let new_supply = old_supply.checked_sub(amount)
        .expect("Requested burn amount causes an underflow of the total supply");
    set_total_supply(new_supply);
    
    // Decrease caller balance
    let old_balance = get_balance(&caller);
    let new_balance = old_balance.checked_sub(amount)
        .expect("Requested burn amount causes an underflow of the recipient balance");
    set_balance(&caller, new_balance);

    abi::generate_event(BURN_EVENT);

    Vec::new()
}

/// Burn tokens from owner using spender's allowance.
///
/// # Arguments
/// - `owner`: Owner address (string)
/// - `amount`: Amount to burn (u256 as bytes)
///
/// # Events
/// - `BURN_SUCCESS`
#[massa_export]
pub fn burnFrom(binary_args: &[u8]) -> Vec<u8> {
    let mut args = Args::from_bytes(binary_args.to_vec());
    let owner = args.next_string().expect("owner argument is missing or invalid");
    let amount_bytes = args.next_bytes().expect("amount argument is missing or invalid");
    
    let amount = if amount_bytes.len() >= 32 {
        let mut arr = [0u8; 32];
        arr.copy_from_slice(&amount_bytes[..32]);
        U256::from_le_bytes(arr)
    } else {
        panic!("amount argument is missing or invalid");
    };

    let spender = context::caller();
    
    // Check allowance
    let spender_allowance = get_allowance(&owner, &spender);
    assert!(spender_allowance >= amount, "burnFrom failed: insufficient allowance");
    
    // Decrease total supply
    let old_supply = get_total_supply();
    let new_supply = old_supply.checked_sub(amount)
        .expect("Requested burn amount causes an underflow of the total supply");
    set_total_supply(new_supply);
    
    // Decrease owner balance
    let old_balance = get_balance(&owner);
    let new_balance = old_balance.checked_sub(amount)
        .expect("Requested burn amount causes an underflow of the recipient balance");
    set_balance(&owner, new_balance);
    
    // Decrease allowance
    let new_allowance = spender_allowance.checked_sub(amount).unwrap();
    set_allowance(&owner, &spender, new_allowance);

    abi::generate_event(BURN_EVENT);

    Vec::new()
}

// ============================================================================
// Ownership
// ============================================================================

/// Set the contract owner (only current owner can call, or anyone if no owner set).
///
/// # Arguments
/// - `newOwner`: New owner address (string)
///
/// # Events
/// - `CHANGE_OWNER:newOwner`
#[massa_export]
pub fn setOwner(binary_args: &[u8]) -> Vec<u8> {
    let mut args = Args::from_bytes(binary_args.to_vec());
    let new_owner = args.next_string().expect("newOwnerAddress argument is missing or invalid");
    
    // If owner exists, only owner can change
    if get_owner().is_some() {
        only_owner();
    }
    
    set_owner_internal(&new_owner);
    
    abi::generate_event(&alloc::format!("{}:{}", CHANGE_OWNER_EVENT, new_owner));

    Vec::new()
}

/// Returns the owner address (raw bytes).
#[massa_export]
pub fn ownerAddress(_binary_args: &[u8]) -> Vec<u8> {
    if !storage::has(OWNER_KEY) {
        return Vec::new();
    }
    storage::get(OWNER_KEY)
}

/// Returns true (1) if address is owner, false (0) otherwise.
///
/// # Arguments
/// - `address`: Address to check (string)
#[massa_export]
pub fn isOwner(binary_args: &[u8]) -> Vec<u8> {
    if !storage::has(OWNER_KEY) {
        return alloc::vec![0u8];
    }
    let mut args = Args::from_bytes(binary_args.to_vec());
    let address = args.next_string().expect("address argument is missing or invalid");
    
    if is_owner_check(&address) {
        alloc::vec![1u8]
    } else {
        alloc::vec![0u8]
    }
}
