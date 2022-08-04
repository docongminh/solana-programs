import * as anchor from "@project-serum/anchor";
import { Program } from "@project-serum/anchor";
import { Connection, PublicKey, Keypair } from "@solana/web3.js";
import { BN } from "bn.js";

(async () => {
  const idl = require("/Users/minhdo/Documents/learning/solana-program/counter/target/idl/counter_app.json");
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
    "4UW83kGzjovz2gujn87fcDG7oEhjp43VgjHvaoAaCtPc"
  );
  const program = new anchor.Program(idl, Program_ID, provider);

  const counterAccount = anchor.web3.Keypair.generate();
  const account2 = anchor.web3.Keypair.generate()
  const signature = await program.rpc.init({
    accounts: {
      computeAccount: counterAccount.publicKey,
      user: provider.wallet.publicKey,
      authority: provider.wallet.publicKey,
      systemProgram: anchor.web3.SystemProgram.programId,
    },
    signers: [counterAccount],
  });
  console.log(signature)
  
  // const tx = await program.rpc.changeAuthority(
  //   account2.publicKey,
  //   {
  //   accounts: {
  //     computeAccount: counterAccount.publicKey,
  //     authority: provider.wallet.publicKey,
  //   },
  //   signers: [payer]
  // });
  // console.log(tx)

  const tx1 = await program.rpc.add(
    new BN(1),
    {
    accounts: {
      computeAccount: counterAccount.publicKey,
      authority: account2.publicKey,
    },
    signers: [account2]
  });
  console.log(tx1)

  // const tx2 = await program.rpc.sub(
  //   new BN(2),
  //   {
  //   accounts: {
  //     computeAccount: counterAccount.publicKey,
  //     authority: provider.wallet.publicKey,
  //   }
  // });
  // console.log(tx2)

  const account: any = await program.account.computeAccount.fetch(
    counterAccount.publicKey
  );
  console.log(account.total.toString());
})();
