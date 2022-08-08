use anchor_lang::prelude::*;

#[error_code]
pub enum ErrorCode {
  #[msg("Wallet to withdraw from is not owned by owner")]
  WalletToWithdrawFromInvalid,
  #[msg("Invalid state storage")]
  InvalidStage,
}
