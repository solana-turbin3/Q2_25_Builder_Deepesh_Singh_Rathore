use anchor_lang::prelude::*;
use anchor_spl::{associated_token::AssociatedToken,
token_interface::{TokenAccount , TokenInterface ,
     Mint , TransferChecked , transfer_checked}
};
use crate::state::Escrow;

// all the account that we need to make this instruction happen.

#[derive(Accounts)]
#[instruction(seed:u64)]
// so that i can use the seed paased in the escrow pda to be used as parament in ctx. 
pub struct Make<'info> {
    #[account(mut)]
    pub maker : Signer<'info>,
    
    // 1.mint a and mint b are interface account
    // 2. now here mintA and mintB is token program 
    // interface account tell that it may be 2020 program may be 2022.
    #[account(
        mint::token_program = token_program
    )]
    pub mint_a : InterfaceAccount<'info , Mint>,
    
    #[account(
        mint::token_program = token_program
    )]
    pub mint_b : InterfaceAccount<'info , Mint>,

    #[account(
        mut,
        associated_token::mint = mint_a,
        associated_token::authority = maker,
        associated_token::token_program = token_program
    )]
    pub maker_ata_a : InterfaceAccount<'info , TokenAccount>,

    #[account(
        init,
        payer = maker,
        seeds= [b"escrow",maker.key().as_ref(),seeds.to_le_bytes().as_ref()],
        bump,
        space = 8 + Escrow::InitSpace
    )]
    pub escrow: Account<'info,Escrow>,

    #[account(
        init , 
        payer = maker,
        associated_token::mint = mint_a,
        associated_token::authority = escrow,
        associated_token::token_program = token_program
    )]
    pub vault : InterfaceAccount<'info , TokenAccount>,

    pub associated_token_program : Program<'info,AssociatedToken>,
    pub system_program : Program<'info , System>,
    // defining a token program 
    pub token_program : InterfaceAccount<'info , TokenInterfaceAccount>
}

impl<'info> Make<'info> {

    pub fn init_escrow(&mut self , seed : u64 , receive_amount : u64 , bumps : &MakeBumps)-> Result<()>{
        self.escrow.set_inner(Escrow { seed, maker: self.maker.key(), mint_a:self.mint_a.key(), mint_b: self.mint_b.key(), recieve_amount, bump: bumps.escrow });
        Ok(())
    }

    pub fn deposit(&mut self , deposit_amount : u64 , ) -> Result<()>{
        let cpi_program = self.token_program.to_account_info();

        let transfer_accounts = TransferChecked{
            from : self.maker_ata_a.to_account_info(),
            mint : self.mint_a.to_account_info(),
            to : self.vault.to_account_info(),
            authority : self.maker.to_account_info()
        };

        let cpi_ctx = CpiContext::new(cpi_program, transfer_accounts);

        transfer_checked(cpi_ctx, deposit_amount, self.mint_a.decimals)
        
    }
}

