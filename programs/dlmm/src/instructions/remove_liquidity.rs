use crate::state::{BinArray, LbPair, Position};
use anchor_lang::prelude::*;
use anchor_spl::token::{self, Token, TokenAccount, Transfer};

#[derive(AnchorSerialize, AnchorDeserialize, Clone)]
pub struct BinLiquidityReduction {
    pub bin_id: i32,
    pub shares_to_burn: u128,
}

#[derive(Accounts)]
pub struct RemoveLiquidity<'info> {
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
        constraint = position.owner == user.key()
    )]
    pub position: Account<'info, Position>,

    #[account(
        mut,
        constraint = user_token_x.owner == user.key(),
        constraint = user_token_x.mint == lb_pair.token_x_mint,
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
        constraint = reserve_x.owner == lb_pair.key()
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
    pub token_program: Program<'info, Token>,
}

pub fn handler(
    ctx: Context<RemoveLiquidity>,
    bin_liquidity_removal: Vec<BinLiquidityReduction>,
) -> Result<()> {
    let lb_pair = &mut ctx.accounts.lb_pair;
    let bin_array = &mut ctx.accounts.bin_array;
    let position = &mut ctx.accounts.position;

    let mut total_x_withdrawn: u64 = 0;
    let mut total_y_withdrawn: u64 = 0;

    for reduction in bin_liquidity_removal.iter() {
        let bin_id = reduction.bin_id;
        let shares_to_burn = reduction.shares_to_burn;

        // Find bin index
        let base_bin_id = (bin_array.index as i32) * 70;
        let bin_index = (bin_id - base_bin_id) as usize;
        require!(bin_index < 70, ErrorCode::BinOutOfRange);

        let bin = &mut bin_array.bins[bin_index];

        require!(
            shares_to_burn <= bin.total_shares,
            ErrorCode::InsufficientLiquidity
        );
        require!(
            shares_to_burn <= position.liquidity_shares[bin_index],
            ErrorCode::InsufficientShares
        );

        if shares_to_burn == 0 {
            continue;
        }

        // Calculate amounts to withdraw
        // amount = reserve * shares / total_shares
        let amount_x = (bin.reserve_x as u128)
            .checked_mul(shares_to_burn)
            .ok_or(ErrorCode::Overflow)?
            .checked_div(bin.total_shares)
            .ok_or(ErrorCode::Overflow)? as u64;

        let amount_y = (bin.reserve_y as u128)
            .checked_mul(shares_to_burn)
            .ok_or(ErrorCode::Overflow)?
            .checked_div(bin.total_shares)
            .ok_or(ErrorCode::Overflow)? as u64;

        // Update bin
        bin.reserve_x = bin
            .reserve_x
            .checked_sub(amount_x)
            .ok_or(ErrorCode::Overflow)?;
        bin.reserve_y = bin
            .reserve_y
            .checked_sub(amount_y)
            .ok_or(ErrorCode::Overflow)?;
        bin.total_shares = bin
            .total_shares
            .checked_sub(shares_to_burn)
            .ok_or(ErrorCode::Overflow)?;

        // Update position
        position.liquidity_shares[bin_index] = position.liquidity_shares[bin_index]
            .checked_sub(shares_to_burn)
            .ok_or(ErrorCode::Overflow)?;

        total_x_withdrawn += amount_x;
        total_y_withdrawn += amount_y;

        msg!(
            "Withdrawn from bin {}: {} X, {} Y, {} shares",
            bin_id,
            amount_x,
            amount_y,
            shares_to_burn
        );
    }

    // Signer seeds for PDA
    let seeds = &[
        b"lb_pair",
        lb_pair.token_x_mint.as_ref(),
        lb_pair.token_y_mint.as_ref(),
        &[lb_pair.bump],
    ];
    let signer = &[&seeds[..]];

    // Transfer X
    if total_x_withdrawn > 0 {
        let cpi_accounts = Transfer {
            from: ctx.accounts.reserve_x.to_account_info(),
            to: ctx.accounts.user_token_x.to_account_info(),
            authority: lb_pair.to_account_info(),
        };
        let cpi_ctx = CpiContext::new_with_signer(
            ctx.accounts.token_program.to_account_info(),
            cpi_accounts,
            signer,
        );
        token::transfer(cpi_ctx, total_x_withdrawn)?;
    }

    // Transfer Y
    if total_y_withdrawn > 0 {
        let cpi_accounts = Transfer {
            from: ctx.accounts.reserve_y.to_account_info(),
            to: ctx.accounts.user_token_y.to_account_info(),
            authority: lb_pair.to_account_info(),
        };
        let cpi_ctx = CpiContext::new_with_signer(
            ctx.accounts.token_program.to_account_info(),
            cpi_accounts,
            signer,
        );
        token::transfer(cpi_ctx, total_y_withdrawn)?;
    }

    // Update lb_pair reserves
    lb_pair.reserve_x = lb_pair
        .reserve_x
        .checked_sub(total_x_withdrawn)
        .ok_or(ErrorCode::Overflow)?;
    lb_pair.reserve_y = lb_pair
        .reserve_y
        .checked_sub(total_y_withdrawn)
        .ok_or(ErrorCode::Overflow)?;

    msg!(
        "Remove liquidity complete: {} X, {} Y",
        total_x_withdrawn,
        total_y_withdrawn
    );

    Ok(())
}

#[error_code]
pub enum ErrorCode {
    #[msg("Bin index out of range for this bin array")]
    BinOutOfRange,
    #[msg("Insufficient liquidity in bin")]
    InsufficientLiquidity,
    #[msg("Insufficient shares in position")]
    InsufficientShares,
    #[msg("Arithmetic overflow")]
    Overflow,
}
