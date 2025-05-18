import * as anchor from "@project-serum/anchor";
import { Program, BN, IdlAccounts } from "@project-serum/anchor";
import { PublicKey, Keypair, SystemProgram } from "@solana/web3.js";
import { EscrowDemo } from "../target/types/escrow_demo";
import { TOKEN_PROGRAM_ID, getAssociatedTokenAddress, createMint, mintTo, createAssociatedTokenAccount } from "@solana/spl-token";

describe("escrow-demo", () => {
  // Configure the client to use the local cluster.
  const provider = anchor.AnchorProvider.local();
  anchor.setProvider(provider);
  const program = anchor.workspace.EscrowDemo as Program<EscrowDemo>;

  // Keypairs
  const maker = provider.wallet.payer;
  const receiver = Keypair.generate();

  // Token mint and ATAs
  let mintA: PublicKey;
  let makerAtaA: PublicKey;
  let receiverAta: PublicKey;

  // PDA bump and address
  let escrowPda: PublicKey;
  let escrowBump: number;

  // Vault ATA
  let vaultAta: PublicKey;

  // Amount to escrow
  const amount = new BN(1_000);

  before(async () => {
    // Airdrop to receiver so they can pay for ATA & transactions
    await provider.connection.requestAirdrop(receiver.publicKey, 1e9);

    // 1. Create Mint A (decimals = 6)
    mintA = await createMint(
      provider.connection,
      maker,          // feePayer
      maker.publicKey,
      null,
      6,
    );

    // 2. Maker's ATA for mintA
    makerAtaA = await getAssociatedTokenAddress(mintA, maker.publicKey);
    await createAssociatedTokenAccount(
      provider.connection,
      maker,          // payer
      mintA,
      maker.publicKey
    );

    // Mint some tokens to makerAtaA
    await mintTo(
      provider.connection,
      maker,
      mintA,
      makerAtaA,
      maker.publicKey,
      10_000 * (10 ** 6)
    );
  });

  it("1. make & deposit", async () => {
    // Derive the escrow PDA
    [escrowPda, escrowBump] = await PublicKey.findProgramAddress(
      [Buffer.from("escrow"), maker.publicKey.toBuffer(), Buffer.from(new BN(42).toArray("le", 8))],
      program.programId
    );

    // Vault ATA (PDA-owned)
    vaultAta = await getAssociatedTokenAddress(mintA, escrowPda, true);

    // 1a. Call `make`
    await program.methods
      .make(new BN(42), amount)
      .accounts({
        maker: maker.publicKey,
        mintA,
        makerAtaA,
        escrow: escrowPda,
        vault: vaultAta,
        systemProgram: SystemProgram.programId,
        tokenProgram: TOKEN_PROGRAM_ID,
        associatedTokenProgram: anchor.utils.token.ASSOCIATED_PROGRAM_ID,
      })
      .signers([])
      .rpc();

    // Fetch and assert EscrowState
    const state = await program.account.escrowState.fetch(escrowPda);
    anchor.assert.ok(state.maker.equals(maker.publicKey));
    anchor.assert.ok(state.mintA.equals(mintA));
    anchor.assert.ok(state.seed.eq(new BN(42)));
    anchor.assert.ok(state.amount.eq(amount));
    anchor.assert.ok(state.bump === escrowBump);
    anchor.assert.ok(state.receiver.equals(PublicKey.default));

    // 1b. Call `deposit`
    await program.methods
      .deposit(new BN(1_000))
      .accounts({
        maker: maker.publicKey,
        mintA,
        makerAtaA,
        vault: vaultAta,
        tokenProgram: TOKEN_PROGRAM_ID,
      })
      .rpc();

    // Check vault balance
    const vaultBalance = await provider.connection.getTokenAccountBalance(vaultAta);
    anchor.assert.equal(vaultBalance.value.uiAmount, 1_000);
  });

  it("2. set_receiver", async () => {
    // Create receiver ATA ahead of Release
    receiverAta = await getAssociatedTokenAddress(mintA, receiver.publicKey);
    await createAssociatedTokenAccount(
      provider.connection,
      maker, // payer
      mintA,
      receiver.publicKey
    );

    // Call `setReceiver`
    await program.methods
      .setReceiver()
      .accounts({
        escrow: escrowPda,
        maker: maker.publicKey,
        receiver: receiver.publicKey,
      })
      .signers([receiver])  // receiver isn't a signer in constraints, but TS SDK requires pass
      .rpc();

    // Verify
    const updated = await program.account.escrowState.fetch(escrowPda);
    anchor.assert.ok(updated.receiver.equals(receiver.publicKey));
  });

  it("3. release & close", async () => {
    // Call `release`
    await program.methods
      .release()
      .accounts({
        maker: maker.publicKey,
        receiver: receiver.publicKey,
        escrow: escrowPda,
        receiverAta,
        vault: vaultAta,
        mintA,
        tokenProgram: TOKEN_PROGRAM_ID,
        associatedTokenProgram: anchor.utils.token.ASSOCIATED_PROGRAM_ID,
        systemProgram: SystemProgram.programId,
      })
      .signers([receiver])
      .rpc();

    // After release + close, escrow account should be gone
    const escrowInfo = await provider.connection.getAccountInfo(escrowPda);
    anchor.assert.ok(escrowInfo === null);

    // Vault ATA should be closed => getAccountInfo null
    const vaultInfo = await provider.connection.getAccountInfo(vaultAta);
    anchor.assert.ok(vaultInfo === null);

    // Receiver ATA should have the tokens
    const recvBal = await provider.connection.getTokenAccountBalance(receiverAta);
    anchor.assert.equal(recvBal.value.uiAmount, 1_000);
  });
});
