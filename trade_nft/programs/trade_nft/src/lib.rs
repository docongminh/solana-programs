pub mod error;
pub mod state;
pub mod processor;

use anchor_lang::prelude::*;
use anchor_spl::token::TokenAccount;
use anchor_spl::token::{Mint, Token};

use crate::error::ErrorCode;
use crate::state::{Stage, State};
use crate::processor::{transfer_sol, transfer_token, to_close_account};

declare_id!("8JDESvt2JBggQMc5qDCuc3mDCqMf6EzJq8iEwRTcQJdx");

#[program]
pub mod trade_nft {
    use super::*;

    pub fn sell(ctx: Context<SellInstruction>, price: u64) -> Result<()>{
        let state = &mut ctx.accounts.state_account;
        state.seller = ctx.accounts.seller.key().clone();
        state.escrow_associate_wallet = ctx.accounts.escrow_wallet_associate_account.key().clone();
        state.price = price;
        state.mint_nft = ctx.accounts.nft_mint.key().clone();
        let bump = *ctx.bumps.get("state_account").unwrap();
        state.bumps.state_bump = bump;
        
        // inner
        let bump_vector = state.bumps.state_bump.to_le_bytes();
        let seller = ctx.accounts.seller.key().clone();
        let mint = ctx.accounts.nft_mint.key().clone();
        let inner = vec![
            b"state".as_ref(),
            seller.as_ref(),
            mint.as_ref(),
            bump_vector.as_ref()
        ];

        let outer = vec![inner.as_slice()];

        transfer_token(
            ctx.accounts.seller_associated_account.to_account_info(),
            ctx.accounts.escrow_wallet_associate_account.to_account_info(),
            1,
            ctx.accounts.seller.to_account_info(),
            outer.clone(),
            ctx.accounts.token_program.to_account_info()
        )?;
        state.stage = Stage::Sell.to_code();
        Ok(())
    }

    pub fn buy(ctx: Context<BuyInstruction>, price: u64) -> Result<()>{
        let current_stage = Stage::from(ctx.accounts.state_account.stage)?;
        let is_valid_stage = current_stage == Stage::Sell;
        if !is_valid_stage {
            return Err(ErrorCode::InvalidStage.into());
        }
        let state = &mut ctx.accounts.state_account;
        let bump_vector = state.bumps.state_bump.to_le_bytes();
        let nft_mint = ctx.accounts.nft_mint.key().clone();
        let inner = vec![
            b"state".as_ref(),
            ctx.accounts.seller.key.as_ref(),
            nft_mint.as_ref(),
            bump_vector.as_ref(),
        ];
        let outer = vec![inner.as_slice()];
        // transfer sol from buyer -> escrow
        transfer_sol(
            ctx.accounts.buyer.to_account_info(),
            state.to_account_info(),
            price,
            outer.clone(),
            ctx.accounts.system_program.to_account_info()
        )?;
        // transfer nft from escrow -> buyer
        transfer_token(
            ctx.accounts.escrow_wallet_associate_account.to_account_info(),
            ctx.accounts.buyer_associated_account.to_account_info(),
            1,
            state.to_account_info(),
            outer.clone(),
            ctx.accounts.token_program.to_account_info()
        )?;
        // transfer sol from escrow -> seller
        transfer_sol(
            state.to_account_info(),
            ctx.accounts.seller.to_account_info(),
            price,
            outer.clone(),
            ctx.accounts.system_program.to_account_info()
        )?;

        // close account
        to_close_account(
            ctx.accounts.escrow_wallet_associate_account.to_account_info(),
            ctx.accounts.seller.to_account_info(),
            ctx.accounts.state_account.to_account_info(),
            outer,
            ctx.accounts.token_program.to_account_info()
        )?;
        Ok(())
    }

    pub fn cancel(ctx: Context<CancelSellInstruction>) -> Result<()> {
        let current_stage = Stage::from(ctx.accounts.state_account.stage)?;
        let is_valid_stage = current_stage == Stage::Sell;
        if !is_valid_stage {
            return Err(ErrorCode::InvalidStage.into());
        }
        let bump_vector = ctx.accounts.state_account.bumps.state_bump.to_le_bytes();
        let mint_token = ctx.accounts.nft_mint.key().clone();
        let inner = vec![
            b"state".as_ref(),
            ctx.accounts.seller.key.as_ref(),
            mint_token.as_ref(),
            bump_vector.as_ref(),
        ];
        let outer = vec![inner.as_slice()];

        // withdraw back nft to seller wallet
        transfer_token(
            ctx.accounts.escrow_wallet_associate_account.to_account_info(), 
            ctx.accounts.seller_associated_account.to_account_info(),
            1, 
            ctx.accounts.state_account.to_account_info(),
            outer.clone(),
            ctx.accounts.token_program.to_account_info()
        )?;

        // close all accounts (state escrow account & state escrow associated account)
        // close escrow associate account
        to_close_account(
            ctx.accounts.escrow_wallet_associate_account.to_account_info(),
            ctx.accounts.seller.to_account_info(),
            ctx.accounts.state_account.to_account_info(),
            outer,
            ctx.accounts.token_program.to_account_info()
        )?;
        // close escrow state account
        // to_close_account(
        //     ctx.accounts.state_account.to_account_info(),
        //     ctx.accounts.seller.to_account_info(),
        //     ctx.accounts.state_account.to_account_info(),
        //     outer,
        //     ctx.accounts.token_program.
        // )
        Ok(())
    }   
}

#[derive(Accounts)]
pub struct SellInstruction<'info> {
    #[account(mut)]
    seller: Signer<'info>,
    // PDA account
    #[account(
        init,
        payer = seller,
        space = 115,
        seeds=[b"state", seller.key().as_ref(), nft_mint.key().as_ref()],
        bump,    
    )]
    state_account: Account<'info, State>,
    #[account(
        init,
        payer=seller,
        seeds=[b"wallet", seller.key().as_ref(), nft_mint.key().as_ref()],
        bump,
        token::mint=nft_mint,
        token::authority=state_account,
    )]
    escrow_wallet_associate_account: Account<'info, TokenAccount>,
    // mint nft sell
    nft_mint: Account<'info, Mint>,

    #[account(
        mut,
        token::mint=nft_mint,
        token::authority=seller
    )]
    seller_associated_account: Account<'info, TokenAccount>,


    system_program: Program<'info, System>,
    token_program: Program<'info, Token>,
    rent: Sysvar<'info, Rent>,
}

#[derive(Accounts)]
#[instruction(price: u64)]
pub struct BuyInstruction<'info> {
    #[account(mut, constraint = buyer.lamports() > price)]
    buyer: Signer<'info>,
    /// CHECK
    seller: AccountInfo<'info>,
    #[account(mut)]
    state_account: Account<'info, State>,
    nft_mint: Account<'info, Mint>,
    #[account(
        mut,
        token::mint = nft_mint,
        token::authority = buyer
    )]
    buyer_associated_account: Account<'info, TokenAccount>,
    #[account(mut)]
    escrow_wallet_associate_account: Account<'info, TokenAccount>,
    // system
    system_program: Program<'info, System>,
    token_program: Program<'info, Token>,
}

#[derive(Accounts)]
pub struct CancelSellInstruction<'info> {
    #[account(mut)]
    seller: Signer<'info>,
    #[account(mut)]
    state_account: Account<'info, State>,
    #[account(mut)]
    escrow_wallet_associate_account: Account<'info, TokenAccount>,
    nft_mint: Account<'info, Mint>,
    // refund wallet
    #[account(
        mut,
        token::mint = nft_mint,
        token::authority = seller
    )]
    seller_associated_account: Account<'info, TokenAccount>,
    // system
    system_program: Program<'info, System>,
    token_program: Program<'info, Token>,
}

