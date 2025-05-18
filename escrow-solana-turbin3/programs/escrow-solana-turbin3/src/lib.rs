use anchor_lang::prelude::*;

pub mod instructions;
pub mod state;

pub use instruction;
pub use state;



declare_id!("AfxA7zdgcUoyNb2McwehJWdoYChMsjZxMjH44yzoqyai");

#[program]
pub mod escrow_solana_turbin3 {
    use super::*;

    pub fn make(ctx: Context<Make>, seed : u64 , deposit_amout : u64 , receive_amout : u64) -> Result<()> {
        
        Ok(())
    }
}

#[derive(Accounts)]
pub struct Initialize {}

// since we are having multiple instruction , hence creating modules for 
// that
// So , creating modules names instructions.