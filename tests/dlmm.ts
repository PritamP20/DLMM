import * as anchor from "@coral-xyz/anchor";
import { Program } from "@coral-xyz/anchor";
import { Dlmm } from "../target/types/dlmm";
import { PublicKey, Keypair, SystemProgram, SYSVAR_RENT_PUBKEY } from "@solana/web3.js";
import { TOKEN_PROGRAM_ID, createMint, createAccount, mintTo, getAccount, getAssociatedTokenAddress, getOrCreateAssociatedTokenAccount } from "@solana/spl-token";
import { assert } from "chai";

describe("dlmm", () => {
  const provider = anchor.AnchorProvider.env();
  anchor.setProvider(provider);

  const program = anchor.workspace.dlmm as Program<Dlmm>;

  let tokenX: PublicKey;
  let tokenY: PublicKey;
  let userTokenX: PublicKey;
  let userTokenY: PublicKey;
  let lbPair: PublicKey;
  let reserveX: PublicKey;
  let reserveY: PublicKey;
  let position: PublicKey;
  let binArray: PublicKey;

  const user = Keypair.generate();
  const binStep = 100;
  const activeId = 0;
  const binArrayIndex = 0;

  before(async () => {
    const signature = await provider.connection.requestAirdrop(user.publicKey, 10 * anchor.web3.LAMPORTS_PER_SOL);
    await provider.connection.confirmTransaction(signature);

    tokenX = await createMint(provider.connection, user, user.publicKey, null, 6);
    tokenY = await createMint(provider.connection, user, user.publicKey, null, 6);

    userTokenX = await createAccount(provider.connection, user, tokenX, user.publicKey);
    userTokenY = await createAccount(provider.connection, user, tokenY, user.publicKey);

    await mintTo(provider.connection, user, tokenX, userTokenX, user, 10_000_000_000); // 10,000 X
    await mintTo(provider.connection, user, tokenY, userTokenY, user, 10_000_000_000); // 10,000 Y

    const [lbPairPda] = PublicKey.findProgramAddressSync(
      [Buffer.from("lb_pair"), tokenX.toBuffer(), tokenY.toBuffer()],
      program.programId
    );
    lbPair = lbPairPda;
  });

  it("Initialize LbPair", async () => {

    await program.methods
      .initializeLbPair(binStep)
      .accounts({
        lbPair: lbPair,
        user: user.publicKey,
        tokenXMint: tokenX,
        tokenYMint: tokenY,
        systemProgram: SystemProgram.programId,
      } as any)
      .signers([user])
      .rpc();

    const account = await program.account.lbPair.fetch(lbPair);
    assert.ok(account.tokenXMint.equals(tokenX));
    assert.ok(account.tokenYMint.equals(tokenY));
    assert.equal(account.binStep, binStep);
  });

  it("Initialize BinArray", async () => {
    const indexBuffer = Buffer.alloc(4);
    indexBuffer.writeInt32LE(binArrayIndex, 0);

    const [binArrayPda, bump] = PublicKey.findProgramAddressSync(
      [
        Buffer.from("bin_array"),
        lbPair.toBuffer(),
        indexBuffer
      ],
      program.programId
    );
    binArray = binArrayPda;

    try {
      const tx = await program.methods
        .initializeBinArray(binArrayIndex)
        .accounts({
          lbPair: lbPair,
          user: user.publicKey,
        })
        .signers([user])
        .rpc();

      console.log("Transaction: ", tx);
      await provider.connection.confirmTransaction(tx);
      const accountInfo = await provider.connection.getAccountInfo(binArray);

      if (!accountInfo) {
        throw new Error("BinArray was not insitilaized");
      }

      const binArrayAccount = await program.account.binArray.fetch(binArray);
      assert.ok(binArrayAccount.lbPair.equals(lbPair));
      assert.equal(binArrayAccount.index, binArrayIndex);
      assert.equal(binArrayAccount.bump, bump);

    } catch (error) {
      console.log(error);
    }
  });

  it("Prepare Reserves", async () => {
    const rx = await getOrCreateAssociatedTokenAccount(
      provider.connection,
      user,
      tokenX,
      lbPair,
      true
    );
    reserveX = rx.address;

    const ry = await getOrCreateAssociatedTokenAccount(
      provider.connection,
      user,
      tokenY,
      lbPair,
      true
    );
    reserveY = ry.address;
  });

  it("Add Liquidity", async () => {
    const [positionPda] = PublicKey.findProgramAddressSync(
      [Buffer.from("position"), lbPair.toBuffer(), user.publicKey.toBuffer()],
      program.programId
    );
    position = positionPda;

    const amountX = new anchor.BN(1_000_000);
    const amountY = new anchor.BN(1_000_000);
    const binDist = [
      { deltaId: 0, distX: 5000, distY: 5000 }
    ];

    await program.methods
      .addLiquidity(amountX, amountY, binDist)
      .accounts({
        lbPair: lbPair,
        binArray: binArray,
        position: position,
        userTokenX: userTokenX,
        userTokenY: userTokenY,
        reserveX: reserveX,
        reserveY: reserveY,
        user: user.publicKey,
        tokenProgram: TOKEN_PROGRAM_ID,
        systemProgram: SystemProgram.programId,
      } as any)
      .signers([user])
      .rpc();

    const posAccount = await program.account.position.fetch(position);
    assert.ok(posAccount.lbPair.equals(lbPair));
    assert.ok(posAccount.owner.equals(user.publicKey));

    const baAccount = await program.account.binArray.fetch(binArray);
    const bin = baAccount.bins[0];
    assert.equal(bin.reserveX.toString(), "500000");
    assert.equal(bin.reserveY.toString(), "500000");
    assert.ok(bin.totalShares.gt(new anchor.BN(0)));
  });

  it("Remove Liquidity", async () => {
    const posAccount = await program.account.position.fetch(position);
    const shares = posAccount.liquidityShares[0];
    const halfShares = shares.div(new anchor.BN(2));

    const removal = [
      { binId: 0, sharesToBurn: halfShares }
    ];

    await program.methods
      .removeLiquidity(removal)
      .accounts({
        lbPair: lbPair,
        binArray: binArray,
        position: position,
        userTokenX: userTokenX,
        userTokenY: userTokenY,
        reserveX: reserveX,
        reserveY: reserveY,
        user: user.publicKey,
        tokenProgram: TOKEN_PROGRAM_ID,
      } as any)
      .signers([user])
      .rpc();

    const baAccount = await program.account.binArray.fetch(binArray);
    const bin = baAccount.bins[0];
    assert.ok(bin.reserveX.lt(new anchor.BN(250005)));
    assert.ok(bin.reserveX.gt(new anchor.BN(249995)));
  });

  it("Swap X for Y", async () => {
    const amountIn = new anchor.BN(100_000);
    const minAmountOut = new anchor.BN(65_000);
    const swapForY = true;

    const tx = await program.methods
      .swap(amountIn, minAmountOut, swapForY)
      .accounts({
        lbPair: lbPair,
        binArray: binArray,
        user: user.publicKey,
        userXToken: userTokenX,
        userYToken: userTokenY,
        reserveX: reserveX,
        reserveY: reserveY,
        tokenProgram: TOKEN_PROGRAM_ID,
      } as any)
      .signers([user])
      .rpc();

    const baAccount = await program.account.binArray.fetch(binArray);
    const bin = baAccount.bins[0];
    assert.ok(bin.reserveX.gt(new anchor.BN(300_000)));
    assert.ok(bin.reserveY.lt(new anchor.BN(200_000)));
  });

  it("Swap Y for X", async () => {
    const amountIn = new anchor.BN(50_000);
    const minAmountOut = new anchor.BN(40_000);
    const swapForY = false;

    await program.methods
      .swap(amountIn, minAmountOut, swapForY)
      .accounts({
        lbPair: lbPair,
        binArray: binArray,
        user: user.publicKey,
        userXToken: userTokenX,
        userYToken: userTokenY,
        reserveX: reserveX,
        reserveY: reserveY,
        tokenProgram: TOKEN_PROGRAM_ID,
      } as any)
      .signers([user])
      .rpc();

    const baAccount = await program.account.binArray.fetch(binArray);
    const bin = baAccount.bins[0];
    assert.ok(bin.reserveY.gt(new anchor.BN(150_000)));
  });

  it("Fail: Remove more liquidity than owned", async () => {
    const removal = [
      { binId: 0, sharesToBurn: new anchor.BN("1000000000000000000") } // Huge amount
    ];

    try {
      await program.methods
        .removeLiquidity(removal)
        .accounts({
          lbPair: lbPair,
          binArray: binArray,
          position: position,
          userTokenX: userTokenX,
          userTokenY: userTokenY,
          reserveX: reserveX,
          reserveY: reserveY,
          user: user.publicKey,
          tokenProgram: TOKEN_PROGRAM_ID,
        } as any)
        .signers([user])
        .rpc();
      assert.fail("Should have failed");
    } catch (e) {
      assert.ok(true);
    }
  });
});
