pub mod error;

use crate::error::ErrorDefine;
use anchor_lang::prelude::*;

declare_id!("4UW83kGzjovz2gujn87fcDG7oEhjp43VgjHvaoAaCtPc");

#[program]
pub mod counter_app {
    use super::*;

    pub fn init(ctx: Context<Create>) -> Result<()> {
        let counter_account = &mut ctx.accounts.compute_account;
        let authority = *ctx.accounts.authority.key;
        counter_account.total = 0;
        counter_account.authority = authority;
        Ok(())
    }

    pub fn change_authority(ctx: Context<ChangeAuthor>, new_auth: Pubkey) -> Result<()> {
        ctx.accounts.compute_account.authority = new_auth;
        Ok(())
    }

    pub fn add(ctx: Context<Add>, value: i8) -> Result<()> {
        let compute_account = &mut ctx.accounts.compute_account;
        // check conditions
        require_gte!(10, compute_account.total + value, ErrorDefine::AddError);
        compute_account.total += value;
        Ok(())
    }

    pub fn sub(ctx: Context<Sub>, value: i8) -> Result<()> {
        let compute_account = &mut ctx.accounts.compute_account;
        // check conditions
        require_gte!(-5, compute_account.total - value, ErrorDefine::SubError);
        compute_account.total -= value;
        Ok(())
    }
}

#[account]
pub struct ComputeAccount {
    pub total: i8,
    pub authority: Pubkey,
}

#[derive(Accounts)]
pub struct Create<'info> {
    #[account(init, payer=user, space = 64)]
    pub compute_account: Account<'info, ComputeAccount>,

    #[account(mut)]
    pub authority: Signer<'info>,

    #[account(mut)]
    pub user: Signer<'info>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct ChangeAuthor<'info> {
    #[account(mut, has_one=authority @ ErrorDefine::Unauthorized)]
    pub compute_account: Account<'info, ComputeAccount>,

    #[account()]
    pub authority: Signer<'info>,
}

#[derive(Accounts)]
pub struct Add<'info> {
    #[account(mut, has_one=authority @ ErrorDefine::Unauthorized)]
    pub compute_account: Account<'info, ComputeAccount>,

    #[account()]
    pub authority: Signer<'info>,
}

#[derive(Accounts)]
pub struct Sub<'info> {
    #[account(mut, has_one=authority @ ErrorDefine::Unauthorized)]
    pub compute_account: Account<'info, ComputeAccount>,

    #[account()]
    pub authority: Signer<'info>,
}
