import * as anchor from "@coral-xyz/anchor";
import { Program } from "@coral-xyz/anchor";
import { LAMPORTS_PER_SOL, PublicKey } from "@solana/web3.js";
import { createMint, getMint, TOKEN_PROGRAM_ID } from "@solana/spl-token";
import { assert } from "chai";
import { Stakely } from "../target/types/stakely";

describe("stakely", () => {
  // Configure the client to use the local cluster.
  // anchor.setProvider(anchor.AnchorProvider.env());
  const provider = anchor.AnchorProvider.env();
  anchor.setProvider(provider);

  const program = anchor.workspace.stakely as Program<Stakely>;

  let poolTitle = "The first pool";
  const admin = anchor.web3.Keypair.generate();
  const user1 = anchor.web3.Keypair.generate();
  const user2 = anchor.web3.Keypair.generate();
  let lstMint: PublicKey;
  let poolPda: PublicKey;
  let poolBump: number;
  let reservePda: PublicKey;
  let reserveBump: number;
  let stakeAccount1: PublicKey;
  let stakeAccount2: PublicKey;
  let unstakeAccount1: PublicKey;
  let unstakeAccount2: PublicKey;

  describe("AIRDROP AND CREATE LST MINT TOKEN", async () => {
    before(async () => {
      console.log("\t======================================");

      // airdrops
      await airdrop(
        provider.connection,
        admin.publicKey,
        10 * LAMPORTS_PER_SOL,
      );
      await airdrop(
        provider.connection,
        user1.publicKey,
        10 * LAMPORTS_PER_SOL,
      );
      await airdrop(
        provider.connection,
        user2.publicKey,
        10 * LAMPORTS_PER_SOL,
      );

      console.log("\tadmin:", admin.publicKey.toString());
      console.log("\tuser1:", user1.publicKey.toString());
      console.log("\tuser2:", user2.publicKey.toString());
    });

    describe("Create LST Token Mint", () => {
      it("Should create LST mint successfully", async () => {
        // create LST mint with 9 decimals
        lstMint = await createMint(
          provider.connection,
          admin, // payer
          admin.publicKey, // mint authority
          admin.publicKey, // freeze authority
          9, // decimals
        );
        console.log("\tLST Mint:", lstMint.toString());

        // derive pool PDA
        [poolPda, poolBump] = PublicKey.findProgramAddressSync(
          [Buffer.from("pool"), lstMint.toBuffer()],
          program.programId,
        );
        console.log("\tPool PDA:", poolPda.toString());

        // derive reserve PDA
        [reservePda, reserveBump] = PublicKey.findProgramAddressSync(
          [Buffer.from("pool-reserve"), poolPda.toBuffer()],
          program.programId,
        );
        console.log("\tReserve PDA:", reservePda.toString());

        // verify mint was created correctly
        const mintInfo = await getMint(provider.connection, lstMint);
        console.log("\tMint decimals:", mintInfo.decimals);
        console.log("\tMint authority:", mintInfo.mintAuthority?.toString());
        console.log("\tMint supply:", mintInfo.supply.toString());

        assert.equal(mintInfo.decimals, 9, "Decimals should be 9");
        assert.equal(
          mintInfo.mintAuthority?.toString(),
          admin.publicKey.toString(),
          "Mint authority should be admin",
        );
        assert.equal(
          mintInfo.supply.toString(),
          "0",
          "Initial supply should be 0",
        );
        assert.equal(
          mintInfo.isInitialized,
          true,
          "Mint should be initialized",
        );
      });
    });
  });

  describe("CREATE POOL", async () => {
    describe("Success case", async () => {
      it("initializes pool successfully", async () => {
        const tx = await program.methods
          .initializePool()
          .accounts({
            admin: admin.publicKey,
            pool: poolPda,
            reserveAccount: reservePda,
            lstMint: lstMint,
            systemProgram: anchor.web3.SystemProgram.programId,
            tokenProgram: TOKEN_PROGRAM_ID,
          })
          .signers([admin])
          .rpc({ commitment: "confirmed" });
        // console.log("✅ initializePool tx:", tx);

        // verify pool account
        const poolAccount = await program.account.pool.fetch(poolPda);
        // console.log("Pool admin:", poolAccount.admin.toString());
        // console.log(
        //   "Pool reserveAccount:",
        //   poolAccount.reserveAccount.toString(),
        // );
        // console.log("Pool lstMint:", poolAccount.lstMint.toString());
        // console.log("Pool totalStaked:", poolAccount.totalStaked.toString());
        // console.log(
        //   "Pool totalLstMinted:",
        //   poolAccount.totalLstMinted.toString(),
        // );
        // console.log("Pool stakedCount:", poolAccount.stakedCount.toString());
        // console.log(
        //   "Pool unstakedCount:",
        //   poolAccount.unstakedCount.toString(),
        // );
        // console.log("Pool lstDecimals:", poolAccount.lstDecimals);
        // console.log("Pool bump:", poolAccount.bump);
        // console.log("Pool reserveBump:", poolAccount.reserveBump);

        assert.equal(
          poolAccount.admin.toString(),
          admin.publicKey.toString(),
          "Admin should match",
        );
        assert.equal(
          poolAccount.reserveAccount.toString(),
          reservePda.toString(),
          "Reserve should match",
        );
        assert.equal(
          poolAccount.lstMint.toString(),
          lstMint.toString(),
          "LST mint should match",
        );
        assert.equal(
          poolAccount.totalStaked.toString(),
          "0",
          "Total staked should be 0",
        );
        assert.equal(
          poolAccount.totalLstMinted.toString(),
          "0",
          "Total LST minted should be 0",
        );
        assert.equal(
          poolAccount.stakedCount.toString(),
          "0",
          "Staked count should be 0",
        );
        assert.equal(
          poolAccount.unstakedCount.toString(),
          "0",
          "Unstaked count should be 0",
        );
        assert.equal(poolAccount.lstDecimals, 9, "Decimals should be 9");
        assert.equal(poolAccount.bump, poolBump, "Pool bump should match");

        // verify reserve account exists on chain
        const reserveBalance = await provider.connection.getBalance(reservePda);
        // console.log("Reserve balance:", reserveBalance);
        assert.ok(reserveBalance > 0, "Reserve should be rent exempt");
      });
    });

    describe("Failure case", () => {
      it("fails to initialize pool with same lstMint", async () => {
        try {
          await program.methods
            .initializePool()
            .accounts({
              admin: admin.publicKey,
              pool: poolPda,
              reserveAccount: reservePda,
              lstMint: lstMint,
              systemProgram: anchor.web3.SystemProgram.programId,
              tokenProgram: TOKEN_PROGRAM_ID,
            })
            .signers([admin])
            .rpc({ commitment: "confirmed" });
          assert.fail("Should have thrown an error");
        } catch (err: any) {
          // console.log("Expected error:", err.message);
          assert.ok(
            err.message.includes("already in use"),
            "Should fail because pool already exists",
          );
        }
      });

      it("fails to initialize pool with non admin signer", async () => {
        // create a different lst mint so PDA is different
        const anotherLstMint = await createMint(
          provider.connection,
          user1, // user1 pays
          user1.publicKey,
          user1.publicKey,
          9,
        );
        const [anotherPoolPda] = PublicKey.findProgramAddressSync(
          [Buffer.from("pool"), anotherLstMint.toBuffer()],
          program.programId,
        );
        const [anotherReservePda] = PublicKey.findProgramAddressSync(
          [Buffer.from("pool-reserve"), anotherPoolPda.toBuffer()],
          program.programId,
        );
        try {
          await program.methods
            .initializePool()
            .accounts({
              admin: user1.publicKey, // ← wrong admin
              pool: anotherPoolPda,
              reserveAccount: anotherReservePda,
              lstMint: anotherLstMint,
              systemProgram: anchor.web3.SystemProgram.programId,
              tokenProgram: TOKEN_PROGRAM_ID,
            })
            .signers([user2]) // ← user2 signs not admin
            .rpc({ commitment: "confirmed" });
          assert.fail("Should have thrown an error");
        } catch (err: any) {
          // console.log("Expected error:", err.message);
          assert.ok(
            err.message.includes("NotTheOwner") ||
              err.message.includes("unknown signer"),
            "Should fail because signer is not admin",
          );
        }
      });

      it("should fail if we call the initiliazePool with wrong pool seeds", async () => {
        try {
          const [wrongPoolPda] = PublicKey.findProgramAddressSync(
            [Buffer.from("wrong pool"), lstMint.toBuffer()],
            program.programId,
          );
          const [wronPoolReservePda] = PublicKey.findProgramAddressSync(
            [Buffer.from("pool-reserve"), wrongPoolPda.toBuffer()],
            program.programId,
          );
          await program.methods
            .initializePool()
            .accounts({
              admin: admin.publicKey,
              pool: wrongPoolPda,
              reserveAccount: wronPoolReservePda,
              lstMint: lstMint,
              systemProgram: anchor.web3.SystemProgram.programId,
              tokenProgram: TOKEN_PROGRAM_ID,
            })
            .signers([admin])
            .rpc({
              commitment: "confirmed",
            });
          assert.fail("Should have thrown an error");
        } catch (err: any) {
          // console.log("Expected error:", err.message);
          assert.ok(
            err.message.includes("ConstraintSeeds.") ||
              err.message.includes("A seeds constraint was violated"),
            "Should fail because of wrong pool pda",
          );
        }
      });
    });
  });

  describe.skip("CREATE USER ATA and DEPOSIT STAKE TO POOL", () => {
    describe("Create user ATA for user1 and user1", () => {});

    describe("Deposit and delegate to pool", () => {});
  });
});

async function airdrop(connection: any, address: any, amount = 1000000000) {
  await connection.confirmTransaction(
    await connection.requestAirdrop(address, amount),
    "confirmed",
  );
}

async function validatePoolEqual(
  program: anchor.Program<Stakely>,
  admin: PublicKey,
  user1: PublicKey,
  user2: PublicKey,
  error: String,
) {}
