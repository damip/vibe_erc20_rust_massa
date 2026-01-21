//! ERC20 Token Implementation for Massa Blockchain
//!
//! This contract implements a standard ERC20-like token with the following features:
//! - Token metadata (name, symbol, decimals)
//! - Balance tracking per address
//! - Transfer functionality
//! - Approval and allowance mechanism for delegated transfers
//!
//! # Storage Keys
//! - `NAME`: Token name (string)
//! - `SYMBOL`: Token symbol (string)
//! - `DECIMALS`: Token decimals (u8)
//! - `TOTAL_SUPPLY`: Total supply (u64)
//! - `BALANCE:{address}`: Balance of address (u64)
//! - `ALLOWANCE:{owner}:{spender}`: Allowance amount (u64)

#![no_std]

extern crate alloc;

use alloc::format;
use alloc::string::String;
use alloc::vec::Vec;
use massa_export::massa_export;
use massa_sc_sdk::{abi, context, storage, Args};

// Storage key prefixes
const KEY_NAME: &[u8] = b"NAME";
const KEY_SYMBOL: &[u8] = b"SYMBOL";
const KEY_DECIMALS: &[u8] = b"DECIMALS";
const KEY_TOTAL_SUPPLY: &[u8] = b"TOTAL_SUPPLY";
const KEY_BALANCE_PREFIX: &[u8] = b"BALANCE:";
const KEY_ALLOWANCE_PREFIX: &[u8] = b"ALLOWANCE:";

/// Build the storage key for a balance entry.
fn balance_key(address: &str) -> Vec<u8> {
    let mut key = KEY_BALANCE_PREFIX.to_vec();
    key.extend_from_slice(address.as_bytes());
    key
}

/// Build the storage key for an allowance entry.
fn allowance_key(owner: &str, spender: &str) -> Vec<u8> {
    let mut key = KEY_ALLOWANCE_PREFIX.to_vec();
    key.extend_from_slice(owner.as_bytes());
    key.push(b':');
    key.extend_from_slice(spender.as_bytes());
    key
}

/// Get balance for an address from storage.
fn get_balance_internal(address: &str) -> u64 {
    let key = balance_key(address);
    if !storage::has(&key) {
        return 0;
    }
    let data = storage::get(&key);
    if data.len() >= 8 {
        u64::from_le_bytes([
            data[0], data[1], data[2], data[3], data[4], data[5], data[6], data[7],
        ])
    } else {
        0
    }
}

/// Set balance for an address in storage.
fn set_balance_internal(address: &str, amount: u64) {
    let key = balance_key(address);
    storage::set(&key, &amount.to_le_bytes());
}

/// Get allowance for owner/spender pair from storage.
fn get_allowance_internal(owner: &str, spender: &str) -> u64 {
    let key = allowance_key(owner, spender);
    if !storage::has(&key) {
        return 0;
    }
    let data = storage::get(&key);
    if data.len() >= 8 {
        u64::from_le_bytes([
            data[0], data[1], data[2], data[3], data[4], data[5], data[6], data[7],
        ])
    } else {
        0
    }
}

/// Set allowance for owner/spender pair in storage.
fn set_allowance_internal(owner: &str, spender: &str, amount: u64) {
    let key = allowance_key(owner, spender);
    storage::set(&key, &amount.to_le_bytes());
}

/// Constructor - Initialize the ERC20 token.
///
/// # Arguments
/// - `name`: Token name (string)
/// - `symbol`: Token symbol (string)
/// - `decimals`: Token decimals (u32, typically 18)
/// - `initial_supply`: Initial supply to mint to deployer (u64)
///
/// # Events
/// - `ERC20_DEPLOYED:{name}:{symbol}:{decimals}:{initial_supply}:{deployer}`
/// - `TRANSFER:0x0:{deployer}:{initial_supply}` (mint event)
#[massa_export]
pub fn constructor(binary_args: &[u8]) -> Vec<u8> {
    // Only allow during deployment
    if !context::is_deploying_contract() {
        abi::generate_event("ERROR:constructor can only be called during deployment");
        return Vec::new();
    }

    let mut args = Args::from_bytes(binary_args.to_vec());
    let name = args.next_string().unwrap_or_else(|_| String::from("MassaToken"));
    let symbol = args.next_string().unwrap_or_else(|_| String::from("MTK"));
    let decimals = args.next_u32().unwrap_or(18) as u8;  // Read as u32 for CLI compatibility, cast to u8
    let initial_supply = args.next_u64().unwrap_or(1_000_000_000_000_000_000); // 1 token with 18 decimals

    // Store token metadata
    storage::set(KEY_NAME, name.as_bytes());
    storage::set(KEY_SYMBOL, symbol.as_bytes());
    storage::set(KEY_DECIMALS, &[decimals]);
    storage::set(KEY_TOTAL_SUPPLY, &initial_supply.to_le_bytes());

    // Get deployer address (caller during deployment)
    let deployer = context::caller();

    // Mint initial supply to deployer
    set_balance_internal(&deployer, initial_supply);

    abi::generate_event(&format!(
        "ERC20_DEPLOYED:{}:{}:{}:{}:{}",
        name, symbol, decimals, initial_supply, deployer
    ));

    abi::generate_event(&format!("TRANSFER:0x0:{}:{}", deployer, initial_supply));

    Args::new().into_bytes()
}

/// Get token name.
///
/// # Returns
/// - Token name as string
#[massa_export]
pub fn name(_binary_args: &[u8]) -> Vec<u8> {
    let data = storage::get(KEY_NAME);
    let name = core::str::from_utf8(&data).unwrap_or("Unknown");
    let mut out = Args::new();
    out.add_string(name);
    out.into_bytes()
}

/// Get token symbol.
///
/// # Returns
/// - Token symbol as string
#[massa_export]
pub fn symbol(_binary_args: &[u8]) -> Vec<u8> {
    let data = storage::get(KEY_SYMBOL);
    let symbol = core::str::from_utf8(&data).unwrap_or("???");
    let mut out = Args::new();
    out.add_string(symbol);
    out.into_bytes()
}

/// Get token decimals.
///
/// # Returns
/// - Token decimals as u8
#[massa_export]
pub fn decimals(_binary_args: &[u8]) -> Vec<u8> {
    let data = storage::get(KEY_DECIMALS);
    let decimals = if data.is_empty() { 18 } else { data[0] };
    let mut out = Args::new();
    out.add_u8(decimals);
    out.into_bytes()
}

/// Get total supply.
///
/// # Returns
/// - Total supply as u64
#[massa_export]
pub fn totalSupply(_binary_args: &[u8]) -> Vec<u8> {
    let data = storage::get(KEY_TOTAL_SUPPLY);
    let total = if data.len() >= 8 {
        u64::from_le_bytes([
            data[0], data[1], data[2], data[3], data[4], data[5], data[6], data[7],
        ])
    } else {
        0
    };
    let mut out = Args::new();
    out.add_u64(total);
    out.into_bytes()
}

/// Get balance of an address.
///
/// # Arguments
/// - `address`: Address to query balance for (string)
///
/// # Returns
/// - Balance as u64
#[massa_export]
pub fn balanceOf(binary_args: &[u8]) -> Vec<u8> {
    let mut args = Args::from_bytes(binary_args.to_vec());
    let address = args.next_string().unwrap_or_else(|_| String::new());

    let balance = get_balance_internal(&address);

    let mut out = Args::new();
    out.add_u64(balance);
    out.into_bytes()
}

/// Transfer tokens from caller to recipient.
///
/// # Arguments
/// - `to`: Recipient address (string)
/// - `amount`: Amount to transfer (u64)
///
/// # Returns
/// - Success flag as bool
///
/// # Events
/// - `TRANSFER:{from}:{to}:{amount}` on success
/// - `ERROR:...` on failure
#[massa_export]
pub fn transfer(binary_args: &[u8]) -> Vec<u8> {
    let mut args = Args::from_bytes(binary_args.to_vec());
    let to = args.next_string().unwrap_or_else(|_| String::new());
    let amount = args.next_u64().unwrap_or(0);

    let from = context::caller();

    // Validate inputs
    if to.is_empty() {
        abi::generate_event("ERROR:transfer to zero address");
        let mut out = Args::new();
        out.add_bool(false);
        return out.into_bytes();
    }

    let from_balance = get_balance_internal(&from);
    if from_balance < amount {
        abi::generate_event(&format!(
            "ERROR:insufficient balance: has {} needs {}",
            from_balance, amount
        ));
        let mut out = Args::new();
        out.add_bool(false);
        return out.into_bytes();
    }

    // Perform transfer
    set_balance_internal(&from, from_balance - amount);
    let to_balance = get_balance_internal(&to);
    set_balance_internal(&to, to_balance + amount);

    abi::generate_event(&format!("TRANSFER:{}:{}:{}", from, to, amount));

    let mut out = Args::new();
    out.add_bool(true);
    out.into_bytes()
}

/// Approve spender to spend tokens on behalf of caller.
///
/// # Arguments
/// - `spender`: Spender address (string)
/// - `amount`: Amount to approve (u64)
///
/// # Returns
/// - Success flag as bool
///
/// # Events
/// - `APPROVAL:{owner}:{spender}:{amount}`
#[massa_export]
pub fn approve(binary_args: &[u8]) -> Vec<u8> {
    let mut args = Args::from_bytes(binary_args.to_vec());
    let spender = args.next_string().unwrap_or_else(|_| String::new());
    let amount = args.next_u64().unwrap_or(0);

    let owner = context::caller();

    if spender.is_empty() {
        abi::generate_event("ERROR:approve to zero address");
        let mut out = Args::new();
        out.add_bool(false);
        return out.into_bytes();
    }

    set_allowance_internal(&owner, &spender, amount);

    abi::generate_event(&format!("APPROVAL:{}:{}:{}", owner, spender, amount));

    let mut out = Args::new();
    out.add_bool(true);
    out.into_bytes()
}

/// Get allowance for owner/spender pair.
///
/// # Arguments
/// - `owner`: Owner address (string)
/// - `spender`: Spender address (string)
///
/// # Returns
/// - Allowance amount as u64
#[massa_export]
pub fn allowance(binary_args: &[u8]) -> Vec<u8> {
    let mut args = Args::from_bytes(binary_args.to_vec());
    let owner = args.next_string().unwrap_or_else(|_| String::new());
    let spender = args.next_string().unwrap_or_else(|_| String::new());

    let amount = get_allowance_internal(&owner, &spender);

    let mut out = Args::new();
    out.add_u64(amount);
    out.into_bytes()
}

/// Transfer tokens from one address to another using allowance.
///
/// # Arguments
/// - `from`: Source address (string)
/// - `to`: Destination address (string)
/// - `amount`: Amount to transfer (u64)
///
/// # Returns
/// - Success flag as bool
///
/// # Events
/// - `TRANSFER:{from}:{to}:{amount}` on success
/// - `ERROR:...` on failure
#[massa_export]
pub fn transferFrom(binary_args: &[u8]) -> Vec<u8> {
    let mut args = Args::from_bytes(binary_args.to_vec());
    let from = args.next_string().unwrap_or_else(|_| String::new());
    let to = args.next_string().unwrap_or_else(|_| String::new());
    let amount = args.next_u64().unwrap_or(0);

    let spender = context::caller();

    // Validate inputs
    if to.is_empty() {
        abi::generate_event("ERROR:transferFrom to zero address");
        let mut out = Args::new();
        out.add_bool(false);
        return out.into_bytes();
    }

    // Check allowance
    let current_allowance = get_allowance_internal(&from, &spender);
    if current_allowance < amount {
        abi::generate_event(&format!(
            "ERROR:insufficient allowance: has {} needs {}",
            current_allowance, amount
        ));
        let mut out = Args::new();
        out.add_bool(false);
        return out.into_bytes();
    }

    // Check balance
    let from_balance = get_balance_internal(&from);
    if from_balance < amount {
        abi::generate_event(&format!(
            "ERROR:insufficient balance: has {} needs {}",
            from_balance, amount
        ));
        let mut out = Args::new();
        out.add_bool(false);
        return out.into_bytes();
    }

    // Update allowance
    set_allowance_internal(&from, &spender, current_allowance - amount);

    // Perform transfer
    set_balance_internal(&from, from_balance - amount);
    let to_balance = get_balance_internal(&to);
    set_balance_internal(&to, to_balance + amount);

    abi::generate_event(&format!("TRANSFER:{}:{}:{}", from, to, amount));

    let mut out = Args::new();
    out.add_bool(true);
    out.into_bytes()
}

/// Mint new tokens (only callable by contract owner - in this demo, anyone can mint for testing).
/// In production, you'd add access control.
///
/// # Arguments
/// - `to`: Recipient address (string)
/// - `amount`: Amount to mint (u64)
///
/// # Returns
/// - Success flag as bool
///
/// # Events
/// - `TRANSFER:0x0:{to}:{amount}` (mint event)
#[massa_export]
pub fn mint(binary_args: &[u8]) -> Vec<u8> {
    let mut args = Args::from_bytes(binary_args.to_vec());
    let to = args.next_string().unwrap_or_else(|_| String::new());
    let amount = args.next_u64().unwrap_or(0);

    if to.is_empty() {
        abi::generate_event("ERROR:mint to zero address");
        let mut out = Args::new();
        out.add_bool(false);
        return out.into_bytes();
    }

    // Update total supply
    let supply_data = storage::get(KEY_TOTAL_SUPPLY);
    let current_supply = if supply_data.len() >= 8 {
        u64::from_le_bytes([
            supply_data[0],
            supply_data[1],
            supply_data[2],
            supply_data[3],
            supply_data[4],
            supply_data[5],
            supply_data[6],
            supply_data[7],
        ])
    } else {
        0
    };
    let new_supply = current_supply.saturating_add(amount);
    storage::set(KEY_TOTAL_SUPPLY, &new_supply.to_le_bytes());

    // Update recipient balance
    let to_balance = get_balance_internal(&to);
    set_balance_internal(&to, to_balance.saturating_add(amount));

    abi::generate_event(&format!("TRANSFER:0x0:{}:{}", to, amount));

    let mut out = Args::new();
    out.add_bool(true);
    out.into_bytes()
}

/// Burn tokens from caller's balance.
///
/// # Arguments
/// - `amount`: Amount to burn (u64)
///
/// # Returns
/// - Success flag as bool
///
/// # Events
/// - `TRANSFER:{from}:0x0:{amount}` (burn event)
#[massa_export]
pub fn burn(binary_args: &[u8]) -> Vec<u8> {
    let mut args = Args::from_bytes(binary_args.to_vec());
    let amount = args.next_u64().unwrap_or(0);

    let from = context::caller();

    let from_balance = get_balance_internal(&from);
    if from_balance < amount {
        abi::generate_event(&format!(
            "ERROR:insufficient balance to burn: has {} needs {}",
            from_balance, amount
        ));
        let mut out = Args::new();
        out.add_bool(false);
        return out.into_bytes();
    }

    // Update balance
    set_balance_internal(&from, from_balance - amount);

    // Update total supply
    let supply_data = storage::get(KEY_TOTAL_SUPPLY);
    let current_supply = if supply_data.len() >= 8 {
        u64::from_le_bytes([
            supply_data[0],
            supply_data[1],
            supply_data[2],
            supply_data[3],
            supply_data[4],
            supply_data[5],
            supply_data[6],
            supply_data[7],
        ])
    } else {
        0
    };
    let new_supply = current_supply.saturating_sub(amount);
    storage::set(KEY_TOTAL_SUPPLY, &new_supply.to_le_bytes());

    abi::generate_event(&format!("TRANSFER:{}:0x0:{}", from, amount));

    let mut out = Args::new();
    out.add_bool(true);
    out.into_bytes()
}
