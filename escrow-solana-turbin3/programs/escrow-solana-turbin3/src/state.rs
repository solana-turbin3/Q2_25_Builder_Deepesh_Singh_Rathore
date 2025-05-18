// in this case we don't have multiple state , so we just need
// to create a single state.rs file 
// reason of creating a instructions folder is 
// because there are numbers of instructions 

use anchor_lang::prelude::*;

// what state we are gonna store ???

// def we are gonna store account so lets define it
#[account]
#[derive(InitSpace)] // this will calculate the structs size.
pub struct Escrow {
    pub seed : u64 ,
    pub maker : Pubkey,
    pub mint_a : Pubkey,
    pub mint_b : Pubkey,
    pub recieve_amount : u64,
    pub bump : u8
}


