use crate::error::ErrorCode;
use anchor_lang::prelude::*;

#[derive(AnchorSerialize, AnchorDeserialize, Clone)]
pub enum Trade {
  TokenNft {
    nft_mint: Pubkey,
    buyer: Pubkey,
    seller: Pubkey,
    token_mint: Pubkey,
  },
  SolNft {
    nft_mint: Pubkey,
    buyer: Pubkey,
    seller: Pubkey,
  },
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone, Default)]
pub struct Bumps {
  pub state_bump: u8,
  pub wallet_nft_bump: u8,
  pub wallet_token_bump: u8,
}

#[account]
pub struct StateAccount {
  pub seller: Pubkey,
  pub mint_nft: Pubkey,
  // Token use to buy NFT
  pub mint_token: Pubkey,
  pub escrow_associate_nft_wallet: Pubkey,
  pub amount: u64,
  // Price to buy NFT in SOL
  pub price_sol: u64,
  // Price to buy NFT in specify Token
  pub price_token: u64,
  pub timestamp: u64,
  pub stage: u8,
  pub bumps: Bumps,
}

impl StateAccount {
  pub const LEN: usize = 8 // internal discriminator
    + 4 * 32 // PubKey
    + 8 * 4 // u64
    + 4 * 1; // u8
}

// define stage (current support: Sell, Buy & CancelSell)
// TODO: Bid, cancelBid, Offer, AcceptOffer
#[derive(Clone, Copy, PartialEq)]
pub enum Stage {
  Sell,
  Buy,
  CancelSell,
}

impl Stage {
  pub fn from(code: u8) -> Result<Stage> {
    match code {
      1 => Ok(Stage::Sell),
      2 => Ok(Stage::Buy),
      3 => Ok(Stage::CancelSell),
      unknown_code => {
        msg!("Unknow state: {}", unknown_code);
        Err(ErrorCode::InvalidStage.into())
      }
    }
  }
  pub fn to_code(&self) -> u8 {
    match self {
      Stage::Sell => 1,
      Stage::Buy => 2,
      Stage::CancelSell => 3,
    }
  }
}
