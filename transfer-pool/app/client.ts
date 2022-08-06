import * as anchor from "@project-serum/anchor";
import { Program } from "@project-serum/anchor";
import { Connection, PublicKey, Keypair, SystemProgram } from "@solana/web3.js";
import { BN } from "bn.js";
import {
  TOKEN_PROGRAM_ID,
  MINT_SIZE,
  createAssociatedTokenAccountInstruction,
  getAssociatedTokenAddress,
  createInitializeMintInstruction,
  ASSOCIATED_TOKEN_PROGRAM_ID,
} from "@solana/spl-token";

(async () => {
  const idl = require("/Users/minhdo/Documents/learning/solana-programs/transfer-pool/target/idl/transfer.json");
  const connection = new Connection("http://127.0.0.1:8899", "processed");

  const payer = Keypair.fromSecretKey(
    new Uint8Array([
      100, 49, 56, 89, 216, 18, 46, 201, 245, 41, 79, 132, 160, 239, 120, 149,
      79, 68, 202, 136, 112, 130, 138, 192, 9, 58, 155, 178, 147, 143, 120, 24,
      98, 99, 48, 114, 112, 245, 112, 153, 140, 153, 71, 195, 54, 41, 153, 193,
      245, 182, 246, 21, 149, 37, 50, 74, 53, 98, 101, 154, 182, 29, 90, 125,
    ])
  );
  const wallet = new anchor.Wallet(payer);
  const provider = new anchor.AnchorProvider(connection, wallet, {
    commitment: "processed",
  });

  const Program_ID = new PublicKey(
    "Wk1uGMfZR6YhTjLAaUD1e944VcrgKvZXsFVPonjy1yD"
  );
  const program = new anchor.Program(idl, Program_ID, provider);

  const mintAddress = new PublicKey(
    "9mw9qp7gdHzNRn9kRych4NMw6zXYySnVUrZF9H8bziRp"
  );


  const associatedTokenAccount = await getAssociatedTokenAddress(
    mintAddress,
    provider.wallet.publicKey
  );
  const toWallet = anchor.web3.Keypair.generate();
  const toATA = await getAssociatedTokenAddress(
    mintAddress,
    toWallet.publicKey
  );

  const mint_tx = new anchor.web3.Transaction().add(
    // Create the ATA account that is associated with our To wallet
    createAssociatedTokenAccountInstruction(
      provider.wallet.publicKey,
      toATA,
      toWallet.publicKey,
      mintAddress
    )
  );
  const m = await provider.sendAndConfirm(mint_tx, []);

  console.log(associatedTokenAccount.toString())
  console.log(toATA.toString())
  // Executes our transfer smart contract
  const tx = await program.rpc
    .transferToken(
      new BN(1000),
      {accounts: {
        tokenProgram: TOKEN_PROGRAM_ID,
        senderAssociate: associatedTokenAccount,
        receiverAssociate: toATA,
        sender: provider.wallet.publicKey,
        receiver: toWallet.publicKey,
      },
    })

  console.log(tx);

})();
