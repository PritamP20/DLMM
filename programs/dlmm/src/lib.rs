use anchor_lang::prelude::*;
pub mod instructions;
pub mod state;

pub use instructions::*;

declare_id!("Azjj9nPdEZToafRKgtU2DpCbscZKbcuAHU3sCvs2bSE4");

#[program]
pub mod dlmm {
    use super::*;

    pub fn initialize(ctx: Context<Initialize>) -> Result<()> {
        msg!("Greetings from: {:?}", ctx.program_id);
        Ok(())
    }

    pub fn add_liquidity(
        ctx: Context<AddLiquidity>,
        amount_x: u64,
        amount_y: u64,
        bin_liquidity_dist: Vec<BinLiquidityDistribution>,
    ) -> Result<()> {
        instructions::add_liquidity::handler(ctx, amount_x, amount_y, bin_liquidity_dist)
    }

    pub fn remove_liquidity(
        ctx: Context<RemoveLiquidity>,
        bin_liquidity_removal: Vec<BinLiquidityReduction>,
    ) -> Result<()> {
        instructions::remove_liquidity::handler(ctx, bin_liquidity_removal)
    }

    pub fn swap(
        ctx: Context<Swap>,
        amount_in: u64,
        min_amount_out: u64,
        swap_for_y: bool,
    ) -> Result<()> {
        instructions::swap::handler(ctx, amount_in, min_amount_out, swap_for_y)
    }

    pub fn initialize_lb_pair(ctx: Context<InitializeLbPair>, bin_step: u16) -> Result<()> {
        instructions::initialize_lbpair::handler(ctx, bin_step)
    }

    pub fn initialize_bin_array(ctx: Context<InitializeBinArray>, index: i32) -> Result<()> {
        instructions::initialize_bin::handler(ctx, index as i32)
    }
}

#[derive(Accounts)]
pub struct Initialize {}
