// use anchor_lang::solana_program::program_error::ProgramError;
// use thiserror::Error;
use anchor_lang::prelude::*;

#[error_code]
pub enum ErrorDefine {
  #[msg("Minimul total is -5")]
  SubError,

  #[msg("Maximum total is 10")]
  AddError,

  #[msg("Unauthorized !!!")]
  Unauthorized,
}

impl From<ErrorDefine> for ProgramError {
  fn from(e: ErrorDefine) -> Self {
    return ProgramError::Custom(e as u32);
  }
}
