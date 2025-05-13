use anchor_lang::prelude::*;

#[account]
#[derive(InitSpace)]
pub struct EscrowState{
  pub seed : u64 , 
  pub maker : Pubkey,
  pub mint_a : Pubkey,
  pub amount : u64 , 
  pub bump : u8,
  pub receiver : Pubkey
}