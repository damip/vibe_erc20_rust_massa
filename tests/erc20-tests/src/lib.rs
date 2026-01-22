//! Tests for the MRC20 Token Contract
//!
//! This test suite validates the MRC20 contract functionality using
//! the massa-testkit runtime with proper U256 arithmetic.

use anyhow::Result;
use massa_types::{Args, U256};
use massa_testkit::{TestInterface, TestRuntime};

/// Test addresses for simulating different users
const DEPLOYER: &str = "AU1deployerAddress123456789012345678901234567890";
const ALICE: &str = "AU1aliceAddress1234567890123456789012345678901234";
const BOB: &str = "AU1bobAddress12345678901234567890123456789012345";
const CHARLIE: &str = "AU1charlieAddress12345678901234567890123456789012";

/// Helper to build WASM path
fn wasm_path() -> std::path::PathBuf {
    std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../target/wasm32v1-none/release/erc20_token.wasm")
}

/// Helper to create constructor args with U256
fn constructor_args(name: &str, symbol: &str, decimals: u8, initial_supply: U256) -> Vec<u8> {
    let mut args = Args::new();
    args.add_string(name)
        .add_string(symbol)
        .add_u8(decimals)
        .add_u256(initial_supply);
    args.into_bytes()
}

#[test]
fn test_constructor() -> Result<()> {
    let wasm = std::fs::read(wasm_path())?;
    let runtime = TestRuntime::new();

    // Set up call stack for deployment context
    runtime
        .interface
        .set_call_stack(vec![DEPLOYER.to_string(), "AS_CONTRACT".to_string()]);

    let initial_supply = U256::from(1_000_000u64);
    let args = constructor_args("TestToken", "TTK", 18, initial_supply);
    let response = runtime.execute(&wasm, "constructor", &args)?;

    // Check events
    let events = runtime.interface.events();
    assert!(events.len() >= 1, "Expected at least 1 event");
    assert!(
        events[0].contains("CHANGE_OWNER"),
        "Expected CHANGE_OWNER event"
    );
    assert!(
        events[0].contains(DEPLOYER),
        "Expected deployer address in event"
    );

    println!("Constructor events: {:?}", events);
    println!("Response: {:?}", response);

    Ok(())
}

#[test]
fn test_name() -> Result<()> {
    let wasm = std::fs::read(wasm_path())?;
    let runtime = TestRuntime::new();

    // Set up deployment
    runtime
        .interface
        .set_call_stack(vec![DEPLOYER.to_string(), "AS_CONTRACT".to_string()]);
    let args = constructor_args("MassaCoin", "MCOIN", 18, U256::from(1_000_000u64));
    runtime.execute(&wasm, "constructor", &args)?;

    // Call name()
    runtime.interface.set_call_stack(vec!["AS_CONTRACT".to_string()]);
    let response = runtime.execute(&wasm, "name", &[])?;
    let name = String::from_utf8(response.ret.clone())?;

    assert_eq!(name, "MassaCoin");
    println!("Token name: {}", name);

    Ok(())
}

#[test]
fn test_symbol() -> Result<()> {
    let wasm = std::fs::read(wasm_path())?;
    let runtime = TestRuntime::new();

    // Set up deployment
    runtime
        .interface
        .set_call_stack(vec![DEPLOYER.to_string(), "AS_CONTRACT".to_string()]);
    let args = constructor_args("MassaCoin", "MCOIN", 18, U256::from(1_000_000u64));
    runtime.execute(&wasm, "constructor", &args)?;

    // Call symbol()
    runtime.interface.set_call_stack(vec!["AS_CONTRACT".to_string()]);
    let response = runtime.execute(&wasm, "symbol", &[])?;
    let symbol = String::from_utf8(response.ret.clone())?;

    assert_eq!(symbol, "MCOIN");
    println!("Token symbol: {}", symbol);

    Ok(())
}

#[test]
fn test_decimals() -> Result<()> {
    let wasm = std::fs::read(wasm_path())?;
    let runtime = TestRuntime::new();

    // Set up deployment
    runtime
        .interface
        .set_call_stack(vec![DEPLOYER.to_string(), "AS_CONTRACT".to_string()]);
    let args = constructor_args("MassaCoin", "MCOIN", 9, U256::from(1_000_000u64));
    runtime.execute(&wasm, "constructor", &args)?;

    // Call decimals()
    runtime.interface.set_call_stack(vec!["AS_CONTRACT".to_string()]);
    let response = runtime.execute(&wasm, "decimals", &[])?;
    let decimals = response.ret[0];

    assert_eq!(decimals, 9);
    println!("Token decimals: {}", decimals);

    Ok(())
}

#[test]
fn test_total_supply() -> Result<()> {
    let wasm = std::fs::read(wasm_path())?;
    let runtime = TestRuntime::new();

    // Set up deployment
    runtime
        .interface
        .set_call_stack(vec![DEPLOYER.to_string(), "AS_CONTRACT".to_string()]);
    let initial_supply = U256::from(5_000_000u64);
    let args = constructor_args("MassaCoin", "MCOIN", 18, initial_supply);
    runtime.execute(&wasm, "constructor", &args)?;

    // Call totalSupply()
    runtime.interface.set_call_stack(vec!["AS_CONTRACT".to_string()]);
    let response = runtime.execute(&wasm, "totalSupply", &[])?;
    
    let mut bytes = [0u8; 32];
    bytes.copy_from_slice(&response.ret[..32]);
    let total_supply = U256::from_le_bytes(bytes);

    assert_eq!(total_supply, initial_supply);
    println!("Total supply: {}", total_supply);

    Ok(())
}

#[test]
fn test_balance_of() -> Result<()> {
    let wasm = std::fs::read(wasm_path())?;
    let runtime = TestRuntime::new();

    // Set up deployment
    runtime
        .interface
        .set_call_stack(vec![DEPLOYER.to_string(), "AS_CONTRACT".to_string()]);
    let initial_supply = U256::from(1_000_000u64);
    let args = constructor_args("MassaCoin", "MCOIN", 18, initial_supply);
    runtime.execute(&wasm, "constructor", &args)?;

    // Check deployer balance
    runtime.interface.set_call_stack(vec!["AS_CONTRACT".to_string()]);
    let mut balance_args = Args::new();
    balance_args.add_string(DEPLOYER);
    let response = runtime.execute(&wasm, "balanceOf", &balance_args.into_bytes())?;
    
    let mut bytes = [0u8; 32];
    bytes.copy_from_slice(&response.ret[..32]);
    let balance = U256::from_le_bytes(bytes);

    assert_eq!(balance, initial_supply);
    println!("Deployer balance: {}", balance);

    // Check Alice balance (should be 0)
    let mut alice_args = Args::new();
    alice_args.add_string(ALICE);
    let response = runtime.execute(&wasm, "balanceOf", &alice_args.into_bytes())?;
    
    let mut bytes = [0u8; 32];
    bytes.copy_from_slice(&response.ret[..32]);
    let alice_balance = U256::from_le_bytes(bytes);

    assert_eq!(alice_balance, U256::ZERO);
    println!("Alice balance: {}", alice_balance);

    Ok(())
}

#[test]
fn test_transfer() -> Result<()> {
    let wasm = std::fs::read(wasm_path())?;
    let runtime = TestRuntime::new();

    // Set up deployment
    runtime
        .interface
        .set_call_stack(vec![DEPLOYER.to_string(), "AS_CONTRACT".to_string()]);
    let initial_supply = U256::from(1_000_000u64);
    let args = constructor_args("MassaCoin", "MCOIN", 18, initial_supply);
    runtime.execute(&wasm, "constructor", &args)?;

    // Transfer from deployer to Alice
    runtime
        .interface
        .set_call_stack(vec![DEPLOYER.to_string(), "AS_CONTRACT".to_string()]);
    let transfer_amount = U256::from(100_000u64);
    let mut transfer_args = Args::new();
    transfer_args.add_string(ALICE).add_u256(transfer_amount);
    runtime.execute(&wasm, "transfer", &transfer_args.into_bytes())?;

    // Check events
    let events = runtime.interface.events();
    let transfer_event = events.iter().find(|e| e.contains("TRANSFER SUCCESS"));
    assert!(transfer_event.is_some(), "Expected transfer event");
    println!("Transfer event: {:?}", transfer_event);

    // Check balances
    runtime.interface.set_call_stack(vec!["AS_CONTRACT".to_string()]);

    let mut deployer_args = Args::new();
    deployer_args.add_string(DEPLOYER);
    let response = runtime.execute(&wasm, "balanceOf", &deployer_args.into_bytes())?;
    let mut bytes = [0u8; 32];
    bytes.copy_from_slice(&response.ret[..32]);
    let deployer_balance = U256::from_le_bytes(bytes);
    
    let expected_deployer = initial_supply.checked_sub(transfer_amount).unwrap();
    assert_eq!(deployer_balance, expected_deployer, "Deployer balance should decrease");

    let mut alice_args = Args::new();
    alice_args.add_string(ALICE);
    let response = runtime.execute(&wasm, "balanceOf", &alice_args.into_bytes())?;
    let mut bytes = [0u8; 32];
    bytes.copy_from_slice(&response.ret[..32]);
    let alice_balance = U256::from_le_bytes(bytes);
    
    assert_eq!(alice_balance, transfer_amount, "Alice balance should increase");

    println!("Deployer balance: {}, Alice balance: {}", deployer_balance, alice_balance);

    Ok(())
}

#[test]
fn test_increase_decrease_allowance() -> Result<()> {
    let wasm = std::fs::read(wasm_path())?;
    let runtime = TestRuntime::new();

    // Set up deployment
    runtime
        .interface
        .set_call_stack(vec![DEPLOYER.to_string(), "AS_CONTRACT".to_string()]);
    let args = constructor_args("MassaCoin", "MCOIN", 18, U256::from(1_000_000u64));
    runtime.execute(&wasm, "constructor", &args)?;

    // Deployer increases allowance for Alice
    runtime
        .interface
        .set_call_stack(vec![DEPLOYER.to_string(), "AS_CONTRACT".to_string()]);
    let approve_amount = U256::from(50_000u64);
    let mut approve_args = Args::new();
    approve_args.add_string(ALICE).add_u256(approve_amount);
    runtime.execute(&wasm, "increaseAllowance", &approve_args.into_bytes())?;

    // Check allowance
    runtime.interface.set_call_stack(vec!["AS_CONTRACT".to_string()]);
    let mut allowance_args = Args::new();
    allowance_args.add_string(DEPLOYER).add_string(ALICE);
    let response = runtime.execute(&wasm, "allowance", &allowance_args.into_bytes())?;
    let mut bytes = [0u8; 32];
    bytes.copy_from_slice(&response.ret[..32]);
    let allowance = U256::from_le_bytes(bytes);

    assert_eq!(allowance, approve_amount);
    println!("Allowance from {} to {}: {}", DEPLOYER, ALICE, allowance);

    // Decrease allowance
    runtime
        .interface
        .set_call_stack(vec![DEPLOYER.to_string(), "AS_CONTRACT".to_string()]);
    let decrease_amount = U256::from(20_000u64);
    let mut decrease_args = Args::new();
    decrease_args.add_string(ALICE).add_u256(decrease_amount);
    runtime.execute(&wasm, "decreaseAllowance", &decrease_args.into_bytes())?;

    // Check new allowance
    runtime.interface.set_call_stack(vec!["AS_CONTRACT".to_string()]);
    let mut allowance_args = Args::new();
    allowance_args.add_string(DEPLOYER).add_string(ALICE);
    let response = runtime.execute(&wasm, "allowance", &allowance_args.into_bytes())?;
    let mut bytes = [0u8; 32];
    bytes.copy_from_slice(&response.ret[..32]);
    let new_allowance = U256::from_le_bytes(bytes);

    let expected = approve_amount.checked_sub(decrease_amount).unwrap();
    assert_eq!(new_allowance, expected);
    println!("New allowance: {}", new_allowance);

    Ok(())
}

#[test]
fn test_transfer_from() -> Result<()> {
    let wasm = std::fs::read(wasm_path())?;
    let runtime = TestRuntime::new();

    // Set up deployment
    runtime
        .interface
        .set_call_stack(vec![DEPLOYER.to_string(), "AS_CONTRACT".to_string()]);
    let initial_supply = U256::from(1_000_000u64);
    let args = constructor_args("MassaCoin", "MCOIN", 18, initial_supply);
    runtime.execute(&wasm, "constructor", &args)?;

    // Deployer increases allowance for Alice
    runtime
        .interface
        .set_call_stack(vec![DEPLOYER.to_string(), "AS_CONTRACT".to_string()]);
    let approve_amount = U256::from(100_000u64);
    let mut approve_args = Args::new();
    approve_args.add_string(ALICE).add_u256(approve_amount);
    runtime.execute(&wasm, "increaseAllowance", &approve_args.into_bytes())?;

    // Alice transfers from Deployer to Bob
    runtime
        .interface
        .set_call_stack(vec![ALICE.to_string(), "AS_CONTRACT".to_string()]);
    let transfer_amount = U256::from(50_000u64);
    let mut transfer_args = Args::new();
    transfer_args
        .add_string(DEPLOYER)
        .add_string(BOB)
        .add_u256(transfer_amount);
    runtime.execute(&wasm, "transferFrom", &transfer_args.into_bytes())?;

    // Check balances
    runtime.interface.set_call_stack(vec!["AS_CONTRACT".to_string()]);

    let mut deployer_args = Args::new();
    deployer_args.add_string(DEPLOYER);
    let response = runtime.execute(&wasm, "balanceOf", &deployer_args.into_bytes())?;
    let mut bytes = [0u8; 32];
    bytes.copy_from_slice(&response.ret[..32]);
    let deployer_balance = U256::from_le_bytes(bytes);
    
    let expected_deployer = initial_supply.checked_sub(transfer_amount).unwrap();
    assert_eq!(deployer_balance, expected_deployer);

    let mut bob_args = Args::new();
    bob_args.add_string(BOB);
    let response = runtime.execute(&wasm, "balanceOf", &bob_args.into_bytes())?;
    let mut bytes = [0u8; 32];
    bytes.copy_from_slice(&response.ret[..32]);
    let bob_balance = U256::from_le_bytes(bytes);
    assert_eq!(bob_balance, transfer_amount);

    // Check remaining allowance
    let mut allowance_args = Args::new();
    allowance_args.add_string(DEPLOYER).add_string(ALICE);
    let response = runtime.execute(&wasm, "allowance", &allowance_args.into_bytes())?;
    let mut bytes = [0u8; 32];
    bytes.copy_from_slice(&response.ret[..32]);
    let remaining_allowance = U256::from_le_bytes(bytes);
    
    let expected_allowance = approve_amount.checked_sub(transfer_amount).unwrap();
    assert_eq!(remaining_allowance, expected_allowance);

    println!(
        "Deployer: {}, Bob: {}, Remaining allowance: {}",
        deployer_balance, bob_balance, remaining_allowance
    );

    Ok(())
}

#[test]
fn test_mint() -> Result<()> {
    let wasm = std::fs::read(wasm_path())?;
    let runtime = TestRuntime::new();

    // Set up deployment
    runtime
        .interface
        .set_call_stack(vec![DEPLOYER.to_string(), "AS_CONTRACT".to_string()]);
    let initial_supply = U256::from(1_000_000u64);
    let args = constructor_args("MassaCoin", "MCOIN", 18, initial_supply);
    runtime.execute(&wasm, "constructor", &args)?;

    // Mint tokens to Alice (owner only)
    runtime
        .interface
        .set_call_stack(vec![DEPLOYER.to_string(), "AS_CONTRACT".to_string()]);
    let mint_amount = U256::from(500_000u64);
    let mut mint_args = Args::new();
    mint_args.add_string(ALICE).add_u256(mint_amount);
    runtime.execute(&wasm, "mint", &mint_args.into_bytes())?;

    // Check new total supply
    runtime.interface.set_call_stack(vec!["AS_CONTRACT".to_string()]);
    let response = runtime.execute(&wasm, "totalSupply", &[])?;
    let mut bytes = [0u8; 32];
    bytes.copy_from_slice(&response.ret[..32]);
    let new_supply = U256::from_le_bytes(bytes);
    
    let expected_supply = initial_supply.checked_add(mint_amount).unwrap();
    assert_eq!(new_supply, expected_supply);

    // Check Alice balance
    let mut alice_args = Args::new();
    alice_args.add_string(ALICE);
    let response = runtime.execute(&wasm, "balanceOf", &alice_args.into_bytes())?;
    let mut bytes = [0u8; 32];
    bytes.copy_from_slice(&response.ret[..32]);
    let alice_balance = U256::from_le_bytes(bytes);
    assert_eq!(alice_balance, mint_amount);

    println!("New total supply: {}, Alice balance: {}", new_supply, alice_balance);

    Ok(())
}

#[test]
fn test_burn() -> Result<()> {
    let wasm = std::fs::read(wasm_path())?;
    let runtime = TestRuntime::new();

    // Set up deployment
    runtime
        .interface
        .set_call_stack(vec![DEPLOYER.to_string(), "AS_CONTRACT".to_string()]);
    let initial_supply = U256::from(1_000_000u64);
    let args = constructor_args("MassaCoin", "MCOIN", 18, initial_supply);
    runtime.execute(&wasm, "constructor", &args)?;

    // Deployer burns some tokens
    runtime
        .interface
        .set_call_stack(vec![DEPLOYER.to_string(), "AS_CONTRACT".to_string()]);
    let burn_amount = U256::from(200_000u64);
    let mut burn_args = Args::new();
    burn_args.add_u256(burn_amount);
    runtime.execute(&wasm, "burn", &burn_args.into_bytes())?;

    // Check new total supply
    runtime.interface.set_call_stack(vec!["AS_CONTRACT".to_string()]);
    let response = runtime.execute(&wasm, "totalSupply", &[])?;
    let mut bytes = [0u8; 32];
    bytes.copy_from_slice(&response.ret[..32]);
    let new_supply = U256::from_le_bytes(bytes);
    
    let expected_supply = initial_supply.checked_sub(burn_amount).unwrap();
    assert_eq!(new_supply, expected_supply);

    // Check deployer balance
    let mut deployer_args = Args::new();
    deployer_args.add_string(DEPLOYER);
    let response = runtime.execute(&wasm, "balanceOf", &deployer_args.into_bytes())?;
    let mut bytes = [0u8; 32];
    bytes.copy_from_slice(&response.ret[..32]);
    let deployer_balance = U256::from_le_bytes(bytes);
    assert_eq!(deployer_balance, expected_supply);

    println!("New total supply: {}, Deployer balance: {}", new_supply, deployer_balance);

    Ok(())
}

#[test]
fn test_full_transfer_flow() -> Result<()> {
    let wasm = std::fs::read(wasm_path())?;
    let runtime = TestRuntime::new();

    println!("=== Full MRC20 Transfer Flow Test ===\n");

    // Step 1: Deploy contract
    println!("Step 1: Deploying MRC20 token...");
    runtime
        .interface
        .set_call_stack(vec![DEPLOYER.to_string(), "AS_CONTRACT".to_string()]);
    let initial_supply = U256::from(10_000_000u64);
    let args = constructor_args("MassaToken", "MASS", 18, initial_supply);
    runtime.execute(&wasm, "constructor", &args)?;
    println!("  Deployed MassaToken (MASS) with initial supply: {}", initial_supply);

    // Step 2: Check initial balances
    println!("\nStep 2: Checking initial balances...");
    runtime.interface.set_call_stack(vec!["AS_CONTRACT".to_string()]);

    let mut deployer_args = Args::new();
    deployer_args.add_string(DEPLOYER);
    let response = runtime.execute(&wasm, "balanceOf", &deployer_args.into_bytes())?;
    let mut bytes = [0u8; 32];
    bytes.copy_from_slice(&response.ret[..32]);
    let deployer_balance = U256::from_le_bytes(bytes);
    println!("  Deployer balance: {}", deployer_balance);

    // Step 3: Transfer to Alice
    println!("\nStep 3: Deployer transfers 1,000,000 to Alice...");
    runtime
        .interface
        .set_call_stack(vec![DEPLOYER.to_string(), "AS_CONTRACT".to_string()]);
    let mut transfer_args = Args::new();
    transfer_args.add_string(ALICE).add_u256(U256::from(1_000_000u64));
    runtime.execute(&wasm, "transfer", &transfer_args.into_bytes())?;

    // Step 4: Alice transfers to Bob
    println!("Step 4: Alice transfers 500,000 to Bob...");
    runtime
        .interface
        .set_call_stack(vec![ALICE.to_string(), "AS_CONTRACT".to_string()]);
    let mut transfer_args = Args::new();
    transfer_args.add_string(BOB).add_u256(U256::from(500_000u64));
    runtime.execute(&wasm, "transfer", &transfer_args.into_bytes())?;

    // Step 5: Bob approves Charlie
    println!("Step 5: Bob approves Charlie to spend 200,000...");
    runtime
        .interface
        .set_call_stack(vec![BOB.to_string(), "AS_CONTRACT".to_string()]);
    let mut approve_args = Args::new();
    approve_args.add_string(CHARLIE).add_u256(U256::from(200_000u64));
    runtime.execute(&wasm, "increaseAllowance", &approve_args.into_bytes())?;

    // Step 6: Charlie transfers from Bob to Alice
    println!("Step 6: Charlie transfers 100,000 from Bob to Alice...");
    runtime
        .interface
        .set_call_stack(vec![CHARLIE.to_string(), "AS_CONTRACT".to_string()]);
    let mut transfer_from_args = Args::new();
    transfer_from_args
        .add_string(BOB)
        .add_string(ALICE)
        .add_u256(U256::from(100_000u64));
    runtime.execute(&wasm, "transferFrom", &transfer_from_args.into_bytes())?;

    // Step 7: Final balances
    println!("\nStep 7: Final balances:");
    runtime.interface.set_call_stack(vec!["AS_CONTRACT".to_string()]);

    let mut args = Args::new();
    args.add_string(DEPLOYER);
    let response = runtime.execute(&wasm, "balanceOf", &args.into_bytes())?;
    let mut bytes = [0u8; 32];
    bytes.copy_from_slice(&response.ret[..32]);
    println!("  Deployer: {}", U256::from_le_bytes(bytes));

    let mut args = Args::new();
    args.add_string(ALICE);
    let response = runtime.execute(&wasm, "balanceOf", &args.into_bytes())?;
    let mut bytes = [0u8; 32];
    bytes.copy_from_slice(&response.ret[..32]);
    println!("  Alice: {}", U256::from_le_bytes(bytes));

    let mut args = Args::new();
    args.add_string(BOB);
    let response = runtime.execute(&wasm, "balanceOf", &args.into_bytes())?;
    let mut bytes = [0u8; 32];
    bytes.copy_from_slice(&response.ret[..32]);
    println!("  Bob: {}", U256::from_le_bytes(bytes));

    let mut args = Args::new();
    args.add_string(CHARLIE);
    let response = runtime.execute(&wasm, "balanceOf", &args.into_bytes())?;
    let mut bytes = [0u8; 32];
    bytes.copy_from_slice(&response.ret[..32]);
    println!("  Charlie: {}", U256::from_le_bytes(bytes));

    // Check remaining allowance
    let mut args = Args::new();
    args.add_string(BOB).add_string(CHARLIE);
    let response = runtime.execute(&wasm, "allowance", &args.into_bytes())?;
    let mut bytes = [0u8; 32];
    bytes.copy_from_slice(&response.ret[..32]);
    println!("\n  Bob->Charlie allowance remaining: {}", U256::from_le_bytes(bytes));

    println!("\n=== Test completed successfully! ===");

    Ok(())
}

#[test]
fn test_u256_large_values() -> Result<()> {
    let wasm = std::fs::read(wasm_path())?;
    let runtime = TestRuntime::new();

    // Use a large U256 value (10^24 = 1 million tokens with 18 decimals)
    let large_supply = U256::from(10u64).pow(24);
    
    println!("Testing with large supply: {}", large_supply);

    // Set up deployment
    runtime
        .interface
        .set_call_stack(vec![DEPLOYER.to_string(), "AS_CONTRACT".to_string()]);
    let args = constructor_args("LargeToken", "LTK", 18, large_supply);
    runtime.execute(&wasm, "constructor", &args)?;

    // Check total supply
    runtime.interface.set_call_stack(vec!["AS_CONTRACT".to_string()]);
    let response = runtime.execute(&wasm, "totalSupply", &[])?;
    let mut bytes = [0u8; 32];
    bytes.copy_from_slice(&response.ret[..32]);
    let total_supply = U256::from_le_bytes(bytes);

    assert_eq!(total_supply, large_supply);
    println!("Large supply verified: {}", total_supply);

    // Transfer a large amount
    runtime
        .interface
        .set_call_stack(vec![DEPLOYER.to_string(), "AS_CONTRACT".to_string()]);
    let transfer_amount = U256::from(10u64).pow(23); // 100,000 tokens
    let mut transfer_args = Args::new();
    transfer_args.add_string(ALICE).add_u256(transfer_amount);
    runtime.execute(&wasm, "transfer", &transfer_args.into_bytes())?;

    // Check Alice balance
    runtime.interface.set_call_stack(vec!["AS_CONTRACT".to_string()]);
    let mut alice_args = Args::new();
    alice_args.add_string(ALICE);
    let response = runtime.execute(&wasm, "balanceOf", &alice_args.into_bytes())?;
    let mut bytes = [0u8; 32];
    bytes.copy_from_slice(&response.ret[..32]);
    let alice_balance = U256::from_le_bytes(bytes);

    assert_eq!(alice_balance, transfer_amount);
    println!("Alice received: {}", alice_balance);

    Ok(())
}
