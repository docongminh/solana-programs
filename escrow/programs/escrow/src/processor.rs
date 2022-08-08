use anchor_lang::prelude::*;

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
