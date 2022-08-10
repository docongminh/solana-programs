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
  pub wallet_bump: u8,
}

#[account]
pub struct State {
  pub seller: Pubkey,
  pub mint_nft: Pubkey,
  pub escrow_associate_wallet: Pubkey,
  pub amount: u64,
  pub price: u64,
  pub stage: u8,
  pub bumps: Bumps,
  pub timestamp: u64,
}
impl State {
  pub const LEN: usize = 8 + 3 * 32 + 8 * 3 + 3 * 1;
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
