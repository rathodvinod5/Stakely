import * as anchor from "@coral-xyz/anchor";
import { Program } from "@coral-xyz/anchor";
import { LAMPORTS_PER_SOL, PublicKey } from "@solana/web3.js";
import {
  AuthorityType,
  createMint,
  getMint,
  setAuthority,
  TOKEN_PROGRAM_ID,
} from "@solana/spl-token";
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

      it("should fail if we call initiliazePool with wrong pool seeds", async () => {
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

      it("fails to initialize pool with wrong reserve PDA seeds", async () => {
        // create a fresh lstMint so poolPda is different
        const freshLstMint = await createMint(
          provider.connection,
          admin,
          admin.publicKey,
          admin.publicKey,
          9,
        );
        // console.log("Fresh LST Mint:", freshLstMint.toString());

        // derive correct pool PDA
        const [freshPoolPda] = PublicKey.findProgramAddressSync(
          [Buffer.from("pool"), freshLstMint.toBuffer()],
          program.programId,
        );
        // console.log("Fresh Pool PDA:", freshPoolPda.toString());

        // derive WRONG reserve PDA using wrong seeds
        // correct seeds: [b"pool-reserve", poolPda]
        // wrong seeds:   [b"wrong-reserve", poolPda]
        const [wrongReservePda] = PublicKey.findProgramAddressSync(
          [Buffer.from("wrong-reserve"), freshPoolPda.toBuffer()], // ← wrong seed prefix
          program.programId,
        );
        // console.log("Wrong Reserve PDA:", wrongReservePda.toString());

        try {
          await program.methods
            .initializePool()
            .accounts({
              admin: admin.publicKey,
              pool: freshPoolPda,
              reserveAccount: wrongReservePda, // ← wrong reserve PDA
              lstMint: freshLstMint,
              systemProgram: anchor.web3.SystemProgram.programId,
              tokenProgram: TOKEN_PROGRAM_ID,
            })
            .signers([admin])
            .rpc({ commitment: "confirmed" });

          assert.fail("Should have thrown an error");
        } catch (err: any) {
          // console.log("Expected error caught:", err.message);
          assert.ok(
            err.message.includes("seeds constraint was violated") ||
              err.message.includes("ConstraintSeeds") ||
              err.message.includes("Error"),
            "Should fail because reserve PDA seeds are wrong",
          );
        }
      });

      it("fails to initialize pool with invalid lst mint", async () => {
        // create a random keypair to use as fake mint
        const fakeMint = anchor.web3.Keypair.generate();
        // console.log("Fake mint:", fakeMint.publicKey.toString());

        // derive pool PDA with fake mint
        const [fakePoolPda] = PublicKey.findProgramAddressSync(
          [Buffer.from("pool"), fakeMint.publicKey.toBuffer()],
          program.programId,
        );
        const [fakeReservePda] = PublicKey.findProgramAddressSync(
          [Buffer.from("pool-reserve"), fakePoolPda.toBuffer()],
          program.programId,
        );
        // console.log("Fake Pool PDA:", fakePoolPda.toString());
        // console.log("Fake Reserve PDA:", fakeReservePda.toString());

        try {
          await program.methods
            .initializePool()
            .accounts({
              admin: admin.publicKey,
              pool: fakePoolPda,
              reserveAccount: fakeReservePda,
              lstMint: fakeMint.publicKey, // ← not a real mint account
              systemProgram: anchor.web3.SystemProgram.programId,
              tokenProgram: TOKEN_PROGRAM_ID,
            })
            .signers([admin])
            .rpc({ commitment: "confirmed" });

          assert.fail("Should have thrown an error");
        } catch (err: any) {
          // console.log("Expected error caught:", err.message);
          assert.ok(
            err.message.includes("AccountNotInitialized") ||
              err.message.includes("AccountOwnedByWrongProgram") ||
              err.message.includes("Error"),
            "Should fail because lstMint is not a real mint account",
          );
        }
      });

      it("fails to initialize pool with insufficient admin SOL", async () => {
        // create a broke keypair with no SOL
        const brokeAdmin = anchor.web3.Keypair.generate();
        // console.log("Broke admin:", brokeAdmin.publicKey.toString());

        // airdrop just a tiny amount - not enough to pay for rent
        // pool rent ~ 1300 lamports
        // reserve rent ~ 890880 lamports
        // we give only 100 lamports which is not enough
        const sig = await provider.connection.requestAirdrop(
          brokeAdmin.publicKey,
          100, // ← only 100 lamports, not enough
        );
        await provider.connection.confirmTransaction(sig, "confirmed");

        const brokeAdminBalance = await provider.connection.getBalance(
          brokeAdmin.publicKey,
        );
        // console.log("Broke admin balance:", brokeAdminBalance, "lamports");

        // create a fresh lstMint so PDA is different
        // brokeAdmin can't even afford this, so admin pays for it
        const freshLstMint = await createMint(
          provider.connection,
          admin, // ← admin pays for mint creation
          admin.publicKey,
          admin.publicKey,
          9,
        );
        // console.log("Fresh LST Mint:", freshLstMint.toString());

        const [freshPoolPda] = PublicKey.findProgramAddressSync(
          [Buffer.from("pool"), freshLstMint.toBuffer()],
          program.programId,
        );
        const [freshReservePda] = PublicKey.findProgramAddressSync(
          [Buffer.from("pool-reserve"), freshPoolPda.toBuffer()],
          program.programId,
        );
        // console.log("Fresh Pool PDA:", freshPoolPda.toString());
        // console.log("Fresh Reserve PDA:", freshReservePda.toString());

        try {
          await program.methods
            .initializePool()
            .accounts({
              admin: brokeAdmin.publicKey, // ← broke admin has no SOL
              pool: freshPoolPda,
              reserveAccount: freshReservePda,
              lstMint: freshLstMint,
              systemProgram: anchor.web3.SystemProgram.programId,
              tokenProgram: TOKEN_PROGRAM_ID,
            })
            .signers([brokeAdmin]) // ← broke admin signs
            .rpc({ commitment: "confirmed" });

          assert.fail("Should have thrown an error");
        } catch (err: any) {
          // console.log("Expected error caught:", err.message);
          assert.ok(
            err.message.includes("insufficient lamports") ||
              err.message.includes("Insufficient") ||
              err.message.includes("Error"),
            "Should fail because admin has insufficient SOL",
          );
        }
      });
    });
  });

  describe("CHANGE LST MINT AND FREEZE AUTHORITY TO POOL PDA", () => {
    describe("Success case", () => {
      it("changes mint and freeze authority to pool PDA successfully", async () => {
        // change mint authority from admin to pool PDA
        await setAuthority(
          provider.connection,
          admin, // payer
          lstMint, // mint account
          admin.publicKey, // current authority
          AuthorityType.MintTokens, // authority type
          poolPda, // new authority
        );

        // change freeze authority from admin to pool PDA
        await setAuthority(
          provider.connection,
          admin, // payer
          lstMint, // mint account
          admin.publicKey, // current authority
          AuthorityType.FreezeAccount, // authority type
          poolPda, // new authority
        );

        // verify both authorities changed
        const mintInfo = await getMint(provider.connection, lstMint);
        // console.log("Mint authority:", mintInfo.mintAuthority?.toString());
        // console.log("Freeze authority:", mintInfo.freezeAuthority?.toString());

        assert.equal(
          mintInfo.mintAuthority?.toString(),
          poolPda.toString(),
          "Mint authority should be pool PDA",
        );
        assert.equal(
          mintInfo.freezeAuthority?.toString(),
          poolPda.toString(),
          "Freeze authority should be pool PDA",
        );
      });
    });

    describe("Failure cases", () => {
      it("fails to change mint authority with wrong current authority", async () => {
        // user1 tries to change mint authority but is not the current authority
        try {
          await setAuthority(
            provider.connection,
            user1, // payer
            lstMint, // mint account
            user1.publicKey, // wrong current authority
            AuthorityType.MintTokens, // authority type
            user1.publicKey, // new authority
          );

          assert.fail("Should have thrown an error");
        } catch (err: any) {
          // console.log("Expected error caught:", err.message);
          assert.ok(
            err.message.includes("owner does not match") ||
              err.message.includes("InvalidAccountData") ||
              err.message.includes("Error"),
            "Should fail because user1 is not the current mint authority",
          );
        }
      });

      it("fails to change freeze authority with wrong current authority", async () => {
        // user2 tries to change freeze authority but is not the current authority
        try {
          await setAuthority(
            provider.connection,
            user2, // payer
            lstMint, // mint account
            user2.publicKey, // wrong current authority
            AuthorityType.FreezeAccount, // authority type
            user2.publicKey, // new authority
          );

          assert.fail("Should have thrown an error");
        } catch (err: any) {
          // console.log("Expected error caught:", err.message);
          assert.ok(
            err.message.includes("owner does not match") ||
              err.message.includes("InvalidAccountData") ||
              err.message.includes("Error"),
            "Should fail because user2 is not the current freeze authority",
          );
        }
      });

      it("fails to change mint authority after it is already transferred to pool PDA", async () => {
        // admin tries to change mint authority again
        // but authority is now pool PDA not admin anymore
        try {
          await setAuthority(
            provider.connection,
            admin, // payer
            lstMint, // mint account
            admin.publicKey, // wrong current authority (was transferred to poolPda)
            AuthorityType.MintTokens, // authority type
            admin.publicKey, // new authority
          );

          assert.fail("Should have thrown an error");
        } catch (err: any) {
          // console.log("Expected error caught:", err.message);
          assert.ok(
            err.message.includes("owner does not match") ||
              err.message.includes("InvalidAccountData") ||
              err.message.includes("Error"),
            "Should fail because mint authority is now pool PDA not admin",
          );
        }
      });
    });
  });

  describe.skip("CREATE USER ATA TO DEPOSIT STAKE TO POOL", () => {
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
