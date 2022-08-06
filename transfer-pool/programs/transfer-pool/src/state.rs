use crate::error::ErrorCode;
use anchor_lang::prelude::*;

// define stage (current support: Deposit & Withdraw)
#[derive(Clone, Copy, PartialEq)]
pub enum Stage {
  Deposit,
  WithDraw,
}

impl Stage {
  pub fn from(code: u8) -> Result<Stage> {
    match code {
      1 => Ok(Stage::Deposit),
      2 => Ok(Stage::WithDraw),
      unknown_code => {
        msg!("Unknow state: {}", unknown_code);
        Err(ErrorCode::InvalidStage.into())
      }
    }
  }
  pub fn to_code(&self) -> u8 {
    match self {
      Stage::Deposit => 1,
      Stage::WithDraw => 2,
    }
  }
}
