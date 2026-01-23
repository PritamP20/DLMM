use crate::state::{Bin, BinArray, Position, LbPair};
use anchor_lang::prelude::*;
use anchor_spl::token::{self, Token, TokenAccount};

#[derive(AnchorDeserialize, AnchorSerialize, Clone)]
pub struct BinLiquidityDistribution{
    pub bin_id: i32,
    pub shares_to_bin: u128
}

#[derive(Accounts)]
pub struct RemoveLiquidity<'info>{
    #[account(mut)]
    pub lb_pair: Account<'info, LbPair>,

    #[account(
        mut,
        constraint = bin_array.lb_pair == lb_pair.key()
    )]
    pub bin_array: Account<'info, BinArray>,

    #[account(
        mut,
        constraint = position.lb_pair == lb_pair.key(),
        constraint = position.owner == user.key(),
    )]
    pub position: Account<'info, Position>,

    #[account(
        mut,
        constraint = user_token_x.owner == user.key(),
        constraint = user_token_x.mint == lb_pair.token_x_mint
    )]
    pub user_token_x: Account<'info, TokenAccount>,

    #[account(
        mut,
        constraint = user_token_y.owner == user.key(),
        constraint = user_token_y.mint == lb_pair.token_y_mint,
    )]
    pub user_token_y: Account<'info, TokenAccount>,

    #[account(
        mut,
        constraint = reserve_x.mint == lb_pair.token_x_mint,
        constraint = reserve_y.owner == lb_pair.key()
    )]
    pub reserve_x: Account<'info, TokenAccount>,

    #[account(
        mut,
        constraint = reserve_y.mint == lb_pair.token_y_mint,
        constraint = reserve_y.owner == lb_pair.key()
    )]
    pub reserve_y: Account<'info, TokenAccount>,

    #[account(mut)]
    pub user: Signer<'info>,
    pub token_program: Program<'info, Token>
}

pub fn handler(
    ctx: Context<RemoveLiquidity>,
    bin_liquidity_removal: Vec<BinLiquidityDistribution>,
)->Result<()>{
    let lb_pair = &mut ctx.accounts.lb_pair;
    let bin_array = &mut ctx.accounts.bin_array;
    let position = &mut ctx.accounts.position;

    let mut total_x_withdraw: u64 = 0;
    let mut total_y_withdraw: u64 = 0;

    for reduction in bin_liquidity_removal.iter() {
        let bin_id = reduction.bin_id;
        let shares_to_burn = reduction.shares_to_bin;

        let base_bin_id = (bin_array.index as i32) * 70;
        let bin_index = (bin_id - base_bin_id) as usize;
        require!(bin_index<70, ErrorCode::BinOutOfRange);

        let bin = &mut bin_array.bins[bin_index];

        require!(
            shares_to_burn <= bin.total_shares,
            ErrorCode::InsufficientLiquidity
        );
        require!(
            shares_to_burn <= bin.total_shares,
            ErrorCode::InsufficientLiquidity
        );

        if shares_to_burn == 0{
            continue;
        }

        // Calculate amounts to withdraw
        // amount = reserve * shares / total_shares
        let amount_x = (bin.reserve_x as u128)
            .checked_mul(shares_to_burn)
            .ok_or(ErrorCode::Overflow)?
            .checked_div(bin.total_shares)
            .ok_or(ErrorCode::Overflow)? as u64;



    }

    Ok(())
}

#[error_code]
pub enum ErrorCode{
    #[msg("Bin index out of range for this bin array")]
    BinOutOfRange,
    #[msg("Insufficient liquidity in bin")]
    InsufficientLiquidity,
    #[msg("Insufficient shares in position")]
    InsufficientShares,
    #[msg("Arithmetic overflow")]
    Overflow
}