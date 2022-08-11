pub mod error;
pub mod state;
pub mod processor;

use anchor_lang::prelude::*;
use anchor_spl::token::TokenAccount;
use anchor_spl::token::{Mint, Token};

use crate::error::ErrorCode;
use crate::state::{Stage, StateAccount};
use crate::processor::{transfer_sol, transfer_token};

declare_id!("4kVr2h7SZkWVV7DBoYEkL4gV7bzcp2T9TbuFqmNBcnUc");

const ESCROW_PDA_SEED: &[u8] = b"escrow_state";
const ESCROW_NFT_ASSOCIATE_PDA_SEED: &[u8] = b"escrow_nft_associate";

#[program]
pub mod trade_nft {
    use super::*;
    
    pub fn create_trade_order(ctx: Context<Create>) -> Result<()>{
        let state = &mut ctx.accounts.state_account;
        state.seller = ctx.accounts.seller.key();
        state.escrow_associate_nft_wallet = ctx.accounts.escrow_associate_nft_wallet.key();
        state.mint_nft = ctx.accounts.mint_nft.key();
        state.mint_token = ctx.accounts.mint_token.key();
        state.bumps.state_bump = *ctx.bumps.get("state_account").unwrap();
        state.bumps.wallet_nft_bump = *ctx.bumps.get("escrow_associate_nft_wallet").unwrap();
        msg!("Trade order created !");
        Ok(())
    }

    pub fn edit_price(ctx: Context<EditPriceInstruction>, new_price_sol: u64, new_price_token: u64) -> Result<()> {
        let state_account = &mut ctx.accounts.state_account;
        state_account.price_sol = new_price_sol;
        state_account.price_token = new_price_token;
        Ok(())
    }

    pub fn sell(ctx: Context<SellInstruction>, price_sol: u64, price_token: u64, amount: u64) -> Result<()>{
        let state = &mut ctx.accounts.state_account;
        state.price_sol = price_sol;
        state.price_token = price_token;
        state.amount = amount;
        transfer_token(
            ctx.accounts.seller_associate_nft_account.to_account_info(),
            ctx.accounts.escrow_associate_nft_wallet.to_account_info(),
            ctx.accounts.seller.to_account_info(),
            amount,
            ctx.accounts.token_program.to_account_info(),
            None
        )?;
        state.stage = Stage::Sell.to_code();
        Ok(())
    }

    pub fn buy(ctx: Context<BuyInstruction>, paid_native: bool, amount: u64, price_sol: u64, price_token: u64) -> Result<()>{
        let current_stage = Stage::from(ctx.accounts.state_account.stage)?;
        let is_valid_stage = current_stage == Stage::Sell;
        if !is_valid_stage {
            return Err(ErrorCode::InvalidStage.into());
        }
        let state_account = &mut ctx.accounts.state_account;
        let user_seed = state_account.seller.key();
        let nft_mint_seed = ctx.accounts.mint_nft.key();
        let seeds = &[&[ESCROW_PDA_SEED, user_seed.as_ref(), nft_mint_seed.as_ref(), bytemuck::bytes_of(&state_account.bumps.state_bump)][..]];
        if paid_native {
            let total_sol_price = amount * price_sol;
            // transfer sol from buyer -> seller
            transfer_sol(
                ctx.accounts.buyer.to_account_info(),
                ctx.accounts.seller.to_account_info(),
                total_sol_price,
                ctx.accounts.system_program.to_account_info(),
                None,
            )?;
        } else {
            let total_token_price = amount * price_token;
            // transfer token from buyer -> seller
            transfer_token(
                ctx.accounts.buyer_associate_token_account.to_account_info(),
                ctx.accounts.seller_associate_token_account.to_account_info(),
                ctx.accounts.buyer.to_account_info(),
                total_token_price,
                ctx.accounts.token_program.to_account_info(),
                None
            )?;
        }
        // transfer nft from escrow -> buyer
        transfer_token(
            ctx.accounts.escrow_associate_nft_wallet.to_account_info(),
            ctx.accounts.buyer_associate_nft_account.to_account_info(),
            state_account.to_account_info(),
            amount,
            ctx.accounts.token_program.to_account_info(),
            Some(seeds)
        )?;
        state_account.amount -= amount;
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
            ctx.accounts.escrow_associate_nft_wallet.to_account_info(), 
            ctx.accounts.seller_associate_nft_account.to_account_info(), 
            state.to_account_info(),
            amount,
            ctx.accounts.token_program.to_account_info(),
            Some(seeds)
        )?;
        state.amount = 0;
        state.price_sol = 0;
        state.price_token = 0;
        Ok(())
    }   
}

#[derive(Accounts)]
pub struct Create<'info> {
    // PDA account
    #[account(
        init,
        payer = seller,
        space = StateAccount::LEN,
        seeds=[ESCROW_PDA_SEED, seller.key().as_ref(), mint_nft.key().as_ref()],
        bump,    
    )]
    state_account: Account<'info, StateAccount>,
    // escrow associate account for nft
    #[account(
        init,
        payer=seller,
        seeds=[ESCROW_NFT_ASSOCIATE_PDA_SEED, seller.key().as_ref(), mint_nft.key().as_ref()],
        bump,
        token::mint=mint_nft,
        token::authority=state_account,
    )]
    escrow_associate_nft_wallet: Account<'info, TokenAccount>,
    // mint nft sell
    #[account(constraint = mint_nft.decimals == 0 @ ErrorCode::InvalidNFT)]
    mint_nft: Account<'info, Mint>,
    mint_token: Account<'info, Mint>,
    // seller associate nft account
    #[account(
        mut,
        token::mint=mint_nft,
        token::authority=seller,
        constraint = seller_associate_nft_account.amount > 0 @ ErrorCode::InsufficientFunds
    )]
    seller_associate_nft_account: Account<'info, TokenAccount>,
    #[account(mut, constraint = seller.lamports() > 0 && seller.data_is_empty())]
    seller: Signer<'info>,
    system_program: Program<'info, System>,
    token_program: Program<'info, Token>,
    rent: Sysvar<'info, Rent>,
}
#[derive(Accounts)]
#[instruction(amount: u64)]
pub struct SellInstruction<'info> {
    // PDA account
    #[account(mut,
        seeds = [ESCROW_PDA_SEED, seller.key().as_ref(), mint_nft.key().as_ref()],
        bump = state_account.bumps.state_bump,
        has_one=seller,
        has_one=mint_nft
    )]
    state_account: Account<'info, StateAccount>,
    #[account(mut,
        seeds = [ESCROW_NFT_ASSOCIATE_PDA_SEED, seller.key().as_ref(), mint_nft.key().as_ref()],
        bump = state_account.bumps.wallet_nft_bump
    )]
    escrow_associate_nft_wallet: Account<'info, TokenAccount>,
    // mint nft sell
    #[account(constraint = mint_nft.decimals == 0 @ ErrorCode::InvalidNFT)]
    mint_nft: Account<'info, Mint>,
    mint_token: Account<'info, Mint>,
    #[account(
        mut,
        token::mint=mint_nft,
        token::authority=seller,
        constraint = seller_associate_nft_account.amount > 0 @ ErrorCode::InsufficientFunds
    )]
    seller_associate_nft_account: Account<'info, TokenAccount>,
    #[account(mut, constraint = seller.lamports() > 0 && seller.data_is_empty())]
    seller: Signer<'info>,
    system_program: Program<'info, System>,
    token_program: Program<'info, Token>,
}


#[derive(Accounts)]
#[instruction(new_price_sol: u64, new_price_token: u64)]
pub struct EditPriceInstruction<'info> {
    #[account(mut,
        has_one=seller @ ErrorCode::InvalidSeller,
        constraint = state_account.amount > 0 @ ErrorCode::NotSelling
    )]
    state_account: Account<'info, StateAccount>,
    seller: Signer<'info>,
    // system
    system_program: Program<'info, System>,
    token_program: Program<'info, Token>,
}
#[derive(Accounts)]
#[instruction(amount: u64, price_sol: u64, price_token: u64)]
pub struct BuyInstruction<'info> {
    /// CHECK
    #[account(mut)]
    seller: AccountInfo<'info>,
    #[account(mut,
        seeds = [ESCROW_PDA_SEED, seller.key().as_ref(), mint_nft.key().as_ref()],
        bump = state_account.bumps.state_bump,
        has_one=seller,
        has_one=mint_nft
    )]
    state_account: Box<Account<'info, StateAccount>>,
    #[account(mut,
        seeds = [ESCROW_NFT_ASSOCIATE_PDA_SEED, seller.key().as_ref(), mint_nft.key().as_ref()],
        bump = state_account.bumps.wallet_nft_bump,
        token::mint = mint_nft,
        token::authority = state_account
    )]
    escrow_associate_nft_wallet: Box<Account<'info, TokenAccount>>,
    mint_nft: Account<'info, Mint>,
    mint_token: Account<'info, Mint>,
    #[account(
        mut,
        token::mint = mint_nft,
        token::authority = buyer
    )]
    buyer_associate_nft_account: Box<Account<'info, TokenAccount>>,
    #[account(
        mut,
        token::mint = mint_token,
        token::authority = buyer
    )]
    buyer_associate_token_account: Box<Account<'info, TokenAccount>>,
    #[account(
        mut,
        token::mint = mint_token,
        token::authority = seller
    )]
    seller_associate_token_account: Box<Account<'info, TokenAccount>>,
    #[account(mut, constraint = buyer.lamports() > amount * price_sol @ ErrorCode::InsufficientFunds)]
    buyer: Signer<'info>,
    // // system
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
        seeds = [ESCROW_NFT_ASSOCIATE_PDA_SEED, seller.key().as_ref(), mint_nft.key().as_ref()],
        bump = state_account.bumps.wallet_nft_bump,
        token::mint = mint_nft,
        token::authority = state_account
    )]
    escrow_associate_nft_wallet: Account<'info, TokenAccount>,
    #[account(constraint = mint_nft.decimals == 0 @ ErrorCode::InvalidNFT)]
    mint_nft: Account<'info, Mint>,
    // refund wallet
    #[account(
        mut,
        token::mint = mint_nft,
        token::authority = seller
    )]
    seller_associate_nft_account: Account<'info, TokenAccount>,
    // system
    system_program: Program<'info, System>,
    token_program: Program<'info, Token>,
}

