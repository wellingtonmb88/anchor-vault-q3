#![cfg(feature = "test-sbf")]

use anchor_lang::{ prelude::*, solana_program::{ rent::Rent }, InstructionData };
use solana_sdk::{ instruction::{ AccountMeta, Instruction } };

mod utils;
use utils::{ setup_initialized_vault, USER_INITIAL_LAMPORTS };

#[test]
fn test_deposit_success() {
    let (mollusk, user, vault_state, vault, _, _, initialize_result) = setup_initialized_vault();
    let (system_program, system_account) = mollusk_svm::program::keyed_account_for_system_program();
    let program_id = anchor_vault_q3::id();

    let rent = Rent::default();
    let vault_state_space = 8 + anchor_vault_q3::VaultState::INIT_SPACE;
    let vault_state_rent = rent.minimum_balance(vault_state_space);
    let vault_rent = rent.minimum_balance(0);

    // Now run the deposit instruction
    let deposit_amount = 5_000_000; // 0.005 SOL

    let deposit_instruction = Instruction::new_with_bytes(
        program_id,
        &(anchor_vault_q3::instruction::Deposit { amount: deposit_amount }).data(),
        vec![
            AccountMeta::new(user, true),
            AccountMeta::new(vault, false),
            AccountMeta::new_readonly(vault_state, false),
            AccountMeta::new_readonly(system_program, false)
        ]
    );

    // Use the accounts from the deposit result
    let deposit_accounts = &vec![
        (user, initialize_result.get_account(&user).unwrap().clone()),
        (vault_state, initialize_result.get_account(&vault_state).unwrap().clone()),
        (vault, initialize_result.get_account(&vault).unwrap().clone()),
        (system_program, system_account)
    ];

    let result = mollusk.process_instruction(&deposit_instruction, &deposit_accounts);

    // Verify success
    assert!(!result.program_result.is_err(), "Deposit should succeed");

    // Calculate expected lamports after both initialize and deposit
    let expected_user_lamports =
        USER_INITIAL_LAMPORTS - vault_state_rent - vault_rent - deposit_amount;
    let expected_vault_lamports = vault_rent + deposit_amount;

    // Verify user lamports decreased
    let user_account = &result.get_account(&user).unwrap();
    assert_eq!(
        user_account.lamports,
        expected_user_lamports,
        "User lamports should decrease by vault rent + deposit amount"
    );

    // Verify vault lamports increased
    let vault_account = &result.get_account(&vault).unwrap();
    assert_eq!(
        vault_account.lamports,
        expected_vault_lamports,
        "Vault lamports should be rent + deposit amount"
    );
}
