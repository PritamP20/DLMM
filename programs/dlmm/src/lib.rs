use anchor_lang::prelude::*;
mod instructions;
mod state;

declare_id!("Azjj9nPdEZToafRKgtU2DpCbscZKbcuAHU3sCvs2bSE4");

#[program]
pub mod dlmm {
    use super::*;

    pub fn initialize(ctx: Context<Initialize>) -> Result<()> {
        msg!("Greetings from: {:?}", ctx.program_id);
        Ok(())
    }
}

#[derive(Accounts)]
pub struct Initialize {}
