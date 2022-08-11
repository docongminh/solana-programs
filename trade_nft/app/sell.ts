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
import { token } from "@project-serum/anchor/dist/cjs/utils";

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
    "4kVr2h7SZkWVV7DBoYEkL4gV7bzcp2T9TbuFqmNBcnUc"
  );
  const program = new anchor.Program(idl, Program_ID, provider);

  // const mintAddress = new PublicKey(
  //   "9H2u1qMjUMtTcbWWtbQK1TCZvm87Jdxs5gyCxtu8HETz"
  // );
  const mintAddress = new PublicKey(
    "63xg3iUnWWxQp3TYTF8MUbyDuMz4F8aYi8qJdiCxYeeo"
  );
  const token_mintAddress = new PublicKey(
    "HqR42d2WMPLWNXnR3RUj9RsTuJcd656KiPvAXibHgPPF"
  );
  const sellerAssociatedNFTAccount = await getAssociatedTokenAddress(
    mintAddress,
    provider.wallet.publicKey
  );
  const sellerAssociatedTokenAccount = await getAssociatedTokenAddress(
    token_mintAddress,
    provider.wallet.publicKey
  );
  const acc = anchor.web3.Keypair.generate();
  const escrowWalletAssociateAccount = await getAssociatedTokenAddress(
    mintAddress,
    acc.publicKey
  );
  try {
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
  } catch (error) {
    ///
  }
  // Executes our transfer smart contract
  let [statePubKey, stateBump] = await anchor.web3.PublicKey.findProgramAddress(
    [
      Buffer.from("escrow_state"),
      payer.publicKey.toBuffer(),
      mintAddress.toBuffer(),
    ],
    program.programId
  );
  let [walletPubKey, walletBump] =
    await anchor.web3.PublicKey.findProgramAddress(
      [
        Buffer.from("escrow_nft_associate"),
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
  console.log(data);
  if (data) {
    console.log("create...");
    const create = await program.rpc.createTradeOrder({
      accounts: {
        stateAccount: statePubKey,
        escrowAssociateNftWallet: walletPubKey,
        mintNft: mintAddress,
        mintToken: token_mintAddress,
        sellerAssociateNftAccount: sellerAssociatedNFTAccount,
        seller: provider.wallet.publicKey,
        systemProgram: SystemProgram.programId,
        tokenProgram: TOKEN_PROGRAM_ID,
        rent: SYSVAR_RENT_PUBKEY,
      },
    });
    console.log("create sell order: ", create);
  }

  // const state_create = await program.account.state.fetch(statePubKey);
  // console.log("state create: ", state_create);
  // console.log(associatedTokenAccount.toString());

  ///////////////////
  console.log("start sell");
  console.log(anchor.web3.LAMPORTS_PER_SOL * 10);
  console.log("sellerAssociatedNFTAccount", sellerAssociatedNFTAccount.toString())
  const tx_sell = await program.rpc.sell(
    new BN(anchor.web3.LAMPORTS_PER_SOL),
    new BN(anchor.web3.LAMPORTS_PER_SOL * 10),
    new BN(1),
    {
      accounts: {
        stateAccount: statePubKey,
        escrowAssociateNftWallet: walletPubKey,
        mintNft: mintAddress,
        mintToken: token_mintAddress,
        sellerAssociateNftAccount: sellerAssociatedNFTAccount,
        seller: provider.wallet.publicKey,
        systemProgram: SystemProgram.programId,
        tokenProgram: TOKEN_PROGRAM_ID,
      },
    }
  );
  console.log(tx_sell);
  // const state_sell = await program.account.state.fetch(statePubKey);
  // console.log("state sell: ", state_sell);

  // const tx_cancel = await program.rpc.cancel(
  //     {
  //       accounts: {
  //         seller: provider.wallet.publicKey,
  //         stateAccount: statePubKey,
  //         escrowAssociateNftWallet: walletPubKey,
  //         mintNft: mintAddress,
  //         sellerAssociateNftAccount: sellerAssociatedNFTAccount,
  //         systemProgram: SystemProgram.programId,
  //         tokenProgram: TOKEN_PROGRAM_ID,
  //       },
  //     }
  //   );
  //   console.log(tx_cancel);

  ////////////////////

  // const state_cancel = await program.account.state.fetch(statePubKey);
  // console.log("state cancel: ", state_cancel);

  const buyerAssociateNFTAccount = await getAssociatedTokenAddress(
    mintAddress,
    buyer.publicKey
  );
  try {
    const buyer_tx = new anchor.web3.Transaction().add(
      // Create the ATA account that is associated with our To wallet
      createAssociatedTokenAccountInstruction(
        provider.wallet.publicKey,
        buyerAssociateNFTAccount,
        buyer.publicKey,
        mintAddress
      )
    );

    const buyer_sig = await provider.sendAndConfirm(buyer_tx, []);
    console.log(buyer_sig);
  } catch (error) {
    console.log("NFT: ", error)
  }

  const buyerAssociateTokenAccount = await getAssociatedTokenAddress(
    token_mintAddress,
    buyer.publicKey
  );
  try {
    const buyer__tx = new anchor.web3.Transaction().add(
      // Create the ATA account that is associated with our To wallet
      createAssociatedTokenAccountInstruction(
        provider.wallet.publicKey,
        buyerAssociateTokenAccount,
        buyer.publicKey,
        token_mintAddress
      )
    );

    const buyer__sig = await provider.sendAndConfirm(buyer__tx, []);
    console.log(buyer__sig);
  } catch (error) {
    //
    console.log("TOKEN: ", error)
  }
  console.log("start buy");
  const tx_buy = await program.rpc.buy(
    false,
    new BN(1),
    new BN(0),
    new BN(anchor.web3.LAMPORTS_PER_SOL * 10),
    {
      accounts: {
        buyer: buyer.publicKey,
        seller: provider.wallet.publicKey,
        stateAccount: statePubKey,
        escrowAssociateNftWallet: walletPubKey,
        mintNft: mintAddress,
        mintToken: token_mintAddress,
        buyerAssociateNftAccount: buyerAssociateNFTAccount,


        buyerAssociateTokenAccount: buyerAssociateTokenAccount,
        sellerAssociateTokenAccount: sellerAssociatedTokenAccount,



        systemProgram: SystemProgram.programId,
        tokenProgram: TOKEN_PROGRAM_ID,
      },
      signers: [buyer],
    }
  );
  console.log(tx_buy);
})();
