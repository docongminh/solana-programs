use anchor_lang::prelude::*;
use anchor_spl::token::Transfer;

pub fn transfer_sol<'info>(
  sender: AccountInfo<'info>,
  receiver: AccountInfo<'info>,
  amount: u64,
  system_program: AccountInfo<'info>,
  seeds: Option<&[&[&[u8]]]>,
) -> Result<()> {
  let transfer_sol_instruction = anchor_lang::system_program::Transfer {
    from: sender.to_account_info(),
    to: receiver.to_account_info(),
  };
  match seeds {
    Some(seeds) => {
      let transfer_sol_account = anchor_lang::system_program::Transfer {
        from: sender.to_account_info(),
        to: receiver.to_account_info(),
      };
      let cpi_ctx_sol = CpiContext::new_with_signer(
        system_program.to_account_info(),
        transfer_sol_account,
        seeds,
      );
      anchor_lang::system_program::transfer(cpi_ctx_sol, amount)?;
    }
    None => {
      let cpi_ctx_sol = CpiContext::new(system_program.to_account_info(), transfer_sol_instruction);
      anchor_lang::system_program::transfer(cpi_ctx_sol, amount)?;
    }
  }
  return Ok(());
}

// transfer fungible token & nft token
pub fn transfer_token<'info>(
  sender: AccountInfo<'info>,
  receiver: AccountInfo<'info>,
  user: AccountInfo<'info>,
  amount: u64,
  token_program: AccountInfo<'info>,
  seeds: Option<&[&[&[u8]]]>,
) -> Result<()> {
  let transfer_instruction_account = Transfer {
    from: sender.to_account_info(),
    to: receiver.to_account_info(),
    authority: user.to_account_info(),
  };
  let cpi_ctx;
  match seeds {
    Some(seeds) => {
      cpi_ctx = CpiContext::new_with_signer(
        token_program.to_account_info(),
        transfer_instruction_account,
        seeds,
      );
    }
    None => {
      cpi_ctx = CpiContext::new(
        token_program.to_account_info(),
        transfer_instruction_account,
      );
    }
  }
  anchor_spl::token::transfer(cpi_ctx, amount)?;
  Ok(())
}

pub fn random_select() -> Result<u8> {
  // 0: creator | 1: acceptor
  // TODO late
  return Ok(1);
}
