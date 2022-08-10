pub mod error;
pub mod state;
pub mod processor;

use anchor_lang::prelude::*;
use anchor_spl::token::TokenAccount;
use anchor_spl::token::{Mint, Token, Transfer};

use crate::error::ErrorCode;
use crate::state::Stage;
use crate::processor::{transfer_token, to_close_account};
declare_id!("C3iRXuEMdHwVUXoPtsMBKps5eVS9KLh7o57gpsgQuNCj");

#[program]
pub mod escrow {
    use super::*;

    pub fn init(ctx: Context<InitState>) -> Result<()> {
        let state = &mut ctx.accounts.state_account;
        
        state.user = ctx.accounts.user.key();
        state.mint = ctx.accounts.mint.key();
        state.escrow_wallet = ctx.accounts.escrow_wallet_associate_account.key();
        state.amount = 0;
        state.bumps.state_bump = *ctx.bumps.get("state_account").unwrap();
        msg!("The state account created");
        Ok(())
    }

    pub fn deposit(ctx: Context<DepositInstruction>, amount: u64) -> Result<()> {
        let state = &mut ctx.accounts.state_account;
        let bump = state.bumps.state_bump;
        // handle transfer token
        state.amount += amount;
        let seeds = &[&[b"state", bytemuck::bytes_of(&bump)][..]];
        transfer_token(
            ctx.accounts.user_associated_account.to_account_info(), 
            ctx.accounts.escrow_wallet_associate_account.to_account_info(), 
            ctx.accounts.user.to_account_info(),
            amount,
            seeds,
            ctx.accounts.token_program.to_account_info()
        )?;
        
        state.stage = Stage::Deposit.to_code();
        Ok(())
    }

    pub fn withdraw(ctx: Context<WithDrawInstruction>, amount: u64) -> Result<()> {
        require_gte!(ctx.accounts.state_account.amount, amount, ErrorCode::InsufficientFunds);
        //
        let state = &mut ctx.accounts.state_account;
        let current_stage = Stage::from(state.stage)?;
        let is_valid_stage = current_stage == Stage::Deposit || current_stage == Stage::WithDraw;
        if !is_valid_stage {
            return Err(ErrorCode::InvalidStage.into());
        }
        let user_seed = ctx.accounts.user.key().clone();
        let mint_seed = ctx.accounts.mint.key().clone();
        let seeds = &[&[b"state", user_seed.as_ref(), mint_seed.as_ref(), bytemuck::bytes_of(&state.bumps.state_bump)][..]];
        transfer_token(
            ctx.accounts.escrow_wallet_associate_account.to_account_info(), 
            ctx.accounts.user_associated_account.to_account_info(),
            state.to_account_info(),
            amount,
            seeds,
            ctx.accounts.token_program.to_account_info()
        )?;
        state.stage = Stage::WithDraw.to_code();
        state.amount -= amount;
        
        // close Account when dont use any where again
        // let is_close = {
        //     ctx.accounts.escrow_wallet_associate_account.reload()?;
        //     ctx.accounts.escrow_wallet_associate_account.amount == 0
        // };

        // if is_close {
            // to_close_account(ctx.accounts.escrow_wallet_associate_account.to_account_info(),
            // ctx.accounts.user.to_account_info(),
            // ctx.accounts.state_account.to_account_info(),
            // outer.clone(),
            // ctx.accounts.token_program.to_account_info())?;
        // }

        Ok(())
    }
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone, Default)]
pub struct Bumps {
    pub state_bump: u8,
    pub wallet_bump: u8,
}
#[account]
pub struct State {
    user: Pubkey,
    mint: Pubkey,
    // associated account
    escrow_wallet: Pubkey,
    amount: u64,
    stage: u8,
    bumps: Bumps,
}

#[derive(Accounts)]
pub struct InitState<'info> {
    #[account(mut)]
    user: Signer<'info>,
    // PDA account
    #[account(
        init,
        payer = user,
        space = 131,
        seeds=[b"state", user.key().as_ref(), mint.key().as_ref()],
        bump,    
    )]
    state_account: Account<'info, State>,
    #[account(
        init,
        payer=user,
        seeds=[b"wallet", user.key().as_ref(), mint.key().as_ref()],
        bump,
        token::mint=mint,
        token::authority=state_account,
    )]
    escrow_wallet_associate_account: Account<'info, TokenAccount>,
    mint: Account<'info, Mint>,
    system_program: Program<'info, System>,
    token_program: Program<'info, Token>,
    rent: Sysvar<'info, Rent>,
}

#[derive(Accounts)]
pub struct DepositInstruction<'info> {
    #[account(mut)]
    user: Signer<'info>,
    // PDA account
    #[account(mut, 
        seeds = [b"state", user.key().as_ref(), mint.key().as_ref()],
        bump = state_account.bumps.state_bump
    )]
    state_account: Account<'info, State>,
    #[account(mut, 
        token::mint=mint,
        token::authority=state_account
    )]
    escrow_wallet_associate_account: Account<'info, TokenAccount>,
    
    mint: Account<'info, Mint>,

    #[account(
        mut,
        token::mint=mint,
        token::authority=user
    )]
    user_associated_account: Account<'info, TokenAccount>,

    system_program: Program<'info, System>,
    token_program: Program<'info, Token>,
}

#[derive(Accounts)]
pub struct WithDrawInstruction<'info> {
    #[account(mut)]
    user: Signer<'info>,
    #[account(mut,
        seeds = [b"state", user.key().as_ref(), mint.key().as_ref()],
        bump = state_account.bumps.state_bump
    )]
    state_account: Account<'info, State>,
    #[account(mut, 
        token::mint=mint,
        token::authority=state_account
    )]
    escrow_wallet_associate_account: Account<'info, TokenAccount>,
    mint: Account<'info, Mint>,
    #[account(
        mut,
        token::mint = mint,
        token::authority = user
    )]
    user_associated_account: Account<'info, TokenAccount>,
    // system
    system_program: Program<'info, System>,
    token_program: Program<'info, Token>,
}
