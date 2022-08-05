use anchor_lang::prelude::*;
use anchor_spl::associated_token::AssociatedToken;
use anchor_spl::token::{Mint, Token, TokenAccount, Transfer};

declare_id!("6uduzpBoMfGfcRMkFpd8u4PPDJayS8DijDtTXpGHYf1D");

#[program]
pub mod transfer {
    use super::*;

    pub fn transfer(ctx: Context<TransferToken>, amount: u64) -> Result<()> {
        let source = &ctx.accounts.sender_associate;
        let destination = &ctx.accounts.receiver_associate;
        let transfer_accounts = Transfer {
            from: source.to_account_info(),
            to: destination.to_account_info(),
            authority: ctx.accounts.sender.to_account_info(),
        };
        // create cpi context for transfer
        let cpi_ctx = CpiContext::new(
            ctx.accounts.token_program.to_account_info(),
            transfer_accounts,
        );
        // transfer
        anchor_spl::token::transfer(cpi_ctx, amount)?;
        Ok(())
    }
}

#[derive(Accounts)]
pub struct TransferToken<'info> {
    #[account(mut, constraint = sender.data_is_empty() && sender.lamports() > 0)]
    pub sender: Signer<'info>,

    #[account(mut, associated_token::mint = mint, associated_token::authority = sender)]
    pub sender_associate: Account<'info, TokenAccount>,
    /// CHECK: 
    #[account(mut, constraint = receiver.data_is_empty())]
    pub receiver: AccountInfo<'info>,

    #[account(mut, associated_token::mint = mint, associated_token::authority = receiver)]
    pub receiver_associate: Account<'info, TokenAccount>,
    pub mint: Account<'info, Mint>,
    pub token_program: Program<'info, Token>,
    pub system_program: Program<'info, System>,
    pub associated_token_program: Program<'info, AssociatedToken>,
}
