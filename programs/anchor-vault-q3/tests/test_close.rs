#![cfg(feature = "test-sbf")]

use anchor_lang::{ InstructionData };
use solana_sdk::{ instruction::{ AccountMeta, Instruction } };

mod utils;
use utils::{ setup_initialized_and_deposited_vault, USER_INITIAL_LAMPORTS };

#[test]
fn test_close_success() {
    let (mollusk, user, vault_state, vault, _, _, deposit_result) =
        setup_initialized_and_deposited_vault();
    let (system_program, system_account) = mollusk_svm::program::keyed_account_for_system_program();
    let program_id = anchor_vault_q3::id();
    let withdraw_amount = 5_000_000; // 0.005 SOL

    // Create withdraw instruction
    let withdraw_instruction = Instruction::new_with_bytes(
        program_id,
        &(anchor_vault_q3::instruction::Withdraw { amount: withdraw_amount }).data(),
        vec![
            AccountMeta::new(user, true),
            AccountMeta::new(vault, false),
            AccountMeta::new_readonly(vault_state, false),
            AccountMeta::new_readonly(system_program, false)
        ]
    );

    let user_account = deposit_result.get_account(&user).unwrap();
    let vault_account = deposit_result.get_account(&vault).unwrap();
    let withdraw_accounts = &vec![
        (user, user_account.clone()),
        (vault_state, deposit_result.get_account(&vault_state).unwrap().clone()),
        (vault, vault_account.clone()),
        (system_program, system_account.clone())
    ];
    let withdraw_result = mollusk.process_instruction(&withdraw_instruction, &withdraw_accounts);

    // Verify success
    assert!(!withdraw_result.program_result.is_err(), "Withdraw should succeed");

    // Create close instruction
    let close_instruction = Instruction::new_with_bytes(
        program_id,
        &(anchor_vault_q3::instruction::Close {}).data(),
        vec![
            AccountMeta::new(user, true),
            AccountMeta::new(vault_state, false),
            AccountMeta::new(vault, false),
            AccountMeta::new_readonly(system_program, false)
        ]
    );

    let user_account = withdraw_result.get_account(&user).unwrap();
    let vault_state_account = withdraw_result.get_account(&vault_state).unwrap();
    let vault_account = withdraw_result.get_account(&vault).unwrap();

    let close_accounts = &vec![
        (user, user_account.clone()),
        (vault_state, vault_state_account.clone()),
        (vault, vault_account.clone()),
        (system_program, system_account.clone())
    ];
    let close_result = mollusk.process_instruction(&close_instruction, &close_accounts);

    assert!(!close_result.program_result.is_err(), "Close should succeed");

    // Verify user received all lamports from vault and vault_state
    let user_account = &close_result.get_account(&user).unwrap();
    assert_eq!(
        user_account.lamports,
        USER_INITIAL_LAMPORTS,
        "User should receive vault lamports and vault_state rent"
    );

    // Verify vault account is empty
    let vault_account = &close_result.get_account(&vault).unwrap();
    assert_eq!(vault_account.lamports, 0, "Vault should be empty after close");

    // Verify vault_state account is closed (lamports = 0)
    let vault_state_account = &close_result.get_account(&vault_state);
    assert!(
        vault_state_account.is_none() || vault_state_account.unwrap().lamports == 0,
        "Vault state should be closed"
    );
}
