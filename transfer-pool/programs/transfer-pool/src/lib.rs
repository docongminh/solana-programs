pub mod error;

use crate::error::ErrorCode;
use anchor_lang::prelude::*;
use anchor_spl::associated_token::*;
use anchor_spl::token::TokenAccount;
use anchor_spl::token::{CloseAccount, Mint, Token, Transfer};

declare_id!("Wk1uGMfZR6YhTjLAaUD1e944VcrgKvZXsFVPonjy1yD");

#[program]
pub mod transfer {
    use super::*;

    const SEED: &[u8] = b"escrow";
    pub fn deposit(
        ctx: Context<DepositInstruction>,
        amount: u64,
        stage_bump: u8,
        wallet_bump: u8,
    ) -> Result<()> {
        let state = &mut ctx.accounts.application_state;
        state.user = ctx.accounts.user.key().clone();
        state.mint = ctx.accounts.mint.key().clone();
        state.escrow_wallet = ctx.accounts.escrow_wallet_state.key().clone();
        state.amount = amount;

        let bump_vector = stage_bump.to_le_bytes();
        let mint_of_token_being_sent_pk = ctx.accounts.mint.key().clone();

        let inner = vec![
            b"state".as_ref(),
            ctx.accounts.user.key.as_ref(),
            ctx.accounts.user_receiving.key.as_ref(),
            mint_of_token_being_sent_pk.as_ref(),
            bump_vector.as_ref(),
        ];
        let outer = vec![inner.as_slice()];

        // init account
        let transfer_instruction = Transfer {
            from: ctx.accounts.wallet_to_withdraw.to_account_info(),
            to: ctx.accounts.escrow_wallet_state.to_account_info(),
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

    // pub fn with_draw(
    //     ctx: Context<WithDrawInstruction>,
    //     application_idx: u64,
    //     state_bump: u8,
    //     wallet_bump: u8,
    // ) -> Result<()> {
    //     let current_stage = Stage::from(ctx.accounts.application_state.stage)?;
    //     let is_valid_stage = current_stage == Stage::Deposit || current_stage == Stage::WithDraw;
    //     if !is_valid_stage {
    //         return Err(ErrorCode::InvalidStage.into());
    //     }

    //     let bump_vector = state_bump.to_le_bytes();
    //     let mint_of_token_being_sent_pk = ctx.accounts.mint.key().clone();
    //     let application_idx_bytes = application_idx.to_le_bytes();
    //     let inner = vec![
    //         b"state".as_ref(),
    //         ctx.accounts.user.key.as_ref(),
    //         ctx.accounts.user_receiving.key.as_ref(),
    //         mint_of_token_being_sent_pk.as_ref(),
    //         application_idx_bytes.as_ref(),
    //         bump_vector.as_ref(),
    //     ];
    //     let outer = vec![inner.as_slice()];

    //     let transfer_instruction = Transfer {
    //         from: ctx.accounts.escrow_wallet.to_account_info(),
    //         to: ctx.accounts.refund_wallet.to_account_info(),
    //         authority: ctx.accounts.state.to_account_info(),
    //     };
    //     let cpi_ctx = CpiContext::new_with_signer(
    //         ctx.accounts.token_program.to_account_info(),
    //         transfer_instruction,
    //         outer.as_slice(),
    //     );
    //     anchor_spl::token::transfer(cpi_ctx, ctx.accounts.escrow_wallet_state.amount)?;

    //     let should_close = {
    //         ctx.accounts.escrow_wallet_state.reload()?;
    //         ctx.accounts.escrow_wallet_state.amount == 0
    //     };

    //     if should_close {
    //         let ca = CloseAccount {
    //             account: ctx.accounts.escrow_wallet_state.to_account_info(),
    //             destination: ctx.accounts.user.to_account_info(),
    //             authority: ctx.accounts.application_state.to_account_info(),
    //         };
    //         let cpi_ctx = CpiContext::new_with_signer(
    //             ctx.accounts.token_program.to_account_info(),
    //             ca,
    //             outer.as_slice(),
    //         );
    //         anchor_spl::token::close_account(cpi_ctx)?;
    //     }

    //     Ok(())
    // }
}

#[account]
#[derive(Default)]
pub struct State {
    user: Pubkey,
    mint: Pubkey,
    escrow_wallet: Pubkey,
    amount: u64,
    stage: u8,
}

#[derive(Accounts)]
#[instruction(application_idx: u64, state_bump: u8, wallet_bump: u8)]
pub struct DepositInstruction<'info> {
    #[account(
        init,
        payer=user,
        seeds=[b"state".as_ref(), user.key().as_ref(), user_receiving.key.as_ref(), mint.key().as_ref(), application_idx.to_le_bytes().as_ref()],
        state_bump
    )]
    application_state: Account<'info, State>,
    #[account(
        init,
        payer = user,
        seeds=[b"wallet".as_ref(), user_sending.key().as_ref(), user_receiving.key.as_ref(), mint.key().as_ref(), application_idx.to_le_bytes().as_ref()],
        wallet_bump,
        token::mint=mint,
        token::authority=application_state,
    )]
    escrow_wallet_state: Account<'info, TokenAccount>,

    #[account(mut)]
    user: Signer<'info>,
    user_receiving: AccountInfo<'info>,
    mint: Account<'info, Mint>,

    #[account(
        mut,
        constraint=wallet_to_withdraw.owner == user.key(),
        constraint=wallet_to_withdraw.mint == mint.key()
    )]
    wallet_to_withdraw: Account<'info, TokenAccount>,

    system_program: Program<'info, System>,
    token_program: Program<'info, Token>,
}

#[derive(Accounts)]
#[instruction(application_idx: u64, state_bump: u8, wallet_bump: u8)]
pub struct WithDrawInstruction<'info> {
    #[account(
        mut,
        seeds=[b"state".as_ref(), user.key().as_ref(), user_receiving.key.as_ref(), mint.key().as_ref(), application_idx.to_le_bytes().as_ref()],
        bump = state_bump,
        has_one = user,
        has_one = mint,
    )]
    application_state: Account<'info, State>,
    #[account(
        mut,
        seeds=[b"wallet".as_ref(), user.key().as_ref(), user_receiving.key.as_ref(), mint.key().as_ref(), application_idx.to_le_bytes().as_ref()],
        bump = wallet_bump,
    )]
    escrow_wallet_state: Account<'info, TokenAccount>,
    #[account(mut)]
    user: Signer<'info>,
    user_receiving: AccountInfo<'info>,
    mint: Account<'info, Mint>,

    system_program: Program<'info, System>,
    token_program: Program<'info, Token>,
    rent: Sysvar<'info, Rent>,
    #[account(
        mut,
        constraint=refund_wallet.owner == user.key(),
        constraint=refund_wallet.mint == mint.key()
    )]
    refund_wallet: Account<'info, TokenAccount>,
}

// define stage (current support: De)
#[derive(Clone, Copy, PartialEq)]
pub enum Stage {
    Deposit,
    WithDraw,
}

impl Stage {
    fn from(code: u8) -> Result<Stage> {
        match code {
            1 => Ok(Stage::Deposit),
            2 => Ok(Stage::WithDraw),
            unknown_code => {
                msg!("Unknow state: {}", unknown_code);
                Err(ErrorCode::InvalidStage.into())
            }
        }
    }
    fn to_code(&self) -> u8 {
        match self {
            Stage::Deposit => 1,
            Stage::WithDraw => 2,
        }
    }
}
