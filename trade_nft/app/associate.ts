import * as anchor from "@project-serum/anchor";
import { Program } from "@project-serum/anchor";
import {
  Connection,
  PublicKey,
  Keypair,
  SystemProgram,
  SYSVAR_RENT_PUBKEY,
} from "@solana/web3.js";
import { BN } from "bn.js";
import {
  TOKEN_PROGRAM_ID,
  MINT_SIZE,
  createAssociatedTokenAccountInstruction,
  getAssociatedTokenAddress,
  createInitializeMintInstruction,
  ASSOCIATED_TOKEN_PROGRAM_ID,
} from "@solana/spl-token";

export async function createAssociate(provider: anchor.AnchorProvider, mintAddress: PublicKey, owner: PublicKey){
  const escrowWalletAssociateAccount = await getAssociatedTokenAddress(
    mintAddress,
    owner
  );

  const mint_tx = new anchor.web3.Transaction().add(
    // Create the ATA account that is associated with our To wallet
    createAssociatedTokenAccountInstruction(
      provider.wallet.publicKey,
      escrowWalletAssociateAccount,
      owner,
      mintAddress
    )
  );

  const m = await provider.sendAndConfirm(mint_tx, []);

  return escrowWalletAssociateAccount;
}