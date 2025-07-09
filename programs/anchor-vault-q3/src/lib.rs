#![allow(deprecated)]
#![allow(unexpected_cfgs)]

use anchor_lang::{ prelude::*, system_program::{ Transfer, transfer } };

declare_id!("EQSjMmLReExSNm29r7MW1RX5UQCQbhv2bpjZYPTAAwXH");

#[program]
pub mod anchor_vault_q3 {
    use super::*;

    pub fn initialize(ctx: Context<Initialize>) -> Result<()> {
        ctx.accounts.initialize(ctx.bumps)
    }

    pub fn deposit(ctx: Context<Deposit>, amount: u64) -> Result<()> {
        ctx.accounts.deposit(amount)
    }

    pub fn withdraw(ctx: Context<Withdraw>, amount: u64) -> Result<()> {
        ctx.accounts.withdraw(amount)
    }

    pub fn close(ctx: Context<Close>) -> Result<()> {
        ctx.accounts.close()
    }
}

#[derive(Accounts)]
pub struct Initialize<'info> {
    #[account(mut)]
    pub user: Signer<'info>,
    #[account(
        init,
        payer = user,
        seeds = [b"state", user.key().as_ref()],
        bump,
        space = 8 + VaultState::INIT_SPACE
    )]
    pub vault_state: Account<'info, VaultState>,
    #[account(
        mut,
        seeds = [b"vault", vault_state.key().as_ref()],
        bump,
    )]
    pub vault: SystemAccount<'info>,
    pub system_program: Program<'info, System>,
}

impl<'info> Initialize<'info> {
    pub fn initialize(&mut self, bumps: InitializeBumps) -> Result<()> {
        let rent_exempt = Rent::get()?.minimum_balance(self.vault.to_account_info().data_len());
        let cpi_program = self.system_program.to_account_info();

        let cpi_accounts = Transfer {
            from: self.user.to_account_info(),
            to: self.vault.to_account_info(),
        };

        let cpi_ctx = CpiContext::new(cpi_program, cpi_accounts);
        transfer(cpi_ctx, rent_exempt)?;

        self.vault_state.bump = bumps.vault_state;
        self.vault_state.vault_bump = bumps.vault;
        Ok(())
    }
}

#[derive(Accounts)]
pub struct Deposit<'info> {
    #[account(mut)]
    pub user: Signer<'info>,
    #[account(
        mut,
        seeds = [b"vault", vault_state.key().as_ref()],
        bump = vault_state.vault_bump,
    )]
    pub vault: SystemAccount<'info>,
    #[account(seeds = [b"state", user.key().as_ref()], bump = vault_state.bump)]
    pub vault_state: Account<'info, VaultState>,
    pub system_program: Program<'info, System>,
}

impl<'info> Deposit<'info> {
    pub fn deposit(&mut self, amount: u64) -> Result<()> {
        let cpi_program = self.system_program.to_account_info();

        let cpi_accounts = Transfer {
            from: self.user.to_account_info(),
            to: self.vault.to_account_info(),
        };

        let cpi_ctx = CpiContext::new(cpi_program, cpi_accounts);
        transfer(cpi_ctx, amount)?;

        Ok(())
    }
}

#[derive(Accounts)]
pub struct Withdraw<'info> {
    #[account(mut)]
    pub user: Signer<'info>,
    #[account(
        mut,
        seeds = [b"vault", vault_state.key().as_ref()],
        bump = vault_state.vault_bump,
    )]
    pub vault: SystemAccount<'info>,
    #[account(seeds = [b"state", user.key().as_ref()], bump = vault_state.bump)]
    pub vault_state: Account<'info, VaultState>,
    pub system_program: Program<'info, System>,
}

impl<'info> Withdraw<'info> {
    pub fn withdraw(&mut self, amount: u64) -> Result<()> {
        let rent_exempt = Rent::get()?.minimum_balance(self.vault.to_account_info().data_len());
        if amount < rent_exempt {
            return Err(VaultErrorCode::InsufficientWithdrawalAmount.into());
        }
        if amount > self.vault.to_account_info().lamports() {
            return Err(VaultErrorCode::InsufficientVaultBalance.into());
        }
        let cpi_program = self.system_program.to_account_info();

        let cpi_accounts = Transfer {
            from: self.vault.to_account_info(),
            to: self.user.to_account_info(),
        };

        let vault_state_key = self.vault_state.key();
        let seeds = &[b"vault".as_ref(), vault_state_key.as_ref(), &[self.vault_state.vault_bump]];
        let seeds_signer = &[&seeds[..]];

        let cpi_ctx = CpiContext::new_with_signer(cpi_program, cpi_accounts, seeds_signer);
        transfer(cpi_ctx, amount)?;

        Ok(())
    }
}

#[derive(Accounts)]
pub struct Close<'info> {
    #[account(mut)]
    pub user: Signer<'info>,
    #[account(
        mut,
        seeds = [b"state", user.key().as_ref()],
        bump = vault_state.bump,
        close = user,
    )]
    pub vault_state: Account<'info, VaultState>,
    #[account(
        mut,
        seeds = [b"vault", vault_state.key().as_ref()],
        bump = vault_state.vault_bump,
    )]
    pub vault: SystemAccount<'info>,
    pub system_program: Program<'info, System>,
}

impl<'info> Close<'info> {
    pub fn close(&mut self) -> Result<()> {
        // Ensure the vault has lamports before closing
        if self.vault.to_account_info().lamports() <= 0 {
            return Err(VaultErrorCode::VaultAlreadyClosed.into());
        }
        // Close the vault state account
        let vault_state_key = self.vault_state.key();
        let seeds = &[b"vault".as_ref(), vault_state_key.as_ref(), &[self.vault_state.vault_bump]];
        let seeds_signer = &[&seeds[..]];   
        let cpi_program = self.vault.to_account_info();
        let cpi_accounts = Transfer {
            from: self.vault.to_account_info(),
            to: self.user.to_account_info(),
        };
        let cpi_ctx = CpiContext::new_with_signer(cpi_program, cpi_accounts, seeds_signer);
        transfer(cpi_ctx, self.vault.to_account_info().lamports())?; // Transfer lamports to close the account
        // Set the vault state account to zero
        self.vault_state.bump = 0;
        self.vault_state.vault_bump = 0;
        Ok(())
    }
}

#[account]
#[derive(InitSpace)]
pub struct VaultState {
    pub bump: u8,
    pub vault_bump: u8,
}

// impl Space for VaultState {
//     const INIT_SPACE: usize = 8 + 1 + 1; // discriminator + bump + vault_bump
// }

#[error_code]
pub enum VaultErrorCode {
    #[msg("Insufficient withdrawal amount")]
    InsufficientWithdrawalAmount,
    #[msg("Insufficient balance in vault")]
    InsufficientVaultBalance,
    #[msg("Vault already closed")]
    VaultAlreadyClosed,
}
