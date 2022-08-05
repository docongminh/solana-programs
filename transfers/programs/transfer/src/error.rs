use anchor_lang::prelude::*;

#[error_code]
pub enum ErrorDefine {
  #[msg("This mint address not supported now")]
  MintUnSupport,

  #[msg("Unauthorized !!!")]
  Unauthorized,
}

impl From<ErrorDefine> for ProgramError {
  fn from(e: ErrorDefine) -> Self {
    return ProgramError::Custom(e as u32);
  }
}
