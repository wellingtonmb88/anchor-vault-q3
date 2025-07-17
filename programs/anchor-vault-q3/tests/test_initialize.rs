#![cfg(feature = "test-sbf")]

use ::{
    anchor_lang::{ prelude::*, solana_program::rent::Rent, InstructionData },
    mollusk_svm::Mollusk,
    solana_sdk::{ account::Account, instruction::{ AccountMeta, Instruction }, pubkey::{ Pubkey } },
};

#[test]
fn test_initialize() {
    let program_id = anchor_vault_q3::id();
    let mollusk = Mollusk::new(&program_id, "anchor_vault_q3");
    let (system_program, system_account) = mollusk_svm::program::keyed_account_for_system_program();

    // Generate test keypairs
    // let user = Pubkey::new_unique();
    let user = Pubkey::new_from_array([0x01; 32]);

    // Derive PDAs
    let (vault_state, vault_state_bump) = Pubkey::find_program_address(
        &[b"state", user.as_ref()],
        &program_id
    );
    let (vault, vault_bump) = Pubkey::find_program_address(
        &[b"vault", vault_state.as_ref()],
        &program_id
    );

    // Create instruction
    // Setup accounts
    let user_lamports = 10_000_000; // 0.01 SOL
    let rent = Rent::default();
    let vault_state_space = 8 + anchor_vault_q3::VaultState::INIT_SPACE;
    let vault_state_rent = rent.minimum_balance(vault_state_space);
    let user_account = Account::new(user_lamports, 0, &system_program);

    // Create a new system-owned account for the vault
    // Important: For system-owned PDAs, we need to initialize it with the system program as owner
    let vault_account = Account::new(0, 0, &system_program);

    let instruction = Instruction::new_with_bytes(
        program_id,
        &(anchor_vault_q3::instruction::Initialize {}).data(),
        vec![
            AccountMeta::new(user, true),
            AccountMeta::new(vault_state, false),
            AccountMeta::new(vault, false),
            AccountMeta::new_readonly(system_program, false)
        ]
    );

    let accounts = &vec![
        (user, user_account),
        (vault_state, Account::new(0, 0, &system_program)),
        (vault, vault_account),
        (system_program, system_account)
    ];

    let result = mollusk.process_instruction(&instruction, accounts);

    // Verify success
    assert!(!result.program_result.is_err(), "Initialize should succeed");

    // Verify vault state account was initialized
    let vault_state_account = &result.get_account(&vault_state).unwrap();
    assert_eq!(vault_state_account.owner, program_id);
    assert!(vault_state_account.lamports >= vault_state_rent);
    assert_eq!(
        vault_state_account.data.len(),
        vault_state_space,
        "Vault state should have correct space"
    );
    assert_eq!(
        vault_state_account.data[8],
        vault_state_bump,
        "Vault state should have correct bump"
    );
    assert_eq!(vault_state_account.data[9], vault_bump, "Vault should have correct bump");

    // Verify vault account received rent-exempt amount
    let vault_account = &result.get_account(&vault).unwrap();
    let expected_vault_rent = rent.minimum_balance(0);
    assert_eq!(vault_account.lamports, expected_vault_rent);
}
