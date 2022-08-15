import * as anchor from "@project-serum/anchor";
import { Program } from "@project-serum/anchor";
import { Connection, PublicKey, Keypair, SystemProgram, SYSVAR_RENT_PUBKEY } from "@solana/web3.js";
import { BN } from "bn.js";
import {
  TOKEN_PROGRAM_ID,
  MINT_SIZE,
  createAssociatedTokenAccountInstruction,
  getAssociatedTokenAddress,
  createInitializeMintInstruction,
  ASSOCIATED_TOKEN_PROGRAM_ID,
  NATIVE_MINT
} from "@solana/spl-token";

(async () => {
  const idl = require("/Users/minhdo/Documents/learning/solana-programs/coin-flip/target/idl/coin_flip.json");
  const connection = new Connection("http://127.0.0.1:8899", "processed");

  const payer = Keypair.fromSecretKey(
    new Uint8Array([
      100, 49, 56, 89, 216, 18, 46, 201, 245, 41, 79, 132, 160, 239, 120, 149,
      79, 68, 202, 136, 112, 130, 138, 192, 9, 58, 155, 178, 147, 143, 120, 24,
      98, 99, 48, 114, 112, 245, 112, 153, 140, 153, 71, 195, 54, 41, 153, 193,
      245, 182, 246, 21, 149, 37, 50, 74, 53, 98, 101, 154, 182, 29, 90, 125,
    ])
  );
  const user2 = Keypair.fromSecretKey(
    new Uint8Array([
      212, 245, 203, 57, 102, 226, 6, 17, 83, 196, 112, 170, 217, 192, 250, 34,
      244, 225, 124, 7, 42, 11, 56, 2, 160, 216, 174, 129, 148, 99, 133, 219,
      101, 83, 202, 232, 236, 221, 209, 215, 166, 30, 221, 98, 161, 2, 19, 236,
      230, 56, 166, 9, 129, 199, 74, 52, 67, 244, 195, 95, 172, 17, 54, 81,
    ])
  );
  const wallet = new anchor.Wallet(payer);
  const provider = new anchor.AnchorProvider(connection, wallet, {
    commitment: "processed",
  });

  const Program_ID = new PublicKey(
    "4LtpC4z4WpJcApWXM8Hm5N7iUP1C5j51NVyyYj334L3w"
  );
  const program = new anchor.Program(idl, Program_ID, provider);
  const id = 3;
  const uidBuffer = new BN(id).toBuffer('le', 8);
  let [statePubKey, stateBump] = await anchor.web3.PublicKey.findProgramAddress(
    [
      Buffer.from("escrow_flip_state"),
      payer.publicKey.toBuffer(),
      uidBuffer,
    ],
    program.programId
  );
  const tx1 = await program.rpc.createFlipOrder(
    new BN(id),
    new BN(anchor.web3.LAMPORTS_PER_SOL),
    {
    accounts: {
      stateAccount: statePubKey,
      creator: provider.wallet.publicKey,
      feeAccount: new PublicKey("ykkvsfEtAhc7faxK3uJTYMPBmtrisGkU9Kv4SnuxzB7"),
      systemProgram: SystemProgram.programId,
      rent: SYSVAR_RENT_PUBKEY
    },
  });
  console.log("create: ", tx1);

  // Flip
  const tx2 = await program.rpc.acceptFlip(
    new BN(id),
    {
      accounts: {
        stateAccount: statePubKey,
        acceptor: user2.publicKey,
        creator: provider.wallet.publicKey,
        feeAccount: new PublicKey("ykkvsfEtAhc7faxK3uJTYMPBmtrisGkU9Kv4SnuxzB7"),
        systemProgram: SystemProgram.programId,
      },
      signers: [user2]
  });
  console.log("flip: ", tx2);

})();
