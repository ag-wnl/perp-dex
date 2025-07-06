use {anchor_lang::{prelude::*}, anchor_spl::token::{Burn, MintTo, Transfer}};


#[derive(Copy, Clone, PartialEq, AnchorSerialize, AnchorDeserialize, Default, Debug)]
pub struct PriceAndFee {
    pub price: u64,
    pub fee: u64,
}

#[derive(Copy, Clone, PartialEq, AnchorSerialize, AnchorDeserialize, Default, Debug)]
pub struct AmountAndFee {
    pub amount: u64,
    pub fee: u64,
}

#[derive(Copy, Clone, PartialEq, AnchorSerialize, AnchorDeserialize, Default, Debug)]
pub struct NewPositionPricesAndFee {
    pub entry_price: u64,
    pub liquidation_price: u64,
    pub fee: u64,
}

#[derive(Copy, Clone, PartialEq, AnchorSerialize, AnchorDeserialize, Default, Debug)]
pub struct SwapAmountAndFees {
    pub amount_out: u64,
    pub fee_in: u64,
    pub fee_out: u64,
}

#[derive(Copy, Clone, PartialEq, AnchorSerialize, AnchorDeserialize, Default, Debug)]
pub struct ProfitAndLoss {
    pub profit: u64,
    pub loss: u64,
}

#[derive(Copy, Clone, PartialEq, AnchorSerialize, AnchorDeserialize, Default, Debug)]
pub struct Permissions {
    pub allow_swap: bool,
    pub allow_add_liquidity: bool,
    pub allow_remove_liquidity: bool,
    pub allow_open_position: bool,
    pub allow_close_position: bool,
    pub allow_pnl_withdrawal: bool,
    pub allow_collateral_withdrawal: bool,
    pub allow_size_change: bool,
}

#[account]
#[derive(Default, Debug)]
pub struct Perpetuals {
    pub permissions: Permissions,
    pub pools: Vec<Pubkey>,

    pub transfer_authority_bump: u8,
    pub perpetuals_bump: u8,
    pub inception_time: i64,
}   

impl anchor_lang::Id for Perpetuals {
    fn id() -> Pubkey {
        crate::ID // deployed program id
    }
}

impl Perpetuals {
    pub const LEN: usize = 8 + std::mem::size_of::<Perpetuals>();

    // basis pts - 1 pt = 0.01%
    pub const BPS_DECIMALS: u8 = 4;
    pub const BPS_POWER: u128 = 10u64.pow(Self::BPS_DECIMALS as u32) as u128;

    pub const PRICE_DECIMALS: u8 = 6;
    pub const USD_DECIMALS:   u8 = 6;

    pub const LP_DECIMALS: u8 =  Self::USD_DECIMALS;
    pub const RATE_DECIMALS: u8 = 9;
    pub const RATE_POWER: u128 = 10u64.pow(Self::RATE_DECIMALS as u32) as u128;


    
	pub fn get_time(&self) -> Result<i64> {
	    let time = solana_program::sysvar::clock::Clock::get()?.unix_timestamp;
	    if time > 0 {
	        Ok(time)
	    } else {
	        Err(ProgramError::InvalidAccountData.into())
	    }
	}

    // when runtime authority is pda, seed/bump so program can sign it:
    pub fn transfer_token<'info>(
        &self, 
        from: AccountInfo<'info>, 
        to: AccountInfo<'info>, 
        authority: AccountInfo<'info>, 
        amount: u64, 
        token_program: AccountInfo<'info>) -> Result<()> {
        let authority_seeds: &[&[&[u8]]] = &[&[b"transfer-authority", &[self.transfer_authority_bump]]];

        let ctx = CpiContext::new(
            token_program,
            Transfer {
                from,
                to,
                authority,
            }
        ).with_signer(authority_seeds);

        anchor_spl::token::transfer(ctx, amount)?;
        Ok(())
    }

    pub fn transfer_token_from_user<'info>(&self, 
        from: AccountInfo<'info>, 
        to: AccountInfo<'info>, 
        authority: AccountInfo<'info>, 
        amount: u64, 
        token_program: AccountInfo<'info>
    ) -> Result<()> {

        let ctx = CpiContext::new(
            token_program,
            Transfer {
                from,
                to,
                authority,
            }
        );
        anchor_spl::token::transfer(ctx, amount)?;
        
        Ok(())
    }

    pub fn mint_token<'info>(
        &self,
        mint: AccountInfo<'info>,
        to: AccountInfo<'info>,
        authority: AccountInfo<'info>,
        amount: u64,
        token_program: AccountInfo<'info>,
    ) -> Result<()> {
        let authority_seeds: &[&[&[u8]]] = &[&[b"mint-authority", &[self.transfer_authority_bump]]];

        let ctx = CpiContext::new(
            token_program,
            MintTo {
                mint,
                to,
                authority
            }
        ).with_signer(authority_seeds);

        anchor_spl::token::mint_to(ctx, amount)?;
        Ok(())
    }

    pub fn burn_tokens<'info>(
        &self,
        mint: AccountInfo<'info>,
        from: AccountInfo<'info>,
        authority: AccountInfo<'info>,
        token_program: AccountInfo<'info>,
        amount: u64,
    ) -> Result<()> {
        let context = CpiContext::new(
            token_program,
            Burn {
                mint,
                from,
                authority,
            },
        );

        anchor_spl::token::burn(context, amount)?;
        Ok(())
    }

    pub fn transfer_sol_from_owned_account<'a>(
        program_owner_source_acc: AccountInfo<'a>,
        destination_acc: AccountInfo<'a>,
        amount: u64,
    ) -> Result<()> {
        **destination_acc.try_borrow_mut_lamports()? = destination_acc.try_lamports()?
        .checked_add(amount).ok_or(ProgramError::InsufficientFunds)?;

        let source_bal = program_owner_source_acc.try_lamports()?;
        **program_owner_source_acc.try_borrow_mut_lamports()? = source_bal.checked_sub(amount).ok_or(ProgramError::InsufficientFunds)?;

        Ok(())
    }

    pub fn transfer_sol<'a>(
        source_acc: AccountInfo<'a>,
        destination_acc: AccountInfo<'a>,
        system_program: AccountInfo<'a>,
        amount: u64,
    ) -> Result<()> {
        let cpi_accounts = anchor_lang::system_program::Transfer {
            from: source_acc,
            to: destination_acc,
        };

        let ctx = CpiContext::new(system_program, cpi_accounts);
        anchor_lang::system_program::transfer(ctx, amount)
    }

    pub fn realloc_account<'a>(
        source_acc: AccountInfo<'a>,
        destination_acc: AccountInfo<'a>,
        system_program: AccountInfo<'a>,
        new_len: usize,
        zero_init: bool,
    ) -> Result<()> {
        let min_bal_required = Rent::get()?.minimum_balance(new_len);
        let balance_diff = min_bal_required.saturating_sub(destination_acc.try_lamports()?);

        Perpetuals::transfer_sol(source_acc, destination_acc.clone(), system_program, balance_diff)?;

        destination_acc.realloc(new_len, zero_init).map_err(|_| ProgramError::InvalidRealloc.into())
    }
 
}




