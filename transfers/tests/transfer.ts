import * as anchor from "@project-serum/anchor";
import { Program } from "@project-serum/anchor";
import { Transfer } from "../target/types/transfer";

describe("transfer", () => {
  // Configure the client to use the local cluster.
  anchor.setProvider(anchor.AnchorProvider.env());

  const program = anchor.workspace.Transfer as Program<Transfer>;

  it("Is initialized!", async () => {
    // Add your test here.
    const tx = await program.methods.initialize().rpc();
    console.log("Your transaction signature", tx);
  });
});
