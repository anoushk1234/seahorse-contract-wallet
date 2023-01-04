import * as anchor from "@project-serum/anchor";
// import { TOKEN_PROGRAM_ID } from "@project-serum/anchor/dist/cjs/utils/token";
import { PublicKey, LAMPORTS_PER_SOL } from "@solana/web3.js";

const provider = anchor.getProvider();

export async function airdrop(key: PublicKey) {
  const airdropSig = await provider.connection.requestAirdrop(
    key,
    10 * LAMPORTS_PER_SOL
  );
  return provider.connection.confirmTransaction(airdropSig);
}

export async function getBalance(key: PublicKey) {
  console.log(provider.connection.rpcEndpoint);
  return await provider.connection.getBalance(key);
}
