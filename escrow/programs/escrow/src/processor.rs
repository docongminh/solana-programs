use anchor_lang::prelude::*;
use anchor_spl::token::{CloseAccount, Transfer};

pub fn transfer_sol<'info>(
  sender: AccountInfo<'info>,
  receiver: AccountInfo<'info>,
  amount: u64,
  bump: u8,
  system_program: AccountInfo<'info>,
) -> Result<()> {
  let transfer_sol_instruction = anchor_lang::system_program::Transfer {
    from: sender.to_account_info(),
    to: receiver.to_account_info(),
  };
  let seeds = &[&[b"state", bytemuck::bytes_of(&bump)][..]];
  let cpi_ctx_sol = CpiContext::new_with_signer(
    system_program.to_account_info(),
    transfer_sol_instruction,
    seeds,
  );
  anchor_lang::system_program::transfer(cpi_ctx_sol, amount)?;

  return Ok(());
}

pub fn transfer_token<'info>(
  sender: AccountInfo<'info>,
  receiver: AccountInfo<'info>,
  user: AccountInfo<'info>,
  amount: u64,
  seeds: &[&[&[u8]]],
  token_program: AccountInfo<'info>,
) -> Result<()> {
  let transfer_instruction_account = Transfer {
    from: sender.to_account_info(),
    to: receiver.to_account_info(),
    authority: user.to_account_info(),
  };
  let cpi_ctx = CpiContext::new_with_signer(
    token_program.to_account_info(),
    transfer_instruction_account,
    seeds,
  );

  anchor_spl::token::transfer(cpi_ctx, amount)?;
  Ok(())
}

pub fn to_close_account<'info>(
  escrow_wallet_associate_account: AccountInfo<'info>,
  user: AccountInfo<'info>,
  state_account: AccountInfo<'info>,
  outer: Vec<&[&[u8]]>,
  token_program: AccountInfo<'info>,
) -> Result<()> {
  let close_account = CloseAccount {
    account: escrow_wallet_associate_account.to_account_info(),
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
