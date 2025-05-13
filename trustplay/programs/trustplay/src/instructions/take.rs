use anchor_lang::{prelude::*, system_program::Transfer};
use anchor_spl::{
    associated_token::AssociatedToken, 
    token_interface::{
        TokenAccount,
        TokenInterface,
        Mint,
        TransferChecked,
        transfer_checked
    }
};


use crate::state::EscrowState;


#[derive(Accounts)]
pub struct Release<'info> {
    #[account(mut)]
    pub taker: Signer<'info>,               // remains payer
    #[account(mut, close = maker,
        has_one = maker,
        has_one = receiver,
        seeds = [b"escrow", escrow.maker.as_ref(), escrow.seed.to_le_bytes().as_ref()],
        bump = escrow.bump)]
    pub escrow: Account<'info, EscrowState>,

    /// The account that was set earlier
    pub receiver: Signer<'info>,

    /// Init the ATA for the receiver if needed
    #[account(
        init_if_needed,
        payer = taker,
        associated_token::mint = mint_a,
        associated_token::authority = receiver,
        token_program = token_program,
        associated_token_program = associated_token_program,
    )]
    pub receiver_ata: Account<'info, TokenAccount>,

    /// Vault holding tokens
    #[account(
        mut,
        associated_token::mint = mint_a,
        associated_token::authority = escrow,
        token_program = token_program,
    )]
    pub vault: Account<'info, TokenAccount>,

    pub mint_a: InterfaceAccount<'info, Mint>,
    pub token_program: InterfaceAccount<'info, TokenInterface>,
    pub associated_token_program: Program<'info, AssociatedToken>,
    pub system_program: Program<'info, System>,
}

impl<'info> Release<'info> {
    pub fn release(&mut self) -> Result<()> {
        // Seed derivation for PDA authority
        let seeds = &[
            b"escrow",
            self.escrow.maker.as_ref(),
            &self.escrow.seed.to_le_bytes(),
            &[self.escrow.bump],
        ];
        let signer_seeds = &[&seeds[..]];

        // Transfer tokens from the vault PDA to the receiver ATA
        let cpi_accounts = TransferChecked {
            from:      self.vault.to_account_info(),
            mint:      self.mint_a.to_account_info(),
            to:        self.receiver_ata.to_account_info(),
            authority: self.escrow.to_account_info(),
        };
        let cpi_ctx = CpiContext::new_with_signer(
            self.token_program.to_account_info(),
            cpi_accounts,
            signer_seeds,
        );
        transfer_checked(cpi_ctx, self.escrow.amount, self.mint_a.decimals)?;
        Ok(())
    }

    pub fn close(&mut self) -> Result<()> {
        // Close the vault and escrow PDA just as before
        let seeds = &[
            b"escrow",
            self.escrow.maker.as_ref(),
            &self.escrow.seed.to_le_bytes(),
            &[self.escrow.bump],
        ];
        let signer_seeds = &[&seeds[..]];

        // Close vault
        let cpi_close = CloseAccount {
            account:     self.vault.to_account_info(),
            destination: self.taker.to_account_info(),
            authority:   self.escrow.to_account_info(),
        };
        let cpi_ctx = CpiContext::new_with_signer(
            self.token_program.to_account_info(),
            cpi_close,
            signer_seeds,
        );
        close_account(cpi_ctx)?;

        // Escrow account is closed automatically by `close = maker` in struct
        Ok(())
    }
}


#[derive(Accounts)]
pub struct SetReceiver<'info> {
    #[account(
        mut,
        has_one = maker,
        seeds = [b"escrow", maker.key().as_ref(), escrow.seed.to_le_bytes().as_ref()],
        bump = escrow.bump,
    )]
    pub escrow: Account<'info, EscrowState>,
    pub maker: Signer<'info>,
    /// CHECK: Only storing the pubkey
    pub receiver: UncheckedAccount<'info>,
}

impl<'info> SetReceiver<'info> {
    pub fn set_receiver(&mut self) -> Result<()> {
        self.escrow.receiver = self.receiver.key();
        Ok(())
    }
}
