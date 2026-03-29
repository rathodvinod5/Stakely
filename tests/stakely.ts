import * as anchor from "@coral-xyz/anchor";
import { Program } from "@coral-xyz/anchor";
import { LAMPORTS_PER_SOL, PublicKey } from "@solana/web3.js";
import {
  AuthorityType,
  createMint,
  getAccount,
  getMint,
  getOrCreateAssociatedTokenAccount,
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

  const admin = anchor.web3.Keypair.generate();
  const user1 = anchor.web3.Keypair.generate();
  const user2 = anchor.web3.Keypair.generate();

  // ── LST mint + pool ──────────────────────────────────────
  let lstMint: PublicKey;
  let poolPda: PublicKey;
  let poolBump: number;
  let reservePda: PublicKey;
  let reserveBump: number;

  // ── another pool (for failure cases / reuse) ─────────────
  let anotherLstMint: PublicKey;
  let anotherPoolPda: PublicKey;
  let anotherReservePda: PublicKey;

  // ── user ATAs ────────────────────────────────────────────
  let userAta1: PublicKey;
  let userAta2: PublicKey;

  // ── stake accounts ───────────────────────────────────────
  let stakeAccount1: anchor.web3.Keypair;
  let stakeAccount2: anchor.web3.Keypair;

  // ── stake entries ────────────────────────────────────────
  let stakeEntry1Pda: PublicKey;
  let stakeEntry2Pda: PublicKey;

  // ── unstake tickets ──────────────────────────────────────
  let unstakeTicket1Pda: PublicKey;
  let unstakeTicket2Pda: PublicKey;

  // ── fakeStakeAccount  ────────────────────────────────────
  // not owned by stake program
  let fakeStakeAccount: anchor.web3.Keypair;
  let fakeStakeEntryPda: PublicKey;

  // ── broken admin  ────────────────────────────────────────
  let brokeAdmin: anchor.web3.Keypair;

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
        anotherLstMint = await createMint(
          provider.connection,
          user1, // user1 pays
          user1.publicKey,
          user1.publicKey,
          9,
        );
        [anotherPoolPda] = PublicKey.findProgramAddressSync(
          [Buffer.from("pool"), anotherLstMint.toBuffer()],
          program.programId,
        );
        [anotherReservePda] = PublicKey.findProgramAddressSync(
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
        // derive WRONG reserve PDA using wrong seeds
        // correct seeds: [b"pool-reserve", poolPda]
        // wrong seeds:   [b"wrong-reserve", poolPda]
        const [wrongReservePda] = PublicKey.findProgramAddressSync(
          [Buffer.from("wrong-reserve"), poolPda.toBuffer()], // ← wrong seed prefix
          program.programId,
        );
        // console.log("Wrong Reserve PDA:", wrongReservePda.toString());

        try {
          await program.methods
            .initializePool()
            .accounts({
              admin: admin.publicKey,
              pool: poolPda,
              reserveAccount: wrongReservePda, // ← wrong reserve PDA
              lstMint: lstMint,
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

        // derive pool PDA with fake mint
        const [fakePoolPda] = PublicKey.findProgramAddressSync(
          [Buffer.from("pool"), fakeMint.publicKey.toBuffer()],
          program.programId,
        );
        const [fakeReservePda] = PublicKey.findProgramAddressSync(
          [Buffer.from("pool-reserve"), fakePoolPda.toBuffer()],
          program.programId,
        );

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
        brokeAdmin = anchor.web3.Keypair.generate();

        // airdrop just a tiny amount - not enough to pay for rent
        // pool rent ~ 1300 lamports
        // reserve rent ~ 890880 lamports
        // we give only 100 lamports which is not enough
        const sig = await provider.connection.requestAirdrop(
          brokeAdmin.publicKey,
          100, // ← only 100 lamports, not enough
        );
        await provider.connection.confirmTransaction(sig, "confirmed");

        const [freshPoolPda] = PublicKey.findProgramAddressSync(
          [Buffer.from("pool"), lstMint.toBuffer()],
          program.programId,
        );
        const [freshReservePda] = PublicKey.findProgramAddressSync(
          [Buffer.from("pool-reserve"), freshPoolPda.toBuffer()],
          program.programId,
        );

        try {
          await program.methods
            .initializePool()
            .accounts({
              admin: brokeAdmin.publicKey, // ← broke admin has no SOL
              pool: freshPoolPda,
              reserveAccount: freshReservePda,
              lstMint: lstMint,
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
            admin.publicKey, // wrong current authority (it was already transferred to poolPda)
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

  describe("CREATE USER ATA, STAKE ACCOUNT AND TRANSFER AUTHORITY TO POOL PDA", () => {
    describe("Create user ATA for user1 and user2", () => {
      it("creates ATA for user1 successfully", async () => {
        const ata = await getOrCreateAssociatedTokenAccount(
          provider.connection,
          user1, // payer
          lstMint, // mint
          user1.publicKey, // owner
        );
        userAta1 = ata.address;

        const ataInfo = await getAccount(provider.connection, userAta1);
        assert.equal(
          ataInfo.mint.toString(),
          lstMint.toString(),
          "Mint should match",
        );
        assert.equal(
          ataInfo.owner.toString(),
          user1.publicKey.toString(),
          "Owner should match",
        );
        assert.equal(
          ataInfo.amount.toString(),
          "0",
          "Initial balance should be 0",
        );
      });

      it("creates ATA for user2 successfully", async () => {
        const ata = await getOrCreateAssociatedTokenAccount(
          provider.connection,
          user2, // payer
          lstMint, // mint
          user2.publicKey, // owner
        );
        userAta2 = ata.address;

        const ataInfo = await getAccount(provider.connection, userAta2);
        assert.equal(
          ataInfo.mint.toString(),
          lstMint.toString(),
          "Mint should match",
        );
        assert.equal(
          ataInfo.owner.toString(),
          user2.publicKey.toString(),
          "Owner should match",
        );
        assert.equal(
          ataInfo.amount.toString(),
          "0",
          "Initial balance should be 0",
        );
      });
    });

    describe("Create stake account for user1 and user2", () => {
      it("creates and funds stake account for user1", async () => {
        stakeAccount1 = anchor.web3.Keypair.generate();

        const stakeAccountRent =
          await provider.connection.getMinimumBalanceForRentExemption(200);
        const stakeAmount = 2 * LAMPORTS_PER_SOL;

        const createStakeAccountTx = new anchor.web3.Transaction().add(
          // step 1: create the account
          anchor.web3.SystemProgram.createAccount({
            fromPubkey: user1.publicKey,
            newAccountPubkey: stakeAccount1.publicKey,
            lamports: stakeAmount + stakeAccountRent,
            space: 200,
            programId: anchor.web3.StakeProgram.programId,
          }),
          // step 2: initialize stake account
          anchor.web3.StakeProgram.initialize({
            stakePubkey: stakeAccount1.publicKey,
            authorized: new anchor.web3.Authorized(
              user1.publicKey, // staker authority
              user1.publicKey, // withdrawer authority
            ),
            lockup: new anchor.web3.Lockup(0, 0, user1.publicKey),
          }),
        );

        await provider.sendAndConfirm(
          createStakeAccountTx,
          [user1, stakeAccount1],
          {
            commitment: "confirmed",
          },
        );

        const stakeBalance = await provider.connection.getBalance(
          stakeAccount1.publicKey,
        );
        // console.log("Stake account1 balance:", stakeBalance);
        assert.ok(
          stakeBalance >= stakeAmount,
          "Stake account should be funded",
        );
      });

      it("creates and funds stake account for user2", async () => {
        stakeAccount2 = anchor.web3.Keypair.generate();

        const stakeAccountRent =
          await provider.connection.getMinimumBalanceForRentExemption(200);
        const stakeAmount = 3 * LAMPORTS_PER_SOL;

        const createStakeAccountTx = new anchor.web3.Transaction().add(
          anchor.web3.SystemProgram.createAccount({
            fromPubkey: user2.publicKey,
            newAccountPubkey: stakeAccount2.publicKey,
            lamports: stakeAmount + stakeAccountRent,
            space: 200,
            programId: anchor.web3.StakeProgram.programId,
          }),
          anchor.web3.StakeProgram.initialize({
            stakePubkey: stakeAccount2.publicKey,
            authorized: new anchor.web3.Authorized(
              user2.publicKey,
              user2.publicKey,
            ),
            lockup: new anchor.web3.Lockup(0, 0, user2.publicKey),
          }),
        );

        await provider.sendAndConfirm(
          createStakeAccountTx,
          [user2, stakeAccount2],
          {
            commitment: "confirmed",
          },
        );

        const stakeBalance = await provider.connection.getBalance(
          stakeAccount2.publicKey,
        );
        // console.log("Stake account2 balance:", stakeBalance);
        assert.ok(
          stakeBalance >= stakeAmount,
          "Stake account should be funded",
        );
      });
    });

    describe("Transfer stake authority to pool PDA", () => {
      it("transfers stake and withdraw authority to pool PDA for user1", async () => {
        const transferAuthTx = new anchor.web3.Transaction().add(
          // transfer staker authority to pool PDA
          anchor.web3.StakeProgram.authorize({
            stakePubkey: stakeAccount1.publicKey,
            authorizedPubkey: user1.publicKey, // current authority
            newAuthorizedPubkey: poolPda, // new authority = pool PDA
            stakeAuthorizationType: anchor.web3.StakeAuthorizationLayout.Staker,
          }),
          // transfer withdrawer authority to pool PDA
          anchor.web3.StakeProgram.authorize({
            stakePubkey: stakeAccount1.publicKey,
            authorizedPubkey: user1.publicKey, // current authority
            newAuthorizedPubkey: poolPda, // new authority = pool PDA
            stakeAuthorizationType:
              anchor.web3.StakeAuthorizationLayout.Withdrawer,
          }),
        );

        await provider.sendAndConfirm(transferAuthTx, [user1], {
          commitment: "confirmed",
        });

        // verify authority was transferred
        const stakeAccountInfo = await provider.connection.getAccountInfo(
          stakeAccount1.publicKey,
        );
        assert.equal(
          stakeAccountInfo?.owner.toString(),
          anchor.web3.StakeProgram.programId.toString(),
          "Stake account should be owned by stake program",
        );
      });

      it("transfers stake and withdraw authority to pool PDA for user2", async () => {
        const transferAuthTx = new anchor.web3.Transaction().add(
          anchor.web3.StakeProgram.authorize({
            stakePubkey: stakeAccount2.publicKey,
            authorizedPubkey: user2.publicKey,
            newAuthorizedPubkey: poolPda,
            stakeAuthorizationType: anchor.web3.StakeAuthorizationLayout.Staker,
          }),
          anchor.web3.StakeProgram.authorize({
            stakePubkey: stakeAccount2.publicKey,
            authorizedPubkey: user2.publicKey,
            newAuthorizedPubkey: poolPda,
            stakeAuthorizationType:
              anchor.web3.StakeAuthorizationLayout.Withdrawer,
          }),
        );

        await provider.sendAndConfirm(transferAuthTx, [user2], {
          commitment: "confirmed",
        });

        const stakeAccountInfo = await provider.connection.getAccountInfo(
          stakeAccount2.publicKey,
        );
        // console.log(
        //   "Stake account2 owner:",
        //   stakeAccountInfo?.owner.toString(),
        // );
        assert.equal(
          stakeAccountInfo?.owner.toString(),
          anchor.web3.StakeProgram.programId.toString(),
          "Stake account should be owned by stake program",
        );
      });

      it("fails to transfer authority with wrong current authority", async () => {
        // user1 tries to transfer authority of user2 stake account
        try {
          const transferAuthTx = new anchor.web3.Transaction().add(
            anchor.web3.StakeProgram.authorize({
              stakePubkey: stakeAccount2.publicKey,
              authorizedPubkey: user1.publicKey, // ← wrong authority
              newAuthorizedPubkey: poolPda,
              stakeAuthorizationType:
                anchor.web3.StakeAuthorizationLayout.Staker,
            }),
          );

          await provider.sendAndConfirm(transferAuthTx, [user1], {
            commitment: "confirmed",
          });

          assert.fail("Should have thrown an error");
        } catch (err: any) {
          // console.log("Expected error caught:", err.message);
          assert.ok(
            err.message.includes("custom program error") ||
              err.message.includes("Error"),
            "Should fail because user1 is not the authority of stake account2",
          );
        }
      });
    });
  });

  describe("DEPOSIT AND DELEGATE", () => {
    describe("Success cases", () => {
      it("deposits and delegates stake to pool successfully for user1", async () => {
        const stakeAmount = new anchor.BN(2 * LAMPORTS_PER_SOL);

        // derive stake entry PDA for user1
        [stakeEntry1Pda] = PublicKey.findProgramAddressSync(
          [
            Buffer.from("stake-entry"),
            poolPda.toBuffer(),
            user1.publicKey.toBuffer(),
          ],
          program.programId,
        );

        // fetch pool state before deposit
        const poolBefore = await program.account.pool.fetch(poolPda);
        // console.log(
        //   "Pool totalStaked before:",
        //   poolBefore.totalStaked.toString(),
        // );
        // console.log(
        //   "Pool totalLstMinted before:",
        //   poolBefore.totalLstMinted.toString(),
        // );
        // console.log(
        //   "Pool stakedCount before:",
        //   poolBefore.stakedCount.toString(),
        // );

        // fetch user1 ATA balance before deposit
        const ataBefore = await getAccount(provider.connection, userAta1);

        const tx = await program.methods
          .depositAndDelegate(stakeAmount)
          .accounts({
            user: user1.publicKey,
            pool: poolPda,
            reserveAccount: reservePda,
            stakeAccount: stakeAccount1.publicKey,
            lstMint: lstMint,
            userAta: userAta1,
            stakeEntry: stakeEntry1Pda,
            systemProgram: anchor.web3.SystemProgram.programId,
            tokenProgram: TOKEN_PROGRAM_ID,
            rent: anchor.web3.SYSVAR_RENT_PUBKEY,
          })
          .signers([user1])
          .rpc({ commitment: "confirmed" });

        // verify pool state after deposit
        const poolAfter = await program.account.pool.fetch(poolPda);

        assert.equal(
          poolAfter.totalStaked.toString(),
          stakeAmount.toString(),
          "Total staked should increase by stake amount",
        );
        assert.ok(
          poolAfter.totalLstMinted.toString() !== "0",
          "Total LST minted should increase",
        );
        assert.equal(
          poolAfter.stakedCount.toString(),
          "1",
          "Staked count should be 1",
        );

        // verify stake entry was created correctly
        const stakeEntry = await program.account.stakeEntry.fetch(
          stakeEntry1Pda,
        );

        assert.equal(
          stakeEntry.pool.toString(),
          poolPda.toString(),
          "Pool should match",
        );
        assert.equal(
          stakeEntry.stakeAccount.toString(),
          stakeAccount1.publicKey.toString(),
          "Stake account should match",
        );
        assert.equal(
          stakeEntry.depositedLamports.toString(),
          stakeAmount.toString(),
          "Deposited lamports should match",
        );
        assert.deepEqual(
          stakeEntry.stakeStatus,
          { active: {} },
          "Stake status should be active",
        );
        assert.equal(stakeEntry.index.toString(), "0", "Index should be 0");

        // verify user1 received LST tokens
        const ataAfter = await getAccount(provider.connection, userAta1);
        assert.ok(
          BigInt(ataAfter.amount) > BigInt(ataBefore.amount),
          "User1 should have received LST tokens",
        );
      });

      it("deposits and delegates stake to pool successfully for user2", async () => {
        const stakeAmount = new anchor.BN(3 * LAMPORTS_PER_SOL);

        // derive stake entry PDA for user2
        [stakeEntry2Pda] = PublicKey.findProgramAddressSync(
          [
            Buffer.from("stake-entry"),
            poolPda.toBuffer(),
            user2.publicKey.toBuffer(),
          ],
          program.programId,
        );

        // fetch pool state before deposit
        // const poolBefore = await program.account.pool.fetch(poolPda);
        // console.log(
        //   "Pool totalStaked before:",
        //   poolBefore.totalStaked.toString(),
        // );
        // console.log(
        //   "Pool totalLstMinted before:",
        //   poolBefore.totalLstMinted.toString(),
        // );
        // console.log(
        //   "Pool stakedCount before:",
        //   poolBefore.stakedCount.toString(),
        // );

        const ataBefore = await getAccount(provider.connection, userAta2);

        const tx = await program.methods
          .depositAndDelegate(stakeAmount)
          .accounts({
            user: user2.publicKey,
            pool: poolPda,
            reserveAccount: reservePda,
            stakeAccount: stakeAccount2.publicKey,
            lstMint: lstMint,
            userAta: userAta2,
            stakeEntry: stakeEntry2Pda,
            systemProgram: anchor.web3.SystemProgram.programId,
            tokenProgram: TOKEN_PROGRAM_ID,
            rent: anchor.web3.SYSVAR_RENT_PUBKEY,
          })
          .signers([user2])
          .rpc({ commitment: "confirmed" });

        // verify pool state after deposit
        const poolAfter = await program.account.pool.fetch(poolPda);
        // console.log(
        //   "Pool totalStaked after:",
        //   poolAfter.totalStaked.toString(),
        // );
        // console.log(
        //   "Pool totalLstMinted after:",
        //   poolAfter.totalLstMinted.toString(),
        // );
        // console.log(
        //   "Pool stakedCount after:",
        //   poolAfter.stakedCount.toString(),
        // );

        // total staked should now be user1 + user2 stake amounts
        const expectedTotalStaked = new anchor.BN(2 * LAMPORTS_PER_SOL).add(
          stakeAmount,
        );
        assert.equal(
          poolAfter.totalStaked.toString(),
          expectedTotalStaked.toString(),
          "Total staked should be sum of user1 and user2 stake amounts",
        );
        assert.equal(
          poolAfter.stakedCount.toString(),
          "2",
          "Staked count should be 2",
        );

        // verify stake entry for user2
        const stakeEntry = await program.account.stakeEntry.fetch(
          stakeEntry2Pda,
        );
        // console.log(
        //   "Stake entry2 depositedLamports:",
        //   stakeEntry.depositedLamports.toString(),
        // );
        // console.log("Stake entry2 index:", stakeEntry.index.toString());

        assert.equal(
          stakeEntry.depositedLamports.toString(),
          stakeAmount.toString(),
          "Deposited lamports should match",
        );
        assert.deepEqual(
          stakeEntry.stakeStatus,
          { active: {} },
          "Stake status should be active",
        );
        assert.equal(stakeEntry.index.toString(), "1", "Index should be 1");

        // verify user2 received LST tokens
        const ataAfter = await getAccount(provider.connection, userAta2);
        // console.log("User2 LST balance after:", ataAfter.amount.toString());
        assert.ok(
          BigInt(ataAfter.amount) > BigInt(ataBefore.amount),
          "User2 should have received LST tokens",
        );
      });
    });

    describe("Failure cases", () => {
      it("fails to deposit with zero stake amount", async () => {
        try {
          await program.methods
            .depositAndDelegate(new anchor.BN(0)) // ← zero amount
            .accounts({
              user: user1.publicKey,
              pool: poolPda,
              reserveAccount: reservePda,
              stakeAccount: stakeAccount1.publicKey,
              lstMint: lstMint,
              userAta: userAta1,
              stakeEntry: stakeEntry1Pda,
              systemProgram: anchor.web3.SystemProgram.programId,
              tokenProgram: TOKEN_PROGRAM_ID,
              rent: anchor.web3.SYSVAR_RENT_PUBKEY,
            })
            .signers([user1])
            .rpc({ commitment: "confirmed" });

          assert.fail("Should have thrown an error");
        } catch (err: any) {
          // console.log("Expected error caught:", err.message);
          assert.ok(
            err.message.includes("InsufficientBalance") ||
              err.message.includes("Error"),
            "Should fail because stake amount is 0",
          );
        }
      });

      it("fails to deposit with wrong pool", async () => {
        // initialize another pool
        await program.methods
          .initializePool()
          .accounts({
            admin: admin.publicKey,
            pool: anotherPoolPda,
            reserveAccount: anotherReservePda,
            lstMint: anotherLstMint,
            systemProgram: anchor.web3.SystemProgram.programId,
            tokenProgram: TOKEN_PROGRAM_ID,
          })
          .signers([admin])
          .rpc({ commitment: "confirmed" });

        const [wrongStakeEntryPda] = PublicKey.findProgramAddressSync(
          [
            Buffer.from("stake-entry"),
            anotherPoolPda.toBuffer(), // ← wrong pool
            user1.publicKey.toBuffer(),
          ],
          program.programId,
        );

        try {
          await program.methods
            .depositAndDelegate(new anchor.BN(2 * LAMPORTS_PER_SOL))
            .accounts({
              user: user1.publicKey,
              pool: anotherPoolPda, // ← wrong pool
              reserveAccount: anotherReservePda,
              stakeAccount: stakeAccount1.publicKey,
              lstMint: anotherLstMint, // ← wrong mint
              userAta: userAta1,
              stakeEntry: wrongStakeEntryPda,
              systemProgram: anchor.web3.SystemProgram.programId,
              tokenProgram: TOKEN_PROGRAM_ID,
              rent: anchor.web3.SYSVAR_RENT_PUBKEY,
            })
            .signers([user1])
            .rpc({ commitment: "confirmed" });

          assert.fail("Should have thrown an error");
        } catch (err: any) {
          // console.log("Expected error caught:", err.message);
          assert.ok(
            err.message.includes("Error"),
            "Should fail because pool is wrong",
          );
        }
      });

      it("fails to deposit with stake account not owned by stake program", async () => {
        // create a regular system account instead of stake account
        fakeStakeAccount = anchor.web3.Keypair.generate();
        const createFakeAccountTx = new anchor.web3.Transaction().add(
          anchor.web3.SystemProgram.createAccount({
            fromPubkey: user1.publicKey,
            newAccountPubkey: fakeStakeAccount.publicKey,
            lamports: 2 * LAMPORTS_PER_SOL,
            space: 0,
            programId: anchor.web3.SystemProgram.programId, // ← system program not stake program
          }),
        );
        await provider.sendAndConfirm(
          createFakeAccountTx,
          [user1, fakeStakeAccount],
          {
            commitment: "confirmed",
          },
        );

        [fakeStakeEntryPda] = PublicKey.findProgramAddressSync(
          [
            Buffer.from("stake-entry"),
            poolPda.toBuffer(),
            fakeStakeAccount.publicKey.toBuffer(),
          ],
          program.programId,
        );

        try {
          await program.methods
            .depositAndDelegate(new anchor.BN(2 * LAMPORTS_PER_SOL))
            .accounts({
              user: user1.publicKey,
              pool: poolPda,
              reserveAccount: reservePda,
              stakeAccount: fakeStakeAccount.publicKey, // ← not a real stake account
              lstMint: lstMint,
              userAta: userAta1,
              stakeEntry: fakeStakeEntryPda,
              systemProgram: anchor.web3.SystemProgram.programId,
              tokenProgram: TOKEN_PROGRAM_ID,
              rent: anchor.web3.SYSVAR_RENT_PUBKEY,
            })
            .signers([user1])
            .rpc({ commitment: "confirmed" });

          assert.fail("Should have thrown an error");
        } catch (err: any) {
          // console.log("Expected error caught:", err.message);
          assert.ok(
            err.message.includes("NotTheOwner") ||
              err.message.includes("Error"),
            "Should fail because stake account is not owned by stake program",
          );
        }
      });

      it("fails to deposit same stake account twice", async () => {
        // stakeAccount1 already deposited in success case
        // stake entry PDA already exists for user1
        try {
          await program.methods
            .depositAndDelegate(new anchor.BN(2 * LAMPORTS_PER_SOL))
            .accounts({
              user: user1.publicKey,
              pool: poolPda,
              reserveAccount: reservePda,
              stakeAccount: stakeAccount1.publicKey, // ← already deposited
              lstMint: lstMint,
              userAta: userAta1,
              stakeEntry: stakeEntry1Pda, // ← already exists
              systemProgram: anchor.web3.SystemProgram.programId,
              tokenProgram: TOKEN_PROGRAM_ID,
              rent: anchor.web3.SYSVAR_RENT_PUBKEY,
            })
            .signers([user1])
            .rpc({ commitment: "confirmed" });

          assert.fail("Should have thrown an error");
        } catch (err: any) {
          // console.log("Expected error caught:", err.message);
          assert.ok(
            err.message.includes("already in use") ||
              err.message.includes("Error"),
            "Should fail because stake entry already exists",
          );
        }
      });
    });
  });

  describe("MOCK ACCRUE REWARDS", () => {
    describe("Success cases", () => {
      it("accrues rewards to reserve account successfully", async () => {
        const rewardAmount = new anchor.BN(5 * LAMPORTS_PER_SOL);

        // fetch pool state before accruing rewards
        const poolBefore = await program.account.pool.fetch(poolPda);
        // console.log(
        //   "Pool totalStaked before:",
        //   poolBefore.totalStaked.toString(),
        // );

        // fetch reserve balance before
        const reserveBalanceBefore = await provider.connection.getBalance(
          reservePda,
        );
        // console.log("Reserve balance before:", reserveBalanceBefore);

        // fetch admin balance before
        const adminBalanceBefore = await provider.connection.getBalance(
          admin.publicKey,
        );
        // console.log("Admin balance before:", adminBalanceBefore);

        const tx = await program.methods
          .mockAccrueRewards(rewardAmount)
          .accounts({
            admin: admin.publicKey,
            pool: poolPda,
            reserveAccount: reservePda,
            systemProgram: anchor.web3.SystemProgram.programId,
          })
          .signers([admin])
          .rpc({ commitment: "confirmed" });

        // verify pool totalStaked increased by reward amount
        const poolAfter = await program.account.pool.fetch(poolPda);
        // console.log(
        //   "Pool totalStaked after:",
        //   poolAfter.totalStaked.toString(),
        // );

        const expectedTotalStaked = new anchor.BN(
          poolBefore.totalStaked.toString(),
        ).add(rewardAmount);
        // console.log("expected: ", expectedTotalStaked.toString());
        assert.equal(
          poolAfter.totalStaked.toString(),
          expectedTotalStaked.toString(),
          "Total staked should increase by reward amount",
        );

        // verify reserve balance increased by reward amount
        const reserveBalanceAfter = await provider.connection.getBalance(
          reservePda,
        );
        // console.log("Reserve balance after:", reserveBalanceAfter);
        assert.equal(
          reserveBalanceAfter,
          reserveBalanceBefore + rewardAmount.toNumber(),
          "Reserve balance should increase by reward amount",
        );

        // verify admin balance decreased by reward amount
        const adminBalanceAfter = await provider.connection.getBalance(
          admin.publicKey,
        );
        // console.log("Admin balance after:", adminBalanceAfter);
        assert.ok(
          adminBalanceAfter < adminBalanceBefore,
          "Admin balance should decrease by reward amount",
        );
      });

      it("accrues rewards multiple times successfully", async () => {
        const rewardAmount = new anchor.BN(0.5 * LAMPORTS_PER_SOL);

        const poolBefore = await program.account.pool.fetch(poolPda);
        const reserveBalanceBefore = await provider.connection.getBalance(
          reservePda,
        );

        // accrue rewards twice
        for (let i = 0; i < 2; i++) {
          const tx = await program.methods
            .mockAccrueRewards(rewardAmount)
            .accounts({
              admin: admin.publicKey,
              pool: poolPda,
              reserveAccount: reservePda,
              systemProgram: anchor.web3.SystemProgram.programId,
            })
            .signers([admin])
            .rpc({ commitment: "confirmed" });
          // console.log(`✅ mockAccrueRewards tx ${i + 1}:`, tx);
        }

        // verify pool totalStaked increased by 2x reward amount
        const poolAfter = await program.account.pool.fetch(poolPda);
        const expectedTotalStaked = new anchor.BN(
          poolBefore.totalStaked.toString(),
        ).add(rewardAmount.mul(new anchor.BN(2)));
        // console.log(
        //   "Pool totalStaked after 2x rewards:",
        //   poolAfter.totalStaked.toString(),
        // );

        assert.equal(
          poolAfter.totalStaked.toString(),
          expectedTotalStaked.toString(),
          "Total staked should increase by 2x reward amount",
        );

        // verify reserve balance increased by 2x reward amount
        const reserveBalanceAfter = await provider.connection.getBalance(
          reservePda,
        );
        assert.equal(
          reserveBalanceAfter,
          reserveBalanceBefore + rewardAmount.toNumber() * 2,
          "Reserve balance should increase by 2x reward amount",
        );
      });
    });

    describe("Failure cases", () => {
      it("fails to accrue rewards with wrong admin", async () => {
        const rewardAmount = new anchor.BN(1 * LAMPORTS_PER_SOL);

        try {
          await program.methods
            .mockAccrueRewards(rewardAmount)
            .accounts({
              admin: user1.publicKey, // ← wrong admin
              pool: poolPda,
              reserveAccount: reservePda,
              systemProgram: anchor.web3.SystemProgram.programId,
            })
            .signers([user1]) // ← user1 signs not admin
            .rpc({ commitment: "confirmed" });

          assert.fail("Should have thrown an error");
        } catch (err: any) {
          // console.log("Expected error caught:", err.message);
          assert.ok(
            err.message.includes("NotTheOwner") ||
              err.message.includes("Error"),
            "Should fail because user1 is not the admin",
          );
        }
      });

      it("fails to accrue rewards with zero reward amount", async () => {
        try {
          await program.methods
            .mockAccrueRewards(new anchor.BN(0)) // ← zero amount
            .accounts({
              admin: admin.publicKey,
              pool: poolPda,
              reserveAccount: reservePda,
              systemProgram: anchor.web3.SystemProgram.programId,
            })
            .signers([admin])
            .rpc({ commitment: "confirmed" });

          assert.fail("Should have thrown an error");
        } catch (err: any) {
          // console.log("Expected error caught:", err.message);
          assert.ok(
            err.message.includes("InsufficientBalance") ||
              err.message.includes("error"),
            "Should fail because reward amount is 0",
          );
        }
      });

      it("fails to accrue rewards with wrong reserve account", async () => {
        const rewardAmount = new anchor.BN(1 * LAMPORTS_PER_SOL);

        try {
          await program.methods
            .mockAccrueRewards(rewardAmount)
            .accounts({
              admin: admin.publicKey,
              pool: poolPda,
              reserveAccount: anotherReservePda, // ← wrong reserve account
              systemProgram: anchor.web3.SystemProgram.programId,
            })
            .signers([admin])
            .rpc({ commitment: "confirmed" });

          assert.fail("Should have thrown an error");
        } catch (err: any) {
          // console.log("Expected error caught:", err.message);
          assert.ok(
            err.message.includes("has_one") ||
              err.message.includes("ConstraintHasOne") ||
              err.message.includes("Error"),
            "Should fail because reserve account does not match pool",
          );
        }
      });

      it("fails to accrue rewards with insufficient admin balance", async () => {
        // create a fresh pool for broke admin
        const freshLstMint = await createMint(
          provider.connection,
          admin, // ← admin pays for mint
          brokeAdmin.publicKey,
          brokeAdmin.publicKey,
          9,
        );
        const [freshPoolPda] = PublicKey.findProgramAddressSync(
          [Buffer.from("pool"), freshLstMint.toBuffer()],
          program.programId,
        );
        const [freshReservePda] = PublicKey.findProgramAddressSync(
          [Buffer.from("pool-reserve"), freshPoolPda.toBuffer()],
          program.programId,
        );

        // initialize fresh pool with admin paying
        await program.methods
          .initializePool()
          .accounts({
            admin: admin.publicKey,
            pool: freshPoolPda,
            reserveAccount: freshReservePda,
            lstMint: freshLstMint,
            systemProgram: anchor.web3.SystemProgram.programId,
            tokenProgram: TOKEN_PROGRAM_ID,
          })
          .signers([admin])
          .rpc({ commitment: "confirmed" });

        try {
          await program.methods
            .mockAccrueRewards(new anchor.BN(1 * LAMPORTS_PER_SOL))
            .accounts({
              admin: brokeAdmin.publicKey, // ← broke admin
              pool: freshPoolPda,
              reserveAccount: freshReservePda,
              systemProgram: anchor.web3.SystemProgram.programId,
            })
            .signers([brokeAdmin])
            .rpc({ commitment: "confirmed" });

          assert.fail("Should have thrown an error");
        } catch (err: any) {
          assert.ok(
            err.message.includes("insufficient lamports") ||
              err.message.includes("NotTheOwner") ||
              err.message.includes("error"),
            "Should fail because broke admin has insufficient SOL",
          );
        }
      });
    });
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
