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
}

#[derive(Accounts)]
pub struct Initialize {}
