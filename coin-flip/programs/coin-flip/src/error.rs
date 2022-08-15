use anchor_lang::prelude::*;

#[error_code]
pub enum ErrorCode {
  #[msg("Invalid stage")]
  InvalidStage,
  #[msg("insufficient funds")]
  InsufficientFunds,
  #[msg("invalid nft")]
  InvalidToken,
  InvalidRandomValue,
}
