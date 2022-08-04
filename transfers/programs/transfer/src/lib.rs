use anchor_lang::prelude::*;
use anchor_spl::token::{Token, Transfer};

declare_id!("6uduzpBoMfGfcRMkFpd8u4PPDJayS8DijDtTXpGHYf1D");

#[program]
pub mod transfer {
    use super::*;

    pub fn transfer_token(ctx: Context<TransferToken>, amount: u64) -> Result<()> {
        let transfer_instruction = Transfer {
            from: ctx.accounts.from.to_account_info(),
            to: ctx.accounts.to.to_account_info(),
            authority: ctx.accounts.from_authority.to_account_info(),
        };
        let cpi_program = ctx.accounts.token_program.to_account_info();
        let cpi_ctx = CpiContext::new(cpi_program, transfer_instruction);
        anchor_spl::token::transfer(cpi_ctx, amount)?;
        Ok(())
    }
}

#[derive(Accounts)]
pub struct TransferToken<'info> {
    pub token_program: Program<'info, Token>,
    /// CHECK:
    #[account(mut)]
    pub from: UncheckedAccount<'info>,
    /// CHECK:
    #[account(mut)]
    pub to: AccountInfo<'info>,
    pub from_authority: Signer<'info>,
}
