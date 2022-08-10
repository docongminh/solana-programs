use anchor_lang::prelude::*;

#[error_code]
pub enum ErrorCode {
  #[msg("Invalid stage")]
  InvalidStage,
  #[msg("insufficient funds")]
  InsufficientFunds,
  #[msg("invalid nft")]
  InvalidNFT,
  #[msg("have not yet nft selling now")]
  NotSelling,
  InvalidSeller,
  InvalidMint
}
