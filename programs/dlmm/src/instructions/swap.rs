use anchor_lang::prelude::*;
use anchor_spl::token::{self, Token, TokenAccount, Transfer};

use crate::{
    state::{BinArray, LbPair},
};

#[derive(Accounts)]
pub struct Swap<'info> {
    #[account(mut)]
    pub lb_pair: Account<'info, LbPair>,

    #[account(mut)]
    pub bin_array: Account<'info, BinArray>,

    #[account(mut)]
    pub user: Signer<'info>,

    #[account(
        mut,
        constraint = user_x_token.owner == user.key(),
        constraint = user_x_token.mint == lb_pair.token_x_mint
    )]
    pub user_x_token: Account<'info, TokenAccount>,

    #[account(
        mut,
        constraint = user_y_token.owner == user.key(),
        constraint = user_y_token.mint == lb_pair.token_y_mint
    )]
    pub user_y_token: Account<'info, TokenAccount>,

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

    pub token_program: Program<'info, Token>,
}

pub fn handler(
    ctx: Context<Swap>,
    amount_in: u64,
    min_amount_out: u64,
    swap_for_y: bool,
) -> Result<()> {
    let lb_pair = &mut ctx.accounts.lb_pair;
    let bin_array = &mut ctx.accounts.bin_array;

    let mut amount_in_left = amount_in;
    let mut amount_out = 0u64;
    let mut fees_collected = 0u64;
    let mut current_bin_id = lb_pair.active_bin_id;

    while amount_in_left > 0 {
        let base_bin_id = (bin_array.index as i32) * 70;
        let bin_index = (current_bin_id as i32 - base_bin_id) as usize;

        require!(bin_index < 70, ErrorCode::BinOutOfRange);
        let bin = &mut bin_array.bins[bin_index];

        if swap_for_y {
            require!(bin.reserve_y > 0, ErrorCode::InsufficientLiquidity);

            let max_amount_in = bin.reserve_x;
            let amount_in_this_bin = amount_in_left.min(max_amount_in);

            let fee = (amount_in_this_bin as u128 * lb_pair.base_free_rate as u128) / 10000;
            let amount_in_after_fee = amount_in_this_bin - fee as u64;

            let amount_out_this_bin = (amount_in_after_fee as u128 * bin.reserve_y as u128)
                / (bin.reserve_x as u128 + amount_in_after_fee as u128);

            bin.reserve_x = bin
                .reserve_x
                .checked_add(amount_in_after_fee)
                .ok_or(ErrorCode::Overflow)?;
            bin.reserve_y = bin
                .reserve_y
                .checked_sub(amount_out_this_bin as u64)
                .ok_or(ErrorCode::InsufficientLiquidity)?;

            amount_out += amount_out_this_bin as u64;
            fees_collected += fee as u64;
            amount_in_left -= amount_in_this_bin;

            if bin.reserve_y == 0 {
                current_bin_id -= 1; 
            }

        } else {
            require!(bin.reserve_x > 0, ErrorCode::InsufficientLiquidity);

            let max_amount_in = bin.reserve_y;
            let amount_in_this_bin = amount_in_left.min(max_amount_in);

            let fee = (amount_in_this_bin as u128 * lb_pair.base_free_rate as u128) / 10000;
            let amount_in_after_fee = amount_in_this_bin - fee as u64;

            let amount_out_this_bin = (amount_in_after_fee as u128 * bin.reserve_x as u128)
                / (bin.reserve_y as u128 + amount_in_after_fee as u128);

            bin.reserve_y = bin
                .reserve_y
                .checked_add(amount_in_after_fee)
                .ok_or(ErrorCode::Overflow)?;
            bin.reserve_x = bin
                .reserve_x
                .checked_sub(amount_out_this_bin as u64)
                .ok_or(ErrorCode::InsufficientLiquidity)?;

            amount_out += amount_out_this_bin as u64;
            fees_collected += fee as u64;
            amount_in_left -= amount_in_this_bin;

            if bin.reserve_x == 0 {
                current_bin_id += 1; 
            }
        }
    }

    require!(
        amount_out >= min_amount_out,
        ErrorCode::SlippageExceeded
    );

    if swap_for_y {
        let cpi_accounts = Transfer {
            from: ctx.accounts.user_x_token.to_account_info(),
            to: ctx.accounts.reserve_x.to_account_info(),
            authority: ctx.accounts.user.to_account_info(),
        };
        let cpi_ctx = CpiContext::new(
            ctx.accounts.token_program.to_account_info(),
            cpi_accounts,
        );
        token::transfer(cpi_ctx, amount_in)?;

        let seeds = &[
            b"lb_pair",
            lb_pair.token_x_mint.as_ref(),
            lb_pair.token_y_mint.as_ref(),
            &[lb_pair.bump],
        ];
        let signer = &[&seeds[..]];

        let cpi_accounts = Transfer {
            from: ctx.accounts.reserve_y.to_account_info(),
            to: ctx.accounts.user_y_token.to_account_info(),
            authority: lb_pair.to_account_info(),
        };
        let cpi_ctx = CpiContext::new_with_signer(
            ctx.accounts.token_program.to_account_info(),
            cpi_accounts,
            signer,
        );
        token::transfer(cpi_ctx, amount_out)?;

    } else {
        let cpi_accounts = Transfer {
            from: ctx.accounts.user_y_token.to_account_info(),
            to: ctx.accounts.reserve_y.to_account_info(),
            authority: ctx.accounts.user.to_account_info(),
        };
        let cpi_ctx = CpiContext::new(
            ctx.accounts.token_program.to_account_info(),
            cpi_accounts,
        );
        token::transfer(cpi_ctx, amount_in)?;

        let seeds = &[
            b"lb_pair",
            lb_pair.token_x_mint.as_ref(),
            lb_pair.token_y_mint.as_ref(),
            &[lb_pair.bump],
        ];
        let signer = &[&seeds[..]];

        let cpi_accounts = Transfer {
            from: ctx.accounts.reserve_x.to_account_info(),
            to: ctx.accounts.user_x_token.to_account_info(),
            authority: lb_pair.to_account_info(),
        };
        let cpi_ctx = CpiContext::new_with_signer(
            ctx.accounts.token_program.to_account_info(),
            cpi_accounts,
            signer,
        );
        token::transfer(cpi_ctx, amount_out)?;
    }
    if swap_for_y {
        lb_pair.reserve_x = lb_pair
            .reserve_x
            .checked_add(amount_in)
            .ok_or(ErrorCode::Overflow)?;
        lb_pair.reserve_y = lb_pair
            .reserve_y
            .checked_sub(amount_out)
            .ok_or(ErrorCode::Overflow)?;
    } else {
        lb_pair.reserve_y = lb_pair
            .reserve_y
            .checked_add(amount_in)
            .ok_or(ErrorCode::Overflow)?;
        lb_pair.reserve_x = lb_pair
            .reserve_x
            .checked_sub(amount_out)
            .ok_or(ErrorCode::Overflow)?;
    }

    lb_pair.active_bin_id = current_bin_id;

    msg!(
        "Swap complete: {} in, {} out, {} fees, active bin: {}",
        amount_in,
        amount_out,
        fees_collected,
        current_bin_id
    );

    Ok(())
}

#[error_code]
pub enum ErrorCode {
    #[msg("Bin index out of range for this bin array")]
    BinOutOfRange,
    #[msg("Insufficient liquidity in bin")]
    InsufficientLiquidity,
    #[msg("Arithmetic overflow")]
    Overflow,
    #[msg("Slippage exceeded")]
    SlippageExceeded,
}