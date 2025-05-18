use anchor_lang::{prelude::*, Bump};
use anchor_spl::{
    associated_token::AssociatedToken, token::Mint, token_interface::{
        transfer_checked, TokenAccount, TokenInterface, TransferChecked
    }
};
use crate::state::Escrow;

#[derive(account)]
pub struct Take {
    #[account(mut)]
    pub taker : Signer<'info>,

    #[account(
        mint::token_program
    )]
    pub mint_a : InterfaceAccount<'info , Mint>,
    
    #[account(
        mint::token_program
    )]
    pub mint_b : InterfaceAccount<'info , Mint>,
    
    #[account(
        mut , 
        payer = taker,
        associated_token::mint = mint_b,
        associated_token::token_program = token_program,
        associated_token::authority  = taker
    )]
    
    pub taker_ata_b : InterfaceAccount<'info, TokenAccount>, 
    
    #[account(
        init_if_needed,
        associated_token::mint = mint_a,
        associated_token::token_program = token_program,
        associated_token::authority  = maker
    )]
    pub taker_ata_a : InterfaceAccount<'info, TokenAccount>,
    
    #[account(
        close = maker,
        associated_token::mint = mint_b,
        associated_token::authority = escrow,
        associated_token::token_program = token_program
        
    )]
    pub vault : InterfaceAccount<'info , TokenAccount>,

    pub assosiated_token_program : Program<'info , AssociatedToken>,
    pub system_program : Program<'info , System>,
    pub token_program : InterfaceAccount<'info , TokenInterfaceAccount>

}

impl<'info> Take<'info>{
    pub fn deposit(&mut self , deposit_amount : u64 )-> Result<()>{

        let signer_seeds :[&[&[u8]];1]=
        [&[
            b"escrow",
            self.taker.to_account_info().key().as_ref(),
            &self.escrow.seed.to_le_bytes()[..],
            &[self.escrow.bump]
        ]];

        let tranfer_accounts = TransferChecked{
            from:self.taker_ata_b.to_account_info(),
            mint:self.mint_b,
            to:self.taker_ata_a.to_account_info(),
            authority:self.taker.to_account_info()
        };

        let cpi_ctx = CpiContext::new_with_signer(self.token_program.to_account_info(), tranfer_accounts, &signer_seeds);

    }
}