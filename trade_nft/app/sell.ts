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

(async () => {
  const idl = require("/Users/minhdo/Documents/learning/solana-programs/trade_nft/target/idl/trade_nft.json");
  const connection = new Connection("http://127.0.0.1:8899", "processed");

  const payer = Keypair.fromSecretKey(
    new Uint8Array([
      100, 49, 56, 89, 216, 18, 46, 201, 245, 41, 79, 132, 160, 239, 120, 149,
      79, 68, 202, 136, 112, 130, 138, 192, 9, 58, 155, 178, 147, 143, 120, 24,
      98, 99, 48, 114, 112, 245, 112, 153, 140, 153, 71, 195, 54, 41, 153, 193,
      245, 182, 246, 21, 149, 37, 50, 74, 53, 98, 101, 154, 182, 29, 90, 125,
    ])
  ) as anchor.web3.Keypair;
  const buyer = Keypair.fromSecretKey(
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
    "DGWSLJhanAG1mkLgbGtCh2pL4W7CNnbbzY3fmyuh7CZe"
  );
  const program = new anchor.Program(idl, Program_ID, provider);

  // const mintAddress = new PublicKey(
  //   "9H2u1qMjUMtTcbWWtbQK1TCZvm87Jdxs5gyCxtu8HETz"
  // );
  const mintAddress = new PublicKey(
    "Dyk8Ypb1b8S4qfpo8iuu5U5i4uEjByDaYh9qXDxPY1aj"
  );
  const associatedTokenAccount = await getAssociatedTokenAddress(
    mintAddress,
    provider.wallet.publicKey
  );
  const acc = anchor.web3.Keypair.generate();
  const escrowWalletAssociateAccount = await getAssociatedTokenAddress(
    mintAddress,
    acc.publicKey
  );

  const mint_tx = new anchor.web3.Transaction().add(
    // Create the ATA account that is associated with our To wallet
    createAssociatedTokenAccountInstruction(
      provider.wallet.publicKey,
      escrowWalletAssociateAccount,
      acc.publicKey,
      mintAddress
    )
  );

  const m = await provider.sendAndConfirm(mint_tx, []);
  console.log(associatedTokenAccount.toString());
  console.log(escrowWalletAssociateAccount.toString());
  // Executes our transfer smart contract
  let [statePubKey, stateBump] = await anchor.web3.PublicKey.findProgramAddress(
    [Buffer.from("state"), payer.publicKey.toBuffer(), mintAddress.toBuffer()],
    program.programId
  );
  let [walletPubKey, walletBump] =
    await anchor.web3.PublicKey.findProgramAddress(
      [
        Buffer.from("wallet"),
        payer.publicKey.toBuffer(),
        mintAddress.toBuffer(),
      ],
      program.programId
    );
  console.log({
    state_account: statePubKey.toString(),
    escrow: walletPubKey.toString(),
  });

  console.log("------------------------");

  let data;
  try {
    data = await program.account.state.fetch(statePubKey);
  } catch (err) {
    //
  }
  console.log("data: ", data);
  if (!data) {
    console.log("create...");
    const create = await program.rpc.createTradeOrder({
      accounts: {
        seller: provider.wallet.publicKey,
        stateAccount: statePubKey,
        escrowWalletAssociateAccount: walletPubKey,
        nftMint: mintAddress,
        sellerAssociatedAccount: associatedTokenAccount,
        systemProgram: SystemProgram.programId,
        tokenProgram: TOKEN_PROGRAM_ID,
        rent: SYSVAR_RENT_PUBKEY,
      },
    });
    console.log("create sell order: ", create);
  }

  const state0 = await program.account.state.fetch(statePubKey);
  console.log(state0);
  console.log(state0.price.toString());
  const tx_sell = await program.rpc.sell(
    new BN(anchor.web3.LAMPORTS_PER_SOL),
    new BN(1),
    {
      accounts: {
        seller: provider.wallet.publicKey,
        stateAccount: statePubKey,
        escrowWalletAssociateAccount: walletPubKey,
        nftMint: mintAddress,
        sellerAssociatedAccount: associatedTokenAccount,
        systemProgram: SystemProgram.programId,
        tokenProgram: TOKEN_PROGRAM_ID,
      },
    }
  );
  console.log(tx_sell);
  // const state = await program.account.state.fetch(statePubKey);
  // console.log(state);

  console.log("start buy-----------");
  const buyerWalletAssociateAccount = await getAssociatedTokenAddress(
    mintAddress,
    buyer.publicKey
  );

  // const buyer_tx = new anchor.web3.Transaction().add(
  //   // Create the ATA account that is associated with our To wallet
  //   createAssociatedTokenAccountInstruction(
  //     provider.wallet.publicKey,
  //     buyerWalletAssociateAccount,
  //     buyer.publicKey,
  //     mintAddress
  //   )
  // );

  // const buyer_sig = await provider.sendAndConfirm(buyer_tx, []);
  // console.log(buyer_sig)
  const tx_buy = await program.rpc.buy(new BN(1000000000), {
    accounts: {
      buyer: buyer.publicKey,
      seller: provider.wallet.publicKey,
      stateAccount: statePubKey,
      nftMint: mintAddress,
      buyerAssociatedAccount: buyerWalletAssociateAccount,
      escrowWalletAssociateAccount: walletPubKey,
      systemProgram: SystemProgram.programId,
      tokenProgram: TOKEN_PROGRAM_ID,
    },
    signers: [buyer]
  });
  console.log(tx_buy);


  // const state = await program.account.state.fetch(statePubKey);
  // console.log({
  //   seller: state.seller.toString(),
  //   mint: state.mintNft.toString(),
  //   escrow: state.escrowAssociateWallet.toString(),
  //   price: state.price.toString(),
  // });
})();
