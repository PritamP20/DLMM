use anchor_lang::prelude::*;
use anchor_spl::token::Mint;
use crate::state::LbPair;

#[derive(Accounts)]
pub struct InitializeLbPair<'info>{
    #[account(
        init,
        payer = user,
        space = LbPair::LEN,
        seeds = [b"lb_pair", token_x_mint.key().as_ref(), token_y_mint.key().as_ref()],
        bump
    )]
    pub lb_pair: Account<'info, LbPair>,

    #[account(mut)]
    pub user: Signer<'info>,
    pub token_x_mint: Account<'info, Mint>,
    pub token_y_mint: Account<'info, Mint>,
    pub system_program: Program<'info, System>
}

pub fn handler(ctx: Context<InitializeLbPair>, bin_step: u16)->Result<()>{
    let lb_pair = &mut ctx.accounts.lb_pair;
    lb_pair.token_x_mint = ctx.accounts.token_x_mint.key();
    lb_pair.token_y_mint = ctx.accounts.token_y_mint.key();
    lb_pair.bin_step = bin_step;
    lb_pair.active_bin_id = 0;
    lb_pair.bump = ctx.bumps.lb_pair;
    Ok(())
}