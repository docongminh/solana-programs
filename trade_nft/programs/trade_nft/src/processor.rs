use anchor_lang::prelude::*;
use anchor_spl::token::{CloseAccount, Mint, Token, Transfer};

pub fn transfer_sol<'info>(
  sender: AccountInfo<'info>,
  receiver: AccountInfo<'info>,
  amount: u64,
  outer: Vec<&[&[u8]]>,
  system_program: AccountInfo<'info>,
) -> Result<()> {
  let transfer_sol_instruction = anchor_lang::system_program::Transfer {
    from: sender.to_account_info(),
    to: receiver.to_account_info(),
  };
  let cpi_ctx_sol = CpiContext::new_with_signer(
    system_program.to_account_info(),
    transfer_sol_instruction,
    outer.as_slice(),
  );
  anchor_lang::system_program::transfer(cpi_ctx_sol, amount)?;

  return Ok(());
}

// transfer fungible token & nft token
pub fn transfer_token<'info>(
  sender: AccountInfo<'info>,
  receiver: AccountInfo<'info>,
  amount: u64,
  user: AccountInfo<'info>,
  outer: Vec<&[&[u8]]>,
  token_program: AccountInfo<'info>,
) -> Result<()> {
  let transfer_instruction = Transfer {
    from: sender.to_account_info(),
    to: receiver.to_account_info(),
    authority: user.to_account_info(),
  };
  let cpi_ctx = CpiContext::new_with_signer(
    token_program.to_account_info(),
    transfer_instruction,
    outer.as_slice(),
  );

  anchor_spl::token::transfer(cpi_ctx, amount)?;
  Ok(())
}

pub fn to_close_account<'info>(
  account_to_close: AccountInfo<'info>,
  user: AccountInfo<'info>,
  state_account: AccountInfo<'info>,
  outer: Vec<&[&[u8]]>,
  token_program: AccountInfo<'info>,
) -> Result<()> {
  let close_account = CloseAccount {
    account: account_to_close.to_account_info(),
    destination: user.to_account_info(),
    authority: state_account.to_account_info(),
  };
  let cpi_ctx = CpiContext::new_with_signer(
    token_program.to_account_info(),
    close_account,
    outer.as_slice(),
  );
  anchor_spl::token::close_account(cpi_ctx)?;
  Ok(())
}
