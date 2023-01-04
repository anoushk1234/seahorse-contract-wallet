import { LAMPORTS_PER_SOL } from "@solana/web3.js";
import * as anchor from "@project-serum/anchor";
import { Program } from "@project-serum/anchor";
import { SeahourseContractWallet } from "../target/types/seahourse_contract_wallet";
import { airdrop, getBalance } from "./helpers";
import { assert } from "chai";

describe("seahourse_contract_wallet", () => {
  // Configure the client to use the local cluster.
  anchor.setProvider(anchor.AnchorProvider.env());

  const program = anchor.workspace
    .SeahourseContractWallet as Program<SeahourseContractWallet>;
  const authority = anchor.web3.Keypair.generate();
  const recoverAuthority = anchor.web3.Keypair.generate();
  const seeds = [
    anchor.utils.bytes.utf8.encode("safe"),
    authority.publicKey.toBuffer(),
    anchor.utils.bytes.utf8.encode("my_safe"),
  ];
  let previousBalance;
  let currentBalance;
  it("Is initialized!", async () => {
    // Add your test here.
    await airdrop(authority.publicKey);
    const [safe_account, bump] = await anchor.web3.PublicKey.findProgramAddress(
      seeds,
      program.programId
    );
    const tx = await program.methods
      .initSafe("my_safe", recoverAuthority.publicKey)
      .accounts({
        safe: safe_account,
        owner: authority.publicKey,
        systemProgram: anchor.web3.SystemProgram.programId,
      })
      .signers([authority])
      .rpc();

    console.log("Your transaction signature", tx);
  });
  it("Can deposit", async () => {
    const [safe_account, bump] = await anchor.web3.PublicKey.findProgramAddress(
      seeds,
      program.programId
    );
    previousBalance = await getBalance(safe_account);
    await airdrop(safe_account);
    currentBalance = await getBalance(safe_account);
    console.log("Previous balance", previousBalance);
    console.log("Current balance", currentBalance);
  });
  it("Can withdraw", async () => {
    const [safe_account, bump] = await anchor.web3.PublicKey.findProgramAddress(
      seeds,
      program.programId
    );
    const tx = await program.methods
      .withdrawFunds(new anchor.BN(1 * LAMPORTS_PER_SOL))
      .accounts({
        safe: safe_account,
        to: authority.publicKey,
        authority: authority.publicKey,
      })
      .signers([authority])
      .rpc();

    console.log("Your transaction signature", tx);
    const newBalance = await getBalance(safe_account);
    assert(currentBalance - newBalance == 1 * LAMPORTS_PER_SOL);
  });
  it("can recover", async () => {
    const [safe_account, bump] = await anchor.web3.PublicKey.findProgramAddress(
      seeds,
      program.programId
    );

    const newAuthority = anchor.web3.Keypair.generate();
    const tx = await program.methods
      .recoverSafe(newAuthority.publicKey)
      .accounts({
        safe: safe_account,
        recoverAuthority: recoverAuthority.publicKey,
      })
      .signers([recoverAuthority])
      .rpc();

    console.log("Your transaction signature", tx);
    const safeRecoveryAuthority = (
      await program.account.safe.fetch(safe_account)
    ).owner;
    assert(
      safeRecoveryAuthority.toBase58() === newAuthority.publicKey.toBase58(),
      "authority failed to update"
    );
  });
});
