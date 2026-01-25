use crate::state::{BinArray, LbPair, Position};
use anchor_lang::prelude::*;
use anchor_spl::token::{self, Token, TokenAccount, Transfer};

pub const BASIS_POINT_MAX: u64 = 10000;

#[derive(AnchorSerialize, AnchorDeserialize, Clone)]
pub struct BinLiquidityDistribution {
    pub delta_id: i32, // Offset from active bin (-1, 0, +1, etc.)
    pub dist_x: u16,   // Percentage of amount_x in basis points (10000 = 100%)
    pub dist_y: u16,   // Percentage of amount_y in basis points
}

#[derive(Accounts)]
pub struct AddLiquidity<'info> {
    #[account(mut)]
    pub lb_pair: Account<'info, LbPair>,

    #[account(
        mut,
        constraint = bin_array.load()?.lb_pair == lb_pair.key()
    )]
    pub bin_array: AccountLoader<'info, BinArray>,

    #[account(
        init_if_needed,
        payer = user,
        space = 8 + Position::LEN,
        seeds = [b"position", lb_pair.key().as_ref(), user.key().as_ref()],
        bump
    )]
    pub position: Box<Account<'info, Position>>,

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
    )]
    pub reserve_x: Box<Account<'info, TokenAccount>>,

    #[account(
        mut,
        constraint = reserve_y.mint == lb_pair.token_y_mint,
    )]
    pub reserve_y: Box<Account<'info, TokenAccount>>,

    #[account(mut)]
    pub user: Signer<'info>,
    pub token_program: Program<'info, Token>,
    pub system_program: Program<'info, System>,
}

pub fn handler(
    ctx: Context<AddLiquidity>,
    amount_x: u64,
    amount_y: u64,
    bin_liquidity_dist: Vec<BinLiquidityDistribution>,
) -> Result<()> {
    let lb_pair = &mut ctx.accounts.lb_pair;
    let mut bin_array = ctx.accounts.bin_array.load_mut()?;
    let position = &mut ctx.accounts.position;

    let mut total_x_deposited: u64 = 0;
    let mut total_y_deposited: u64 = 0;

    if position.lb_pair == Pubkey::default() {
        position.lb_pair = lb_pair.key();
        position.owner = ctx.accounts.user.key();
    }

    for dist in bin_liquidity_dist.iter() {
        let target_bin_id = (lb_pair.active_bin_id as i32) + dist.delta_id;

        let base_bin_id = (bin_array.index as i32) * 70;
        let bin_index = (target_bin_id - base_bin_id) as usize;
        require!(bin_index < 70, ErrorCode::BinOutOfRange);

        let bin = &mut bin_array.bins[bin_index];

        let deposit_x = (amount_x as u128 * dist.dist_x as u128 / BASIS_POINT_MAX as u128) as u64;
        let deposit_y = (amount_y as u128 * dist.dist_y as u128 / BASIS_POINT_MAX as u128) as u64;

        let shares: u128 = if bin.total_shares == 0 {
            let product = (deposit_x as u128) * (deposit_y as u128);
            integer_sqrt(product)
        } else {
            let shares_x = if bin.reserve_x > 0 {
                (deposit_x as u128) * bin.total_shares / (bin.reserve_x as u128)
            } else {
                u128::MAX
            };
            let shares_y = if bin.reserve_y > 0 {
                (deposit_y as u128) * bin.total_shares / (bin.reserve_y as u128)
            } else {
                u128::MAX
            };
            shares_x.min(shares_y)
        };

        require!(shares > 0, ErrorCode::InsufficientLiquidity);

        bin.reserve_x = bin
            .reserve_x
            .checked_add(deposit_x)
            .ok_or(ErrorCode::Overflow)?;
        bin.reserve_y = bin
            .reserve_y
            .checked_add(deposit_y)
            .ok_or(ErrorCode::Overflow)?;
        bin.total_shares = bin
            .total_shares
            .checked_add(shares)
            .ok_or(ErrorCode::Overflow)?;

        position.liquidity_shares[bin_index] = position.liquidity_shares[bin_index]
            .checked_add(shares)
            .ok_or(ErrorCode::Overflow)?;

        if target_bin_id < position.lower_bin_id {
            position.lower_bin_id = target_bin_id;
        }
        if target_bin_id > position.upper_bin_id {
            position.upper_bin_id = target_bin_id;
        }

        total_x_deposited += deposit_x;
        total_y_deposited += deposit_y;

        msg!(
            "Deposited to bin {}: {} X, {} Y, {} shares",
            target_bin_id,
            deposit_x,
            deposit_y,
            shares
        );
    }

    if total_x_deposited > 0 {
        let cpi_accounts = Transfer {
            from: ctx.accounts.user_token_x.to_account_info(),
            to: ctx.accounts.reserve_x.to_account_info(),
            authority: ctx.accounts.user.to_account_info(),
        };
        let cpi_ctx = CpiContext::new(ctx.accounts.token_program.to_account_info(), cpi_accounts);
        token::transfer(cpi_ctx, total_x_deposited)?;
    }

    if total_y_deposited > 0 {
        let cpi_accounts = Transfer {
            from: ctx.accounts.user_token_y.to_account_info(),
            to: ctx.accounts.reserve_y.to_account_info(),
            authority: ctx.accounts.user.to_account_info(),
        };
        let cpi_ctx = CpiContext::new(ctx.accounts.token_program.to_account_info(), cpi_accounts);
        token::transfer(cpi_ctx, total_y_deposited)?;
    }

    lb_pair.reserve_x = lb_pair
        .reserve_x
        .checked_add(total_x_deposited)
        .ok_or(ErrorCode::Overflow)?;
    lb_pair.reserve_y = lb_pair
        .reserve_y
        .checked_add(total_y_deposited)
        .ok_or(ErrorCode::Overflow)?;

    msg!(
        "Add liquidity complete: {} X, {} Y",
        total_x_deposited,
        total_y_deposited
    );

    Ok(())
}

fn integer_sqrt(n: u128) -> u128 {
    if n == 0 {
        return 0;
    }
    let mut x = n;
    let mut y = (x + 1) / 2;
    while y < x {
        x = y;
        y = (x + n / x) / 2;
    }
    x
}

#[error_code]
pub enum ErrorCode {
    #[msg("Bin index out of range for this bin array")]
    BinOutOfRange,
    #[msg("Insufficient liquidity to mint shares")]
    InsufficientLiquidity,
    #[msg("Arithmetic overflow")]
    Overflow,
}
