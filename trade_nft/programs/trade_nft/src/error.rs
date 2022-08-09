use anchor_lang::prelude::*;

#[error_code]
pub enum ErrorCode {
  #[msg("Invalid state storage")]
  InvalidStage,
  #[msg("insufficient funds")]
  InsufficientFunds,
}
