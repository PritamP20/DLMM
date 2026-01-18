use anchor_lang::prelude::*;
use crate::state::{Bin, BinArray, LbPair};

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

pub fn handler(ctx: Context<InitializeBinArray>, index:u16)->Result<()>{
    let bin_array =&mut ctx.accounts.bin_array;
    bin_array.index = index;
    bin_array.bump = ctx.bumps.bin_array;
    for bins in bin_array.bins.iter_mut() {
        bins.reserve_x = 0;
        bins.reserve_y = 0;
        bins.fee_x_per_share = 0;
        bins.fee_y_per_share = 0;
        bins.bin_id = 0;
        bins.total_shares = 0;
    }
    msg!("Bin array initialised");
    Ok(())
}