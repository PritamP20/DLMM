use anchor_lang::prelude::*;
use crate::state::{BinArray, LbPair};

#[derive(Accounts)]
#[instruction(index: i32)]
pub struct InitializeBinArray<'info> {
    #[account(mut)]
    pub lb_pair: Account<'info, LbPair>,

    #[account(
        init,
        payer = user,
        space = BinArray::LEN,
        seeds = [b"bin_array", lb_pair.key().as_ref(), &index.to_le_bytes()],
        bump
    )]
    pub bin_array: Account<'info, BinArray>,

    #[account(mut)]
    pub user: Signer<'info>,

    pub system_program: Program<'info, System>,
}
