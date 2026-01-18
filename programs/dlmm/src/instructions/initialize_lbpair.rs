use anchor_lang::prelude::*;
use crate::state::*;

#[derive(Accounts)]
pub struct InitializeLbPair<'info>{
    #[account(mut)]
}