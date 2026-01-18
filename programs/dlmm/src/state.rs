use anchor_lang::prelude::*;

#[account]
pub struct LbPair{
    pub token_x_mint: Pubkey,
    pub token_y_mint: Pubkey,
    pub reserve_x: u64,
    pub reserve_y: u64,
    pub bin_step: u16,
    pub active_bin_id: u16,
    pub base_free_rate: u64,
    pub protocol_fee_rate: u64,
    pub volatility_accumulator: u64,
    pub last_update_timestamp: u64,
    pub bump:u8
}

impl LbPair{
    pub const LEN: usize = 8
        + 32 + 32
        + 8 + 8
        + 2 + 2
        + 8 + 8
        + 8 + 8
        + 1;
}

#[account]
pub struct Bin{
    reserve_x: u64,
    reserve_y: u64,
    bin_id: u16,
    total_shares: u128,
    fee_x_per_share: u128,
    fee_y_per_share: u128,
    bump:u8
}

impl Bin{
    const LEN: usize = 8
        + 8 + 8
        + 2 + 2
        + 16 + 16
        + 1;
}

#[account]
pub struct BinArray{
    pub lb_pair: Pubkey,
    pub index: u16,
    pub bins: [Bin; 70],
    pub bump: u8
}

impl BinArray{
    const LEN: usize = 8
        + 32
        + 2
        + 70 * Bin::LEN
        + 1;
}