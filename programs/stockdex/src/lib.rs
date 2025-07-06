use anchor_lang::prelude::*;
mod error;
mod math;
mod state;
mod instructions;

declare_id!("GDtgSdFsBYezcXHPRywtrU9hxktFHDZvEyA22QEBnuYV");

#[program]
pub mod stockdex {
    use super::*;

    pub fn initialize(ctx: Context<Initialize>) -> Result<()> {
        msg!("Greetings from: {:?}", ctx.program_id);
        Ok(())
    }
}

#[derive(Accounts)]
pub struct Initialize {}
