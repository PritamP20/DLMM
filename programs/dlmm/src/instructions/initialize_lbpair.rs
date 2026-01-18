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