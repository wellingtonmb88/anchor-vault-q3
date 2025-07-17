//! Test utilities for the anchor-vault-q3 program
//!
//! This module contains shared setup functions and constants
//! used across multiple test files.

use anchor_lang::prelude::*;
use anchor_lang::InstructionData;
use mollusk_svm::{ result::InstructionResult, Mollusk };
use solana_sdk::{ account::Account, instruction::Instruction, pubkey::Pubkey };

/// Initial lamport balance for test users
pub const USER_INITIAL_LAMPORTS: u64 = 10_000_000;

/// Sets up a fully initialized vault for testing
///
/// Returns:
/// - Mollusk instance
/// - User public key
/// - Vault state public key
/// - Vault public key
/// - Vault state bump
/// - Vault bump
/// - Initialize instruction result
pub fn setup_initialized_vault() -> (
    Mollusk,
    Pubkey, // user
    Pubkey, // vault_state
    Pubkey, // vault
    u8, // vault_state_bump
    u8, // vault_bump
    InstructionResult, // initialize result
) {
    let program_id = anchor_vault_q3::id();
    let mollusk = Mollusk::new(&program_id, "anchor_vault_q3");
    let (system_program, system_account) = mollusk_svm::program::keyed_account_for_system_program();
    let user = Pubkey::new_unique();

    let (vault_state, vault_state_bump) = Pubkey::find_program_address(
        &[b"state", user.as_ref()],
        &program_id
    );

    let (vault, vault_bump) = Pubkey::find_program_address(
        &[b"vault", vault_state.as_ref()],
        &program_id
    );

    // First, run initialize to set up the vault properly
    let initialize_instruction = Instruction::new_with_bytes(
        program_id,
        &(anchor_vault_q3::instruction::Initialize {}).data(),
        vec![
            AccountMeta::new(user, true),
            AccountMeta::new(vault_state, false),
            AccountMeta::new(vault, false),
            AccountMeta::new_readonly(system_program, false)
        ]
    );
    let user_account = Account::new(USER_INITIAL_LAMPORTS, 0, &system_program);
    let vault_account = Account::new(0, 0, &system_program);

    let initialize_accounts = &vec![
        (user, user_account),
        (vault_state, Account::new(0, 0, &system_program)),
        (vault, vault_account),
        (system_program, system_account.clone())
    ];

    let initialize_result = mollusk.process_instruction(
        &initialize_instruction,
        &initialize_accounts
    );
    assert!(!initialize_result.program_result.is_err(), "Initialize should succeed");

    (mollusk, user, vault_state, vault, vault_state_bump, vault_bump, initialize_result)
}

/// Sets up a fully deposited vault for testing
///
/// Returns:
/// - Mollusk instance
/// - User public key
/// - Vault state public key
/// - Vault public key
/// - Vault state bump
/// - Vault bump
/// - Deposit instruction result
pub fn setup_initialized_and_deposited_vault() -> (
    Mollusk,
    Pubkey, // user
    Pubkey, // vault_state
    Pubkey, // vault
    u8, // vault_state_bump
    u8, // vault_bump
    InstructionResult, // deposit result
) {
    let (mollusk, user, vault_state, vault, vault_state_bump, vault_bump, initialize_result) =
        setup_initialized_vault();

    // Now run the deposit instruction
    let deposit_amount = 5_000_000; // 0.005 SOL

    let deposit_instruction = Instruction::new_with_bytes(
        anchor_vault_q3::id(),
        &(anchor_vault_q3::instruction::Deposit { amount: deposit_amount }).data(),
        vec![
            AccountMeta::new(user, true),
            AccountMeta::new(vault, false),
            AccountMeta::new_readonly(vault_state, false),
            AccountMeta::new_readonly(
                mollusk_svm::program::keyed_account_for_system_program().0,
                false
            )
        ]
    );

    let deposit_accounts = &vec![
        (user, initialize_result.get_account(&user).unwrap().clone()),
        (vault_state, initialize_result.get_account(&vault_state).unwrap().clone()),
        (vault, initialize_result.get_account(&vault).unwrap().clone()),
        (
            mollusk_svm::program::keyed_account_for_system_program().0,
            mollusk_svm::program::keyed_account_for_system_program().1,
        )
    ];

    let deposit_result = mollusk.process_instruction(&deposit_instruction, &deposit_accounts);
    assert!(!deposit_result.program_result.is_err(), "Deposit should succeed");

    (mollusk, user, vault_state, vault, vault_state_bump, vault_bump, deposit_result)
}
