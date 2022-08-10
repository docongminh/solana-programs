pub mod error;
pub mod state;
pub mod processor;

use anchor_lang::prelude::*;
use anchor_spl::token::TokenAccount;
use anchor_spl::token::{Mint, Token};

use crate::error::ErrorCode;
use crate::state::{Stage, StateAccount};
use crate::processor::{transfer_sol, transfer_token};

declare_id!("3nDusEFjqioKWzhP6uXxqMSwk2rNMjB2Th8drLAGthh1");
const ESCROW_PDA_SEED: &[u8] = b"state";
const ESCROW_ASSOCIATE_PDA_SEED: &[u8] = b"wallet";
#[program]
pub mod trade_nft {
    use super::*;
    
    pub fn create_trade_order(ctx: Context<Create>) -> Result<()>{
        let state = &mut ctx.accounts.state_account;
        state.seller = ctx.accounts.seller.key();
        state.escrow_associate_wallet = ctx.accounts.escrow_associate_wallet.key();
        state.mint_nft = ctx.accounts.mint_nft.key();
        state.bumps.state_bump = *ctx.bumps.get("state_account").unwrap();
        state.bumps.wallet_bump = *ctx.bumps.get("escrow_associate_wallet").unwrap();
        msg!("state: {:?}", state.bumps.state_bump);
        msg!("state: {:?}", state.bumps.wallet_bump);
        Ok(())
    }
    pub fn sell(ctx: Context<SellInstruction>, price: u64, amount: u64) -> Result<()>{
        let state = &mut ctx.accounts.state_account;
        state.price = price;
        state.amount = amount;
        transfer_token(
            ctx.accounts.seller_associated_account.to_account_info(),
            ctx.accounts.escrow_associate_wallet.to_account_info(),
            ctx.accounts.seller.to_account_info(),
            amount,
            ctx.accounts.token_program.to_account_info(),
            None
        )?;
        state.stage = Stage::Sell.to_code();
        Ok(())
    }

    pub fn buy(ctx: Context<BuyInstruction>, price: u64, amount: u64) -> Result<()>{
        let current_stage = Stage::from(ctx.accounts.state_account.stage)?;
        let is_valid_stage = current_stage == Stage::Sell;
        if !is_valid_stage {
            return Err(ErrorCode::InvalidStage.into());
        }
        let state_account = &mut ctx.accounts.state_account;
        let user_seed = state_account.seller.key();
        let nft_mint_seed = ctx.accounts.mint_nft.key();
        let seeds = &[&[ESCROW_PDA_SEED, user_seed.as_ref(), nft_mint_seed.as_ref(), bytemuck::bytes_of(&state_account.bumps.state_bump)][..]];
        // transfer sol from buyer -> seller
        transfer_sol(
            ctx.accounts.buyer.to_account_info(),
            ctx.accounts.seller.to_account_info(),
            price,
            ctx.accounts.system_program.to_account_info(),
            None,
        )?;
        // // transfer nft from escrow -> buyer
        transfer_token(
            ctx.accounts.escrow_associate_wallet.to_account_info(),
            ctx.accounts.buyer_associated_account.to_account_info(),
            state_account.to_account_info(),
            amount,
            ctx.accounts.token_program.to_account_info(),
            Some(seeds)
        )?;
        Ok(())
    }

    pub fn cancel(ctx: Context<CancelSellInstruction>) -> Result<()> {
        let current_stage = Stage::from(ctx.accounts.state_account.stage)?;
        let is_valid_stage = current_stage == Stage::Sell;
        if !is_valid_stage {
            return Err(ErrorCode::InvalidStage.into());
        }
        let state = &mut ctx.accounts.state_account;
        let amount = state.amount;
        let user_seed = state.seller.key();
        let nft_mint_seed = ctx.accounts.mint_nft.key();
        let seeds = &[&[ESCROW_PDA_SEED, user_seed.as_ref(), nft_mint_seed.as_ref(), bytemuck::bytes_of(&state.bumps.state_bump)][..]];

        // withdraw back nft to seller wallet
        transfer_token(
            ctx.accounts.escrow_associate_wallet.to_account_info(), 
            ctx.accounts.seller_associated_account.to_account_info(), 
            state.to_account_info(),
            amount,
            ctx.accounts.token_program.to_account_info(),
            Some(seeds)
        )?;
        state.amount = 0;
        state.price = 0;
        Ok(())
    }   
}


#[derive(Accounts)]
pub struct Create<'info> {
    #[account(mut, constraint = seller.lamports() > 0 && seller.data_is_empty())]
    seller: Signer<'info>,
    // PDA account
    #[account(
        init,
        payer = seller,
        space = StateAccount::LEN,
        seeds=[ESCROW_PDA_SEED, seller.key().as_ref(), mint_nft.key().as_ref()],
        bump,    
    )]
    state_account: Account<'info, StateAccount>,
    #[account(
        init,
        payer=seller,
        seeds=[ESCROW_ASSOCIATE_PDA_SEED, seller.key().as_ref(), mint_nft.key().as_ref()],
        bump,
        token::mint=mint_nft,
        token::authority=state_account,
    )]
    escrow_associate_wallet: Account<'info, TokenAccount>,
    // mint nft sell
    #[account(constraint = mint_nft.decimals == 0 @ ErrorCode::InvalidNFT)]
    mint_nft: Account<'info, Mint>,

    #[account(
        mut,
        token::mint=mint_nft,
        token::authority=seller,
        constraint = seller_associated_account.amount > 0 @ ErrorCode::InsufficientFunds
    )]
    seller_associated_account: Account<'info, TokenAccount>,
    system_program: Program<'info, System>,
    token_program: Program<'info, Token>,
    rent: Sysvar<'info, Rent>,
}
#[derive(Accounts)]
#[instruction(price: u64, amount: u64)]
pub struct SellInstruction<'info> {
    #[account(mut, constraint = seller.lamports() > 0 && seller.data_is_empty())]
    seller: Signer<'info>,
    // PDA account
    #[account(mut,
        seeds = [ESCROW_PDA_SEED, seller.key().as_ref(), mint_nft.key().as_ref()],
        bump = state_account.bumps.state_bump,
        has_one=seller,
        has_one=mint_nft
    )]
    state_account: Account<'info, StateAccount>,
    #[account(mut,
        seeds = [ESCROW_ASSOCIATE_PDA_SEED, seller.key().as_ref(), mint_nft.key().as_ref()],
        bump = state_account.bumps.wallet_bump
    )]
    escrow_associate_wallet: Account<'info, TokenAccount>,
    // mint nft sell
    #[account(constraint = mint_nft.decimals == 0 @ ErrorCode::InvalidNFT)]
    mint_nft: Account<'info, Mint>,

    #[account(
        mut,
        token::mint=mint_nft,
        token::authority=seller,
        constraint = seller_associated_account.amount > 0 @ErrorCode::InsufficientFunds
    )]
    seller_associated_account: Account<'info, TokenAccount>,
    system_program: Program<'info, System>,
    token_program: Program<'info, Token>,
}

#[derive(Accounts)]
#[instruction(price: u64)]
pub struct BuyInstruction<'info> {
    #[account(mut, constraint = buyer.lamports() > price @ ErrorCode::InsufficientFunds)]
    buyer: Signer<'info>,
    /// CHECK
    #[account(mut)]
    seller: AccountInfo<'info>,
    #[account(mut,
        seeds = [ESCROW_PDA_SEED, seller.key().as_ref(), mint_nft.key().as_ref()],
        bump = state_account.bumps.state_bump,
        has_one=seller,
        has_one=mint_nft
    )]
    state_account: Account<'info, StateAccount>,
    mint_nft: Account<'info, Mint>,
    #[account(
        mut,
        token::mint = mint_nft,
        token::authority = buyer
    )]
    buyer_associated_account: Account<'info, TokenAccount>,
    #[account(mut,
        seeds = [ESCROW_ASSOCIATE_PDA_SEED, seller.key().as_ref(), mint_nft.key().as_ref()],
        bump = state_account.bumps.wallet_bump,
        token::mint = mint_nft,
        token::authority = state_account
    )]
    escrow_associate_wallet: Account<'info, TokenAccount>,
    // system
    system_program: Program<'info, System>,
    token_program: Program<'info, Token>,
}

#[derive(Accounts)]
pub struct CancelSellInstruction<'info> {
    #[account(mut)]
    seller: Signer<'info>,
    #[account(mut,
        has_one=seller @ ErrorCode::InvalidSeller,
        has_one=mint_nft @ ErrorCode::InvalidMint,
        constraint = state_account.amount > 0 @ ErrorCode::NotSelling
    )]
    state_account: Account<'info, StateAccount>,
    #[account(mut,
        seeds = [ESCROW_ASSOCIATE_PDA_SEED, seller.key().as_ref(), mint_nft.key().as_ref()],
        bump = state_account.bumps.wallet_bump,
        token::mint = mint_nft,
        token::authority = state_account
    )]
    escrow_associate_wallet: Account<'info, TokenAccount>,
    #[account(constraint = mint_nft.decimals == 0 @ ErrorCode::InvalidNFT)]
    mint_nft: Account<'info, Mint>,
    // refund wallet
    #[account(
        mut,
        token::mint = mint_nft,
        token::authority = seller
    )]
    seller_associated_account: Account<'info, TokenAccount>,
    // system
    system_program: Program<'info, System>,
    token_program: Program<'info, Token>,
}

