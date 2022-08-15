pub mod error;
pub mod processor;
pub mod state;

use anchor_lang::prelude::*;
use crate::state::{StateFlipAccount};
use crate::processor::{transfer_sol, random_select};
use crate::error::ErrorCode;

declare_id!("4LtpC4z4WpJcApWXM8Hm5N7iUP1C5j51NVyyYj334L3w");

const ESCROW_PDA_SEED: &[u8] = b"escrow_flip_state";
#[program]
pub mod coin_flip {
    use super::*;

    pub fn create_flip_order(ctx: Context<Initialize>, id: u64, flip_value: u64) -> Result<()> {
        let state_account = &mut ctx.accounts.state_account;
        state_account.creator = ctx.accounts.creator.key();
        state_account.id = id;
        state_account.flip_value = flip_value;
        state_account.fee_account = ctx.accounts.fee_account.key();
        let state_bump = *ctx.bumps.get("state_account").unwrap();
        state_account.bumps.state_bump = state_bump;
        transfer_sol(
            ctx.accounts.creator.to_account_info(),
            state_account.to_account_info(),
            flip_value,
            ctx.accounts.system_program.to_account_info(),
            None
        )?;
        state_account.balance = state_account.to_account_info().lamports();
        Ok(())
    }

    pub fn accept_flip(ctx: Context<Flip>) -> Result<()>{
        ctx.accounts.state_account.acceptor = ctx.accounts.acceptor.key();
        transfer_sol(
            ctx.accounts.acceptor.to_account_info(),
            ctx.accounts.state_account.to_account_info(),
            ctx.accounts.state_account.flip_value,
            ctx.accounts.system_program.to_account_info(),
            None
        )?;
        let balance = ctx.accounts.state_account.to_account_info().lamports();
        // let creator = ctx.accounts.state_account.creator;
        // let id_bytes = id.to_le_bytes();
        // let seeds = &[&[ESCROW_PDA_SEED, creator.as_ref(), id_bytes.as_ref(), bytemuck::bytes_of(&ctx.accounts.state_account.bumps.state_bump)][..]];
        // get fee 2% per deal
        let fee_amount = (balance * 2)/100;
        let amount_receive = balance - fee_amount;
        // flip find
        let winner = random_select().unwrap();
        
        match winner {
            0 => {
                **ctx.accounts.state_account.to_account_info().try_borrow_mut_lamports()? -= amount_receive;
                **ctx.accounts.creator.to_account_info().try_borrow_mut_lamports()? += amount_receive;
            }
            1 => {
                **ctx.accounts.state_account.to_account_info().try_borrow_mut_lamports()? -= amount_receive;
                **ctx.accounts.acceptor.to_account_info().try_borrow_mut_lamports()? += amount_receive;
            }
            _ => todo!()
           // @TODO None => 
        }
        // transfer fee to account fee
        **ctx.accounts.state_account.to_account_info().try_borrow_mut_lamports()? -= fee_amount;
        **ctx.accounts.fee_account.to_account_info().try_borrow_mut_lamports()? += fee_amount;
        Ok(())
    }
}

#[derive(Accounts)]
#[instruction(id: u64, amount: u64)]
pub struct Initialize<'info> {
    #[account(
        init,
        payer = creator,
        space = StateFlipAccount::LEN,
        seeds=[ESCROW_PDA_SEED, creator.key().as_ref(), id.to_le_bytes().as_ref()],
        bump,    
    )]
    state_account: Account<'info, StateFlipAccount>,
    /// CHECK:
    fee_account: AccountInfo<'info>,
    #[account(mut, constraint = creator.lamports() > amount @ ErrorCode::InsufficientFunds)]
    creator: Signer<'info>,
    system_program: Program<'info, System>,
    rent: Sysvar<'info, Rent>
}

#[derive(Accounts)]
#[instruction(id: u64)]
pub struct Flip<'info>{
    #[account(mut, has_one=creator, has_one=fee_account, seeds=[ESCROW_PDA_SEED, creator.key().as_ref(), id.to_le_bytes().as_ref()], bump = state_account.bumps.state_bump)]
    state_account: Account<'info, StateFlipAccount>,
    #[account(mut, constraint = acceptor.lamports() > state_account.flip_value @ ErrorCode::InsufficientFunds)]
    /// CHECK:
    fee_account: AccountInfo<'info>,
    #[account(mut)]
    acceptor: Signer<'info>,
    /// CHECK:
    #[account(mut)]
    creator: AccountInfo<'info>,
    system_program: Program<'info, System>,
}
