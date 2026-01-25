use crate::state::{Bin, BinArray, LbPair};
use anchor_lang::prelude::*;

#[derive(Accounts)]
#[instruction(index: i32)]
pub struct InitializeBinArray<'info> {
    #[account(mut)]
    pub lb_pair: Account<'info, LbPair>,

    #[account(
        init,
        payer = user,
        space = 8 + BinArray::LEN,
        seeds = [b"bin_array", lb_pair.key().as_ref(), &index.to_le_bytes()],
        bump
    )]
    pub bin_array: AccountLoader<'info, BinArray>,

    #[account(mut)]
    pub user: Signer<'info>,

    pub system_program: Program<'info, System>,
}

pub fn handler(ctx: Context<InitializeBinArray>, index: i32) -> Result<()> {
    let mut bin_array = ctx.accounts.bin_array.load_init()?;
    bin_array.lb_pair = ctx.accounts.lb_pair.key();
    bin_array.index = index as u16;
    bin_array.bump = ctx.bumps.bin_array;
    let base_bin_id = (index as u32) * 70;
    for (i, bins) in bin_array.bins.iter_mut().enumerate() {
        bins.reserve_x = 0;
        bins.reserve_y = 0;
        bins.fee_x_per_share = 0;
        bins.fee_y_per_share = 0;
        bins.bin_id = (base_bin_id + i as u32) as u16;
        bins.total_shares = 0;
    }
    msg!("Bin array initialised");
    Ok(())
}
