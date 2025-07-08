use anchor_lang::prelude::*;
mod error;
mod math;
mod state;
mod instructions;

use {
    anchor_lang::prelude::*,
    instructions::init::*,
    instructions::pools::*,
    instructions::liquidity::*,
    instructions::collateral::*,
    instructions::position::*,
    state::perps::{
        AmountAndFee, NewPositionPricesAndFee, PriceAndFee, ProfitAndLoss, SwapAmountAndFees,
    },
};



declare_id!("GDtgSdFsBYezcXHPRywtrU9hxktFHDZvEyA22QEBnuYV");

#[program]
pub mod stockdex {
    use super::*;

    pub fn init(ctx: Context<Init>, params: InitParams) -> Result<()> {
        instructions::init::init(ctx, &params)
    }

    pub fn add_pool<'info>(
        ctx: Context<'_, '_, '_, 'info, AddPool<'info>>,
        params: AddPoolParams,
    ) -> Result<u8> {
        instructions::pools::add_pool(ctx, &params)
    }

    pub fn add_liquidity(ctx: Context<AddLiquidity>, params: AddLiquidityParams) -> Result<()> {
        instructions::liquidity::add_liquidity(ctx, &params)
    }

    pub fn add_collateral(ctx: Context<AddCollateral>, params: AddCollateralParams) -> Result<()> {
        instructions::collateral::add_collateral(ctx, &params)
    }

    pub fn remove_collateral(
        ctx: Context<RemoveCollateral>,
        params: RemoveCollateralParams,
    ) -> Result<()> {
        instructions::collateral::remove_collateral(ctx, &params)
    }

    pub fn open_position(ctx: Context<OpenPosition>, params: OpenPositionParams) -> Result<()> {
        instructions::position::open_position(ctx, &params)
    }

    pub fn close_position(ctx: Context<ClosePosition>, params: ClosePositionParams) -> Result<()> {
        instructions::position::close_position(ctx, &params)
    }
}
