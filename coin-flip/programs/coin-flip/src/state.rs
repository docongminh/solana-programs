use anchor_lang::prelude::*;

#[derive(AnchorSerialize, AnchorDeserialize, Clone, Default)]
pub struct Bumps {
  pub state_bump: u8,
  pub wallet_native_associate_bump: u8,
}

#[account]
pub struct StateFlipAccount {
  pub creator: Pubkey,
  pub acceptor: Pubkey,
  pub fee_account: Pubkey,
  pub flip_value: u64,
  pub balance: u64,
  pub id: u64,
  pub bumps: Bumps,
}

impl StateFlipAccount {
  pub const LEN: usize = 8 // internal discriminator
    + 3*32 // PubKey
    + 3*8 // u64
    + 2; // u8
}
