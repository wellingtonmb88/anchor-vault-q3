use anchor_lang::prelude::*;

declare_id!("EQSjMmLReExSNm29r7MW1RX5UQCQbhv2bpjZYPTAAwXH");

#[program]
pub mod anchor_vault_q3 {
    use super::*;

    pub fn initialize(ctx: Context<Initialize>) -> Result<()> {
        msg!("Greetings from: {:?}", ctx.program_id);
        Ok(())
    }
}

#[derive(Accounts)]
pub struct Initialize {}
