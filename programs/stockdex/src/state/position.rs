use crate::{math, state::perps::Perpetuals};
use anchor_lang::prelude::*;


#[derive(Copy, Clone, PartialEq, AnchorSerialize, AnchorDeserialize, Debug)]
pub enum Side {
    Long,
    Short,
    None
}

impl Default for Side {
    fn default() -> Self {
        Side::None
    }
}

#[derive(Copy, Clone, PartialEq, AnchorSerialize, AnchorDeserialize, Debug)]
pub enum CollateralChange {
    Add,
    Remove,
    None
}

impl Default for CollateralChange {
    fn default() -> Self {
        CollateralChange::None
    }
}

#[account]
#[derive(Default, Debug)]
pub struct Position {
    pub owner: Pubkey,
    pub pool: Pubkey,

    pub custody: Pubkey, // acc holding position's actual trading asset (ex: sol)
    pub collateral_custody: Pubkey, // acc holding the collateral for this posn - the vault where the user’s deposited collateral is stored and used for margin, PnL, and liquidations

    pub open_time: i64,
    pub update_time: i64,
    pub side: Side,
    pub price: u64,
    pub size_usd: u64, // P_notational
    pub borrow_size_usd: u64, // amount borrowed for leverage
    pub collateral_usd: u64, 
    pub unrealized_profit_usd: u64,
    pub unrealized_loss_usd: u64,
    pub cumulative_interest_snapshot: u128, // interest/funding snapshot 
    pub locked_amount: u64, // net amount locked for this posn
    pub collateral_amount: u64, // actual collateral amount

    pub bump: u8,
}

impl Position {
    pub const LEN: usize = 8 + std::mem::size_of::<Position>();

    pub fn get_initial_leverage(&self) -> Result<u64> {
        math::checked_as_u64(math::checked_div(
            math::checked_mul(self.size_usd as u128, Perpetuals::BPS_POWER)?,
            self.collateral_usd as u128,
        )?)
    }
}