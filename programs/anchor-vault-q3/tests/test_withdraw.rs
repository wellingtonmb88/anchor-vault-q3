#![cfg(feature = "test-sbf")]

use anchor_lang::{ InstructionData };
use solana_sdk::{ instruction::{ AccountMeta, Instruction } };

mod utils;
use utils::{ setup_initialized_and_deposited_vault };

#[test]
fn test_withdraw_success() {
    let (mollusk, user, vault_state, vault, _, _, deposit_result) =
        setup_initialized_and_deposited_vault();
    let (system_program, system_account) = mollusk_svm::program::keyed_account_for_system_program();
    let program_id = anchor_vault_q3::id();

    let withdraw_amount = 5_000_000; // 0.005 SOL

    // Create withdraw instruction
    let instruction = Instruction::new_with_bytes(
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
    let user_lamports_before = user_account.lamports;
    let vault_account = deposit_result.get_account(&vault).unwrap();
    let vault_lamports_before = vault_account.lamports;

    let withdraw_accounts = &vec![
        (user, user_account.clone()),
        (vault_state, deposit_result.get_account(&vault_state).unwrap().clone()),
        (vault, vault_account.clone()),
        (system_program, system_account)
    ];
    let result = mollusk.process_instruction(&instruction, &withdraw_accounts);

    // Verify success
    assert!(!result.program_result.is_err(), "Withdraw should succeed");

    // Verify user lamports increased
    let user_account = &result.get_account(&user).unwrap();
    assert_eq!(
        user_account.lamports,
        user_lamports_before + withdraw_amount,
        "User lamports should increase by withdraw amount"
    );

    // Verify vault lamports decreased
    let vault_account = &result.get_account(&vault).unwrap();
    assert_eq!(
        vault_account.lamports,
        vault_lamports_before - withdraw_amount,
        "Vault lamports should decrease by withdraw amount"
    );
}
