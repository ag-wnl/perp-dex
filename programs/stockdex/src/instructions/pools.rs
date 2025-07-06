use {crate::
    {
        error::PerpetualsError,
        state:: {
            perps::Perpetuals,
            pool::Pool,
            multisig::{AdminInstruction, Multisig}
        },
    },
    anchor_lang::prelude::*,
    anchor_spl::token::{Mint, Token}
};


/**
 * Add pool
 */
#[derive(AnchorSerialize, AnchorDeserialize)]
pub struct AddPoolParams {
    pub name: String,
}

#[derive(Accounts)]
#[instruction(params: AddPoolParams)]
pub struct AddPool<'info> {
    #[account(mut)]
    pub admin: Signer<'info>,

    #[account(
        mut,
        seeds = [b"multisig"],
        bump = multisig.load()?.bump
    )]
    pub multisig: AccountLoader<'info, Multisig>,

    /// CHECK: empty PDA, authority for token accounts
    #[account(
        seeds = [b"transfer_authority"],
        bump = perpetuals.transfer_authority_bump
    )]
    pub transfer_authority: AccountInfo<'info>,

    #[account(
        mut,
        realloc = Perpetuals::LEN + (perpetuals.pools.len() + 1) * std::mem::size_of::<Pubkey>(),
        realloc::payer = admin,
        realloc::zero = false,
        seeds = [b"perpetuals"],
        bump = perpetuals.perpetuals_bump
    )]
    pub perpetuals: Box<Account<'info, Perpetuals>>,

    /**
     * as multisig used here instruction can be called multiple times, so init if needed
     * on first call acc zero initialized and filled when all sigs collected. 
     * 
     * cant be used in zero state as to derive the pda actual pool name will be used as seed, 
     * but at that time this account will just contain all zeros.
     */
    #[account(
        init_if_needed,
        payer = admin,
        space = Pool::LEN,
        seeds = [b"pool",
                 params.name.as_bytes()],
        bump
    )]
    pub pool: Box<Account<'info, Pool>>,

    #[account(
        init_if_needed,
        payer = admin,
        mint::authority = transfer_authority,
        mint::freeze_authority = transfer_authority,
        mint::decimals = Perpetuals::LP_DECIMALS,
        seeds = [b"lp_token_mint",
                 pool.key().as_ref()],
        bump
    )]
    pub lp_token_mint: Box<Account<'info, Mint>>,

    system_program: Program<'info, System>,
    token_program: Program<'info, Token>,
    rent: Sysvar<'info, Rent>,
}

pub fn add_pool<'info>(
    ctx: Context<'_, '_, '_, 'info, AddPool<'info>>,
    params: &AddPoolParams
) -> Result<u8> {
    if params.name.is_empty() || params.name.len() > 64 {
        return Err(ProgramError::InvalidArgument.into());
    }

    let mut multisig = ctx.accounts.multisig.load_mut()?;

    let signatures_left = multisig.sign_multisig(
        &ctx.accounts.admin,
        &Multisig::get_account_infos(&ctx)[1..],
        &Multisig::get_instruction_data(AdminInstruction::AddPool, params)?,
    )?;
    if signatures_left > 0 {
        msg!(
            "Instruction has been signed but more signatures are required: {}",
            signatures_left
        );
        return Ok(signatures_left);
    }

    let perpetuals = ctx.accounts.perpetuals.as_mut();
    let pool = ctx.accounts.pool.as_mut();

    // check if we already have the pool created:
    if pool.inception_time != 0 {
        return Err(ProgramError::AccountAlreadyInitialized.into());
    } msg!("Record pool: {}", params.name);

    pool.inception_time = perpetuals.get_time()?;
    pool.name = params.name.clone();
    pool.bump = ctx.bumps.pool;
    pool.lp_token_bump = ctx.bumps.lp_token_mint;

    if !pool.validate() {
        return err!(PerpetualsError::InvalidPoolConfig);
    }

    perpetuals.pools.push(ctx.accounts.pool.key());

    Ok(0)
}