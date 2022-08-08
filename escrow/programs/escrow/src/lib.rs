pub mod error;
pub mod state;

use anchor_lang::prelude::*;
use anchor_spl::token::TokenAccount;
use anchor_spl::token::{Mint, Token, Transfer};

use crate::error::ErrorCode;
use crate::state::Stage;
declare_id!("51aT3n5amGTQMT4P5V1xaAQYti83ajaqkcqjJrRPJKg9");

#[program]
pub mod escrow {
    use super::*;

    pub fn init(ctx: Context<InitState>) -> Result<()> {
        let state = &mut ctx.accounts.state_account;
        
        state.user = ctx.accounts.user.key().clone();
        state.mint = ctx.accounts.mint.key().clone();
        state.escrow_wallet = ctx.accounts.escrow_wallet_associate_account.key().clone();
        state.amount = 0;
        state.bumps.state_bump = *ctx.bumps.get("state_account").unwrap();
        msg!("The state account already created");
        Ok(())
    }

    pub fn deposit(ctx: Context<DepositInstruction>, amount: u64) -> Result<()> {
        let state = &mut ctx.accounts.state_account;
        state.amount += amount;
        let bump_vector = state.bumps.state_bump.to_le_bytes();
        let mint_token = ctx.accounts.mint.key().clone();

        let inner = vec![
            b"state".as_ref(),
            ctx.accounts.user.key.as_ref(),
            mint_token.as_ref(),
            bump_vector.as_ref(),
        ];
        let outer = vec![inner.as_slice()];

        // init account
        let transfer_instruction = Transfer {
            from: ctx.accounts.user_associated_account.to_account_info(),
            to: ctx
                .accounts
                .escrow_wallet_associate_account
                .to_account_info(),
            authority: ctx.accounts.user.to_account_info(),
        };
        let cpi_ctx = CpiContext::new_with_signer(
            ctx.accounts.token_program.to_account_info(),
            transfer_instruction,
            outer.as_slice(),
        );

        anchor_spl::token::transfer(cpi_ctx, state.amount)?;
        
        state.stage = Stage::Deposit.to_code();
        Ok(())
    }

    pub fn withdraw(ctx: Context<WithDrawInstruction>, amount: u64) -> Result<()> {
        let current_stage = Stage::from(ctx.accounts.state_account.stage)?;
        require_gte!(ctx.accounts.state_account.amount, amount, ErrorCode::InsufficientFunds);
        let is_valid_stage = current_stage == Stage::Deposit || current_stage == Stage::WithDraw;
        if !is_valid_stage {
            return Err(ErrorCode::InvalidStage.into());
        }
        ctx.accounts.state_account.amount -= amount;
        let state_bump = ctx.accounts.state_account.bumps.state_bump;
        let bump_vector = state_bump.to_le_bytes();
        let mint_token = ctx.accounts.mint.key().clone();
        let inner = vec![
            b"state".as_ref(),
            ctx.accounts.user.key.as_ref(),
            mint_token.as_ref(),
            bump_vector.as_ref(),
        ];
        let outer = vec![inner.as_slice()];

        let transfer_instruction = Transfer {
            from: ctx
                .accounts
                .escrow_wallet_associate_account
                .to_account_info(),
            to: ctx.accounts.user_associated_account.to_account_info(),
            authority: ctx.accounts.state_account.to_account_info(),
        };
        let cpi_ctx = CpiContext::new_with_signer(
            ctx.accounts.token_program.to_account_info(),
            transfer_instruction,
            outer.as_slice(),
        );
        anchor_spl::token::transfer(cpi_ctx, amount)?;
        ctx.accounts.state_account.stage = Stage::WithDraw.to_code();
        // close Account when dont use any where again
        // let is_close = {
        //     ctx.accounts.escrow_wallet_associate_account.reload()?;
        //     ctx.accounts.escrow_wallet_associate_account.amount == 0
        // };

        // if is_close {
        //     let ca = CloseAccount {
        //         account: ctx
        //             .accounts
        //             .escrow_wallet_associate_account
        //             .to_account_info(),
        //         destination: ctx.accounts.user.to_account_info(),
        //         authority: ctx.accounts.state_account.to_account_info(),
        //     };
        //     let cpi_ctx = CpiContext::new_with_signer(
        //         ctx.accounts.token_program.to_account_info(),
        //         ca,
        //         outer.as_slice(),
        //     );
        //     anchor_spl::token::close_account(cpi_ctx)?;
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
