use anchor_lang::{accounts::program, prelude::*};
use anchor_spl::{associated_token::AssociatedToken, token::CloseAccount, token_interface::{transfer_checked, Mint, TokenAccount, TokenInterface, TransferChecked}
};
use crate::state::Escrow;

#[derive(Accounts)]
pub struct Refund<'info>{
    #[account(mut)]
    pub maker : Signer<'info>,

    #[account(
        mint::token_program = token_program
    )]
    pub mint_a : InterfaceAccount<'info , Mint>,

    #[account(
        mut,
        associated_token::mint = mint_a,
        associated_token::authority = maker,
        associated_token::token_program = token_program
    )]
    pub maker_ata_a : InterfaceAccount<'info , TokenAccount>,

    #[account(
        mut,
        close = maker , 
        has_one = mint_a,
        has_one = maker,
        seeds= [b"escrow",maker.key().as_ref(),seeds.to_le_bytes().as_ref()],
        bump = escrow.bump,
    )]
    pub escrow : Account<'info,Escrow>,
    
    #[account( 
        mut,
        associated_token::mint = mint_a,
        associated_token::authority = escrow,
        associated_token::token_program = token_program
    )]
    pub vault : InterfaceAccount<'info , TokenAccount>,

    pub associated_token_program : Program<'info,AssociatedToken>,
    pub system_program : Program<'info , System>,
    pub token_program : InterfaceAccount<'info , TokenInterfaceAccount>
}

impl<'info> Refund<'info>{
    pub fn refund_to_makerATA_and_close_vault(&mut self){

        let cpi_program = self.token_program.to_account_info();
        
        let signer_seeds :[&[&[u8]];1]=
        [&[
            b"escrow",
            self.maker.to_account_info().key().as_ref(),
            &self.escrow.seed.to_le_bytes()[..],
            &[self.escrow.bump]
        ]];

        let tranfer_accounts = TransferChecked{
            from:self.vault.to_account_info(),
            to:self.maker_ata_a.to_account_info(),
            mint:self.mint_a,
            authority:self.escrow
        };

        let cpi_ctx = CpiContext::new_with_signer(cpi_program, tranfer_accounts, &signer_seeds);

        transfer_checked(cpi_ctx, self.vault.amount, self.mint_a.decimals)?;
        // we are writing ?; because we don;t want to return the above line , like we want but after that we will not be able to write other code.
        //hence done that.
        // after that we have run the transaction instruction , we need to close the vault.
        
        let close_account = CloseAccount{
            account : self.vault.to_account_info(),
            destination:self.maker.to_account_info(),
            authority:self.escrow.to_account_info(),
        };

        let close_cpi_ctx = CpiContext::new_with_signer(cpi_program , close_account, &signer_seeds);

        close_account(close_cpi_ctx);
    }
}