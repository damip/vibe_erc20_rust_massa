//! Tests for the ERC20 Token Contract
//!
//! This test suite validates the ERC20 contract functionality using
//! the massa-testkit runtime.

use anyhow::Result;
use massa_args::Args;
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

/// Helper to create constructor args
fn constructor_args(name: &str, symbol: &str, decimals: u32, initial_supply: u64) -> Vec<u8> {
    let mut args = Args::new();
    args.add_string(name)
        .add_string(symbol)
        .add_u32(decimals)  // Changed to u32 for CLI compatibility
        .add_u64(initial_supply);
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

    let args = constructor_args("TestToken", "TTK", 18, 1_000_000);
    let response = runtime.execute(&wasm, "constructor", &args)?;

    // Check events
    let events = runtime.interface.events();
    assert!(events.len() >= 2, "Expected at least 2 events");
    assert!(
        events[0].contains("ERC20_DEPLOYED"),
        "Expected deployment event"
    );
    assert!(
        events[0].contains("TestToken"),
        "Expected token name in event"
    );
    assert!(events[0].contains("TTK"), "Expected token symbol in event");
    assert!(events[1].contains("TRANSFER"), "Expected transfer event");
    assert!(
        events[1].contains("1000000"),
        "Expected initial supply in event"
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
    let args = constructor_args("MassaCoin", "MCOIN", 18, 1_000_000);
    runtime.execute(&wasm, "constructor", &args)?;

    // Call name()
    runtime.interface.set_call_stack(vec!["AS_CONTRACT".to_string()]);
    let response = runtime.execute(&wasm, "name", &[])?;
    let mut result = Args::from_bytes(response.ret);
    let name = result.next_string()?;

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
    let args = constructor_args("MassaCoin", "MCOIN", 18, 1_000_000);
    runtime.execute(&wasm, "constructor", &args)?;

    // Call symbol()
    runtime.interface.set_call_stack(vec!["AS_CONTRACT".to_string()]);
    let response = runtime.execute(&wasm, "symbol", &[])?;
    let mut result = Args::from_bytes(response.ret);
    let symbol = result.next_string()?;

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
    let args = constructor_args("MassaCoin", "MCOIN", 9, 1_000_000);
    runtime.execute(&wasm, "constructor", &args)?;

    // Call decimals()
    runtime.interface.set_call_stack(vec!["AS_CONTRACT".to_string()]);
    let response = runtime.execute(&wasm, "decimals", &[])?;
    let mut result = Args::from_bytes(response.ret);
    let decimals = result.next_u8()?;

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
    let initial_supply = 5_000_000u64;
    let args = constructor_args("MassaCoin", "MCOIN", 18, initial_supply);
    runtime.execute(&wasm, "constructor", &args)?;

    // Call totalSupply()
    runtime.interface.set_call_stack(vec!["AS_CONTRACT".to_string()]);
    let response = runtime.execute(&wasm, "totalSupply", &[])?;
    let mut result = Args::from_bytes(response.ret);
    let total_supply = result.next_u64()?;

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
    let initial_supply = 1_000_000u64;
    let args = constructor_args("MassaCoin", "MCOIN", 18, initial_supply);
    runtime.execute(&wasm, "constructor", &args)?;

    // Check deployer balance
    runtime.interface.set_call_stack(vec!["AS_CONTRACT".to_string()]);
    let mut balance_args = Args::new();
    balance_args.add_string(DEPLOYER);
    let response = runtime.execute(&wasm, "balanceOf", &balance_args.into_bytes())?;
    let mut result = Args::from_bytes(response.ret);
    let balance = result.next_u64()?;

    assert_eq!(balance, initial_supply);
    println!("Deployer balance: {}", balance);

    // Check Alice balance (should be 0)
    let mut alice_args = Args::new();
    alice_args.add_string(ALICE);
    let response = runtime.execute(&wasm, "balanceOf", &alice_args.into_bytes())?;
    let mut result = Args::from_bytes(response.ret);
    let alice_balance = result.next_u64()?;

    assert_eq!(alice_balance, 0);
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
    let initial_supply = 1_000_000u64;
    let args = constructor_args("MassaCoin", "MCOIN", 18, initial_supply);
    runtime.execute(&wasm, "constructor", &args)?;

    // Transfer from deployer to Alice
    runtime
        .interface
        .set_call_stack(vec![DEPLOYER.to_string(), "AS_CONTRACT".to_string()]);
    let transfer_amount = 100_000u64;
    let mut transfer_args = Args::new();
    transfer_args.add_string(ALICE).add_u64(transfer_amount);
    let response = runtime.execute(&wasm, "transfer", &transfer_args.into_bytes())?;
    let mut result = Args::from_bytes(response.ret);
    let success = result.next_bool()?;

    assert!(success, "Transfer should succeed");

    // Check events
    let events = runtime.interface.events();
    let transfer_event = events.iter().find(|e| e.contains("TRANSFER") && e.contains(ALICE));
    assert!(transfer_event.is_some(), "Expected transfer event");
    println!("Transfer event: {:?}", transfer_event);

    // Check balances
    runtime.interface.set_call_stack(vec!["AS_CONTRACT".to_string()]);

    let mut deployer_args = Args::new();
    deployer_args.add_string(DEPLOYER);
    let response = runtime.execute(&wasm, "balanceOf", &deployer_args.into_bytes())?;
    let mut result = Args::from_bytes(response.ret);
    let deployer_balance = result.next_u64()?;
    assert_eq!(
        deployer_balance,
        initial_supply - transfer_amount,
        "Deployer balance should decrease"
    );

    let mut alice_args = Args::new();
    alice_args.add_string(ALICE);
    let response = runtime.execute(&wasm, "balanceOf", &alice_args.into_bytes())?;
    let mut result = Args::from_bytes(response.ret);
    let alice_balance = result.next_u64()?;
    assert_eq!(
        alice_balance, transfer_amount,
        "Alice balance should increase"
    );

    println!(
        "Deployer balance: {}, Alice balance: {}",
        deployer_balance, alice_balance
    );

    Ok(())
}

#[test]
fn test_transfer_insufficient_balance() -> Result<()> {
    let wasm = std::fs::read(wasm_path())?;
    let runtime = TestRuntime::new();

    // Set up deployment
    runtime
        .interface
        .set_call_stack(vec![DEPLOYER.to_string(), "AS_CONTRACT".to_string()]);
    let initial_supply = 1_000u64;
    let args = constructor_args("MassaCoin", "MCOIN", 18, initial_supply);
    runtime.execute(&wasm, "constructor", &args)?;

    // Try to transfer more than balance
    runtime
        .interface
        .set_call_stack(vec![DEPLOYER.to_string(), "AS_CONTRACT".to_string()]);
    let mut transfer_args = Args::new();
    transfer_args.add_string(ALICE).add_u64(initial_supply + 1);
    let response = runtime.execute(&wasm, "transfer", &transfer_args.into_bytes())?;
    let mut result = Args::from_bytes(response.ret);
    let success = result.next_bool()?;

    assert!(!success, "Transfer should fail due to insufficient balance");

    // Check error event
    let events = runtime.interface.events();
    let error_event = events.iter().find(|e| e.contains("ERROR"));
    assert!(error_event.is_some(), "Expected error event");
    println!("Error event: {:?}", error_event);

    Ok(())
}

#[test]
fn test_approve_and_allowance() -> Result<()> {
    let wasm = std::fs::read(wasm_path())?;
    let runtime = TestRuntime::new();

    // Set up deployment
    runtime
        .interface
        .set_call_stack(vec![DEPLOYER.to_string(), "AS_CONTRACT".to_string()]);
    let args = constructor_args("MassaCoin", "MCOIN", 18, 1_000_000);
    runtime.execute(&wasm, "constructor", &args)?;

    // Deployer approves Alice to spend tokens
    runtime
        .interface
        .set_call_stack(vec![DEPLOYER.to_string(), "AS_CONTRACT".to_string()]);
    let approve_amount = 50_000u64;
    let mut approve_args = Args::new();
    approve_args.add_string(ALICE).add_u64(approve_amount);
    let response = runtime.execute(&wasm, "approve", &approve_args.into_bytes())?;
    let mut result = Args::from_bytes(response.ret);
    let success = result.next_bool()?;

    assert!(success, "Approve should succeed");

    // Check allowance
    runtime.interface.set_call_stack(vec!["AS_CONTRACT".to_string()]);
    let mut allowance_args = Args::new();
    allowance_args.add_string(DEPLOYER).add_string(ALICE);
    let response = runtime.execute(&wasm, "allowance", &allowance_args.into_bytes())?;
    let mut result = Args::from_bytes(response.ret);
    let allowance = result.next_u64()?;

    assert_eq!(allowance, approve_amount);
    println!("Allowance from {} to {}: {}", DEPLOYER, ALICE, allowance);

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
    let initial_supply = 1_000_000u64;
    let args = constructor_args("MassaCoin", "MCOIN", 18, initial_supply);
    runtime.execute(&wasm, "constructor", &args)?;

    // Deployer approves Alice
    runtime
        .interface
        .set_call_stack(vec![DEPLOYER.to_string(), "AS_CONTRACT".to_string()]);
    let approve_amount = 100_000u64;
    let mut approve_args = Args::new();
    approve_args.add_string(ALICE).add_u64(approve_amount);
    runtime.execute(&wasm, "approve", &approve_args.into_bytes())?;

    // Alice transfers from Deployer to Bob
    runtime
        .interface
        .set_call_stack(vec![ALICE.to_string(), "AS_CONTRACT".to_string()]);
    let transfer_amount = 50_000u64;
    let mut transfer_args = Args::new();
    transfer_args
        .add_string(DEPLOYER)
        .add_string(BOB)
        .add_u64(transfer_amount);
    let response = runtime.execute(&wasm, "transferFrom", &transfer_args.into_bytes())?;
    let mut result = Args::from_bytes(response.ret);
    let success = result.next_bool()?;

    assert!(success, "TransferFrom should succeed");

    // Check balances
    runtime.interface.set_call_stack(vec!["AS_CONTRACT".to_string()]);

    let mut deployer_args = Args::new();
    deployer_args.add_string(DEPLOYER);
    let response = runtime.execute(&wasm, "balanceOf", &deployer_args.into_bytes())?;
    let mut result = Args::from_bytes(response.ret);
    let deployer_balance = result.next_u64()?;
    assert_eq!(deployer_balance, initial_supply - transfer_amount);

    let mut bob_args = Args::new();
    bob_args.add_string(BOB);
    let response = runtime.execute(&wasm, "balanceOf", &bob_args.into_bytes())?;
    let mut result = Args::from_bytes(response.ret);
    let bob_balance = result.next_u64()?;
    assert_eq!(bob_balance, transfer_amount);

    // Check remaining allowance
    let mut allowance_args = Args::new();
    allowance_args.add_string(DEPLOYER).add_string(ALICE);
    let response = runtime.execute(&wasm, "allowance", &allowance_args.into_bytes())?;
    let mut result = Args::from_bytes(response.ret);
    let remaining_allowance = result.next_u64()?;
    assert_eq!(remaining_allowance, approve_amount - transfer_amount);

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
    let initial_supply = 1_000_000u64;
    let args = constructor_args("MassaCoin", "MCOIN", 18, initial_supply);
    runtime.execute(&wasm, "constructor", &args)?;

    // Mint tokens to Alice
    runtime
        .interface
        .set_call_stack(vec![DEPLOYER.to_string(), "AS_CONTRACT".to_string()]);
    let mint_amount = 500_000u64;
    let mut mint_args = Args::new();
    mint_args.add_string(ALICE).add_u64(mint_amount);
    let response = runtime.execute(&wasm, "mint", &mint_args.into_bytes())?;
    let mut result = Args::from_bytes(response.ret);
    let success = result.next_bool()?;

    assert!(success, "Mint should succeed");

    // Check new total supply
    runtime.interface.set_call_stack(vec!["AS_CONTRACT".to_string()]);
    let response = runtime.execute(&wasm, "totalSupply", &[])?;
    let mut result = Args::from_bytes(response.ret);
    let new_supply = result.next_u64()?;
    assert_eq!(new_supply, initial_supply + mint_amount);

    // Check Alice balance
    let mut alice_args = Args::new();
    alice_args.add_string(ALICE);
    let response = runtime.execute(&wasm, "balanceOf", &alice_args.into_bytes())?;
    let mut result = Args::from_bytes(response.ret);
    let alice_balance = result.next_u64()?;
    assert_eq!(alice_balance, mint_amount);

    println!(
        "New total supply: {}, Alice balance: {}",
        new_supply, alice_balance
    );

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
    let initial_supply = 1_000_000u64;
    let args = constructor_args("MassaCoin", "MCOIN", 18, initial_supply);
    runtime.execute(&wasm, "constructor", &args)?;

    // Deployer burns some tokens
    runtime
        .interface
        .set_call_stack(vec![DEPLOYER.to_string(), "AS_CONTRACT".to_string()]);
    let burn_amount = 200_000u64;
    let mut burn_args = Args::new();
    burn_args.add_u64(burn_amount);
    let response = runtime.execute(&wasm, "burn", &burn_args.into_bytes())?;
    let mut result = Args::from_bytes(response.ret);
    let success = result.next_bool()?;

    assert!(success, "Burn should succeed");

    // Check new total supply
    runtime.interface.set_call_stack(vec!["AS_CONTRACT".to_string()]);
    let response = runtime.execute(&wasm, "totalSupply", &[])?;
    let mut result = Args::from_bytes(response.ret);
    let new_supply = result.next_u64()?;
    assert_eq!(new_supply, initial_supply - burn_amount);

    // Check deployer balance
    let mut deployer_args = Args::new();
    deployer_args.add_string(DEPLOYER);
    let response = runtime.execute(&wasm, "balanceOf", &deployer_args.into_bytes())?;
    let mut result = Args::from_bytes(response.ret);
    let deployer_balance = result.next_u64()?;
    assert_eq!(deployer_balance, initial_supply - burn_amount);

    println!(
        "New total supply: {}, Deployer balance: {}",
        new_supply, deployer_balance
    );

    Ok(())
}

#[test]
fn test_full_transfer_flow() -> Result<()> {
    let wasm = std::fs::read(wasm_path())?;
    let runtime = TestRuntime::new();

    println!("=== Full ERC20 Transfer Flow Test ===\n");

    // Step 1: Deploy contract
    println!("Step 1: Deploying ERC20 token...");
    runtime
        .interface
        .set_call_stack(vec![DEPLOYER.to_string(), "AS_CONTRACT".to_string()]);
    let initial_supply = 10_000_000u64;
    let args = constructor_args("MassaToken", "MASS", 18, initial_supply);
    runtime.execute(&wasm, "constructor", &args)?;
    println!(
        "  Deployed MassaToken (MASS) with initial supply: {}",
        initial_supply
    );

    // Step 2: Check initial balances
    println!("\nStep 2: Checking initial balances...");
    runtime.interface.set_call_stack(vec!["AS_CONTRACT".to_string()]);

    let mut deployer_args = Args::new();
    deployer_args.add_string(DEPLOYER);
    let response = runtime.execute(&wasm, "balanceOf", &deployer_args.into_bytes())?;
    let mut result = Args::from_bytes(response.ret);
    let deployer_balance = result.next_u64()?;
    println!("  Deployer balance: {}", deployer_balance);

    // Step 3: Transfer to Alice
    println!("\nStep 3: Deployer transfers 1,000,000 to Alice...");
    runtime
        .interface
        .set_call_stack(vec![DEPLOYER.to_string(), "AS_CONTRACT".to_string()]);
    let mut transfer_args = Args::new();
    transfer_args.add_string(ALICE).add_u64(1_000_000);
    runtime.execute(&wasm, "transfer", &transfer_args.into_bytes())?;

    // Step 4: Alice transfers to Bob
    println!("Step 4: Alice transfers 500,000 to Bob...");
    runtime
        .interface
        .set_call_stack(vec![ALICE.to_string(), "AS_CONTRACT".to_string()]);
    let mut transfer_args = Args::new();
    transfer_args.add_string(BOB).add_u64(500_000);
    runtime.execute(&wasm, "transfer", &transfer_args.into_bytes())?;

    // Step 5: Bob approves Charlie
    println!("Step 5: Bob approves Charlie to spend 200,000...");
    runtime
        .interface
        .set_call_stack(vec![BOB.to_string(), "AS_CONTRACT".to_string()]);
    let mut approve_args = Args::new();
    approve_args.add_string(CHARLIE).add_u64(200_000);
    runtime.execute(&wasm, "approve", &approve_args.into_bytes())?;

    // Step 6: Charlie transfers from Bob to Alice
    println!("Step 6: Charlie transfers 100,000 from Bob to Alice...");
    runtime
        .interface
        .set_call_stack(vec![CHARLIE.to_string(), "AS_CONTRACT".to_string()]);
    let mut transfer_from_args = Args::new();
    transfer_from_args
        .add_string(BOB)
        .add_string(ALICE)
        .add_u64(100_000);
    runtime.execute(&wasm, "transferFrom", &transfer_from_args.into_bytes())?;

    // Step 7: Final balances
    println!("\nStep 7: Final balances:");
    runtime.interface.set_call_stack(vec!["AS_CONTRACT".to_string()]);

    let mut args = Args::new();
    args.add_string(DEPLOYER);
    let response = runtime.execute(&wasm, "balanceOf", &args.into_bytes())?;
    let mut result = Args::from_bytes(response.ret);
    println!("  Deployer: {}", result.next_u64()?);

    let mut args = Args::new();
    args.add_string(ALICE);
    let response = runtime.execute(&wasm, "balanceOf", &args.into_bytes())?;
    let mut result = Args::from_bytes(response.ret);
    println!("  Alice: {}", result.next_u64()?);

    let mut args = Args::new();
    args.add_string(BOB);
    let response = runtime.execute(&wasm, "balanceOf", &args.into_bytes())?;
    let mut result = Args::from_bytes(response.ret);
    println!("  Bob: {}", result.next_u64()?);

    let mut args = Args::new();
    args.add_string(CHARLIE);
    let response = runtime.execute(&wasm, "balanceOf", &args.into_bytes())?;
    let mut result = Args::from_bytes(response.ret);
    println!("  Charlie: {}", result.next_u64()?);

    // Check remaining allowance
    let mut args = Args::new();
    args.add_string(BOB).add_string(CHARLIE);
    let response = runtime.execute(&wasm, "allowance", &args.into_bytes())?;
    let mut result = Args::from_bytes(response.ret);
    println!("\n  Bob->Charlie allowance remaining: {}", result.next_u64()?);

    println!("\n=== Test completed successfully! ===");

    Ok(())
}
