pub mod error;
pub mod state;
pub mod processor;

use anchor_lang::prelude::*;
use anchor_spl::token::TokenAccount;
use anchor_spl::token::{Mint, Token, Transfer};

use crate::error::ErrorCode;
use crate::state::Stage;
use crate::processor::{transfer_sol, transfer_token, to_close_account};
declare_id!("51aT3n5amGTQMT4P5V1xaAQYti83ajaqkcqjJrRPJKg9");

#[program]
pub mod escrow {
    use super::*;

    pub fn init(ctx: Context<InitState>) -> Result<()> {
        let state = &mut ctx.accounts.state_account;
        
        state.user = ctx.accounts.user.key().clone();
        state.mint = ctx.accounts.mint.key().clone();
        state.escrow_wallet = ctx.accounts.escrow_wallet_associate_account.key().clone();
        state.token_amount = 0;
        state.lamport_amount = 0;
        state.bumps.state_bump = *ctx.bumps.get("state_account").unwrap();
        msg!("The state account already created");
        Ok(())
    }

    pub fn deposit(ctx: Context<DepositInstruction>, amount: u64, is_native: bool) -> Result<()> {
        let state = &mut ctx.accounts.state_account;
        let bump_vector = state.bumps.state_bump.to_le_bytes();
        let mint_token = ctx.accounts.mint.key().clone();
        let inner = vec![
            b"state".as_ref(),
            ctx.accounts.user.key.as_ref(),
            mint_token.as_ref(),
            bump_vector.as_ref(),
        ];

        let outer = vec![inner.as_slice()];

        if is_native {
            state.lamport_amount += amount;
            transfer_sol(
                ctx.accounts.user.to_account_info(),
                state.to_account_info(),
                amount,
                outer.clone(),
                ctx.accounts.system_program.to_account_info()
            )?;
            return Ok(())
        }
        // handle transfer token
        state.token_amount += amount;
        transfer_token(
            ctx.accounts.user_associated_account.to_account_info(), 
            ctx.accounts.escrow_wallet_associate_account.to_account_info(),
            amount, 
            ctx.accounts.user.to_account_info(),
            outer.clone(),
            ctx.accounts.token_program.to_account_info()
        )?;
        
        state.stage = Stage::Deposit.to_code();
        Ok(())
    }

    pub fn withdraw(ctx: Context<WithDrawInstruction>, amount: u64, is_native: bool) -> Result<()> {
        let current_stage = Stage::from(ctx.accounts.state_account.stage)?;
        let is_valid_stage = current_stage == Stage::Deposit || current_stage == Stage::WithDraw;
        if !is_valid_stage {
            return Err(ErrorCode::InvalidStage.into());
        }
        let bump_vector = ctx.accounts.state_account.bumps.state_bump.to_le_bytes();
        let mint_token = ctx.accounts.mint.key().clone();
        let inner = vec![
            b"state".as_ref(),
            ctx.accounts.user.key.as_ref(),
            mint_token.as_ref(),
            bump_vector.as_ref(),
        ];
        let outer = vec![inner.as_slice()];

        if is_native {
            require_gte!(ctx.accounts.state_account.lamport_amount, amount, ErrorCode::InsufficientFunds);
            ctx.accounts.state_account.lamport_amount -= amount;
            transfer_sol(
                ctx.accounts.state_account.to_account_info(),
                ctx.accounts.user.to_account_info(),
                amount,
                outer.clone(),
                ctx.accounts.system_program.to_account_info()
            )?;
            return Ok(())
        }
        // handle withdraw token
        ctx.accounts.state_account.token_amount -= amount;
        require_gte!(ctx.accounts.state_account.token_amount, amount, ErrorCode::InsufficientFunds);
        transfer_token(
            ctx.accounts.escrow_wallet_associate_account.to_account_info(), 
            ctx.accounts.user_associated_account.to_account_info(),
            amount, 
            ctx.accounts.state_account.to_account_info(),
            outer.clone(),
            ctx.accounts.token_program.to_account_info()
        )?;
        ctx.accounts.state_account.stage = Stage::WithDraw.to_code();
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
    token_amount: u64,
    lamport_amount: u64,
    stage: u8,
    bumps: Bumps,
}

#[derive(Accounts)]
pub struct CheckStateExist<'info> {
    user: Signer<'info>,
    state_account: Account<'info, State>,
    system_program: Program<'info, System>,
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
    // PDA account
    #[account(mut)]
    state_account: Account<'info, State>,
    #[account(mut, token::mint=mint, token::authority=state_account)]
    escrow_wallet_associate_account: Account<'info, TokenAccount>,
    #[account(mut)]
    user: Signer<'info>,
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
    #[account(mut)]
    state_account: Account<'info, State>,
    #[account(mut)]
    escrow_wallet_associate_account: Account<'info, TokenAccount>,
    mint: Account<'info, Mint>,
    // refund wallet
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
