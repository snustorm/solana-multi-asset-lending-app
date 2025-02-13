import { describe, it } from 'node:test';
// @ts-ignore
import IDL from '../target/idl/lending2.json';
import { Lending2 } from '../target/types/lending2';
import { BanksClient, ProgramTestContext, startAnchor } from 'solana-bankrun';
import { Connection, LAMPORTS_PER_SOL, PublicKey } from '@solana/web3.js';
import { BankrunProvider } from 'anchor-bankrun';
import { PythSolanaReceiver } from '@pythnetwork/pyth-solana-receiver';
import { BankrunContextWrapper } from '../bankrun-utils/bankrunConnection';
import { BN, Program, Wallet } from '@coral-xyz/anchor';
import { Keypair } from '@solana/web3.js';
import { createMint, mintTo, createAccount, createAssociatedTokenAccount} from 'spl-token-bankrun';
import { AccountLayout, getAssociatedTokenAddress, TOKEN_PROGRAM_ID } from '@solana/spl-token';

const {DisplayUserTokenAccount, DisplayBankData, DisplayUser } = require("./fetchAccountData");


function sleep(ms: number) {
    return new Promise(resolve => setTimeout(resolve, ms));
}


describe('Lending Smart Contract Tests', async () => {
    let signer: Keypair;
    
    let usdcBankAccount: PublicKey;
    let usdcBankTokenAccount: PublicKey;
    let solBankAccount: PublicKey;
    let solBankTokenAccount: PublicKey;
  
    //let solTokenAccount: PublicKey;
    let provider: BankrunProvider;
    let program: Program<Lending2>;
    let banksClient: BanksClient;
    let context: ProgramTestContext;
    let bankrunContextWrapper: BankrunContextWrapper;

    console.log("start~");
  
    const pyth = new PublicKey('7UVimffxr9ow1uXYxsr4LHAcV58mLzhmwaeKvJ1pjLiE');
  
    const devnetConnection = new Connection('https://api.devnet.solana.com');
    console.log("connection~");
    const accountInfo = await devnetConnection.getAccountInfo(pyth);
    console.log("start~");
    context = await startAnchor(
      '',
      [{ name: 'lending2', programId: new PublicKey(IDL.address) }],
      [
        {
          address: pyth,
          info: accountInfo,
        },
      ]
    );
    provider = new BankrunProvider(context);
  
    bankrunContextWrapper = new BankrunContextWrapper(context);
  
    const connection = bankrunContextWrapper.connection.toConnection();
  
    const pythSolanaReceiver = new PythSolanaReceiver({
      connection,
      wallet: provider.wallet,
    });
  
    // Define SOL and USDC Price Feed IDs
    const SOL_PRICE_FEED_ID =
      '0xef0d8b6fda2ceba41da15d4095d1da392a0d2f8ed0c6c7bc0f4cfac8c280b56d';

    const USDC_PRICE_FEED_ID =
      '0xeaa020c61cc479712813461ce153894a96a6c00b21ed0cfc2798d1f9a9e9c94a';
  
    // Fetch the SOL and USDC price feed accounts
    const solUsdPriceFeedAccount = pythSolanaReceiver
      .getPriceFeedAccountAddress(0, SOL_PRICE_FEED_ID)
      .toBase58();
  
    const usdcUsdPriceFeedAccount = pythSolanaReceiver
      .getPriceFeedAccountAddress(0, USDC_PRICE_FEED_ID)
      .toBase58();

    const solUsdPriceFeedAccountPubkey = new PublicKey(solUsdPriceFeedAccount);
    const usdcUsdPriceFeedAccountPubkey = new PublicKey(usdcUsdPriceFeedAccount);

    // Get account info for both price feeds
    const solFeedAccountInfo = await devnetConnection.getAccountInfo(
      solUsdPriceFeedAccountPubkey
    );
    const usdcFeedAccountInfo = await devnetConnection.getAccountInfo(
      usdcUsdPriceFeedAccountPubkey
    );
  
    context.setAccount(solUsdPriceFeedAccountPubkey, solFeedAccountInfo);
    context.setAccount(usdcUsdPriceFeedAccountPubkey, usdcFeedAccountInfo);
  
    console.log('pricefeed:', solUsdPriceFeedAccount);
  
    console.log('Pyth Account Info:', accountInfo);
  
    program = new Program<Lending2>(IDL as Lending2, provider);
  
    banksClient = context.banksClient;
  
    signer = provider.wallet.payer;
  
    const mintUSDC = await createMint(
      // @ts-ignore
      banksClient,
      signer,
      signer.publicKey,
      null,
      2
    );
  
    const mintSOL = await createMint(
      // @ts-ignore
      banksClient,
      signer,
      signer.publicKey,
      null,
      2
    );


    [solBankAccount] = PublicKey.findProgramAddressSync(
        [mintSOL.toBuffer()],
        program.programId
    );
  
    [solBankTokenAccount] = PublicKey.findProgramAddressSync(
      [Buffer.from('treasury'), mintSOL.toBuffer()],
      program.programId
    );

    const [userAccount] = PublicKey.findProgramAddressSync(
        [
            Buffer.from('user'),
            signer.publicKey.toBuffer(),
        ],
        program.programId
    )


    const [userTokenAccount] = PublicKey.findProgramAddressSync(
        [
            Buffer.from('user-token'),
            signer.publicKey.toBuffer(),
            mintUSDC.toBuffer(),
        ],
        program.programId
      );

      const [userTokenAccount2] = PublicKey.findProgramAddressSync(
        [
            Buffer.from('user-token'),
            signer.publicKey.toBuffer(),
            mintSOL.toBuffer(),
        ],
        program.programId
      );

     [usdcBankAccount] = PublicKey.findProgramAddressSync(
        [mintUSDC.toBuffer()],
        program.programId
      );

    [usdcBankTokenAccount] = PublicKey.findProgramAddressSync(
      [Buffer.from('treasury'), mintUSDC.toBuffer()],
      program.programId
    );
    
  
    it('Test Init User1', async () => {
    console.log("mintUSDC: ", mintUSDC); 
      const initUserTx = await program.methods
        .initUserTokenAccount("USDC", mintUSDC)
        .accounts({
          signer: signer.publicKey,
          mint: mintUSDC,
          userTokenAccount: userTokenAccount,
        })
        .rpc({ commitment: 'confirmed' });
  
      console.log('Create User Account', initUserTx);
    });

    it('Test Init User2', async () => {
          const initUserTx = await program.methods
            .initUserTokenAccount("SOL", mintSOL)
            .accounts({
              signer: signer.publicKey,
              mint: mintSOL,
              userTokenAccount: userTokenAccount2,
            })
            .rpc({ commitment: 'confirmed' });
      
          console.log('Create User Account', initUserTx);
        });
    
    it('Fetch Account Data', async () => { 
        await DisplayUserTokenAccount(program, userTokenAccount);
        await DisplayUserTokenAccount(program, userTokenAccount2);
    });

    // it('Test Init and Fund USDC Bank', async () => {
    //     const initUSDCBankTx = await program.methods.initBank
    //       (new BN(8000), new BN(5000), USDC_PRICE_FEED_ID)
    //       .accounts({
    //         signer: signer.publicKey,
    //         mint: mintUSDC,
    //         tokenProgram: TOKEN_PROGRAM_ID,
    //       })
    //       .rpc({ commitment: 'confirmed' });
    
    //     console.log('Create USDC Bank Account', initUSDCBankTx);
    //   });

    //   it('Test Init amd Fund SOL Bank', async () => {
    //     const initSOLBankTx = await program.methods
    //       .initBank(new BN(8000), new BN(5000), SOL_PRICE_FEED_ID)
    //       .accounts({
    //         signer: signer.publicKey,
    //         mint: mintSOL,
    //         tokenProgram: TOKEN_PROGRAM_ID,
    //       })
    //       .rpc({ commitment: 'confirmed' });
    
    //     console.log('Create SOL Bank Account', initSOLBankTx);

        // const amount = 10_000 * 10 ** 9;
        // const mintSOLTx = await mintTo(
        //   // @ts-ignores
        //   banksClient,
        //   signer,
        //   mintSOL,
        //   solBankTokenAccount,
        //   signer,
        //   amount
        // );
    
        // console.log('Mint to SOL Bank Signature:', mintSOLTx);
    
    //   });


    //   it('Create and Fund Token Account', async () => {
    //     const USDCTokenAccount = await createAccount(
    //       // @ts-ignores
    //       banksClient,
    //       signer,
    //       mintUSDC,
    //       signer.publicKey
    //     );
    
    //     console.log('USDC Token Account Created:', USDCTokenAccount);
    
    //     const amount = 10_000 * 10 ** 9;
    //     const mintUSDCTx = await mintTo(
    //       // @ts-ignores
    //       banksClient,
    //       signer,
    //       mintUSDC,
    //       USDCTokenAccount,
    //       signer,
    //       amount
    //     );
    
    //     console.log('Mint to USDC Bank Signature:', mintUSDCTx);
    //   });

    //   it('Test Deposit', async () => {

    //     console.log("usdcUsdPriceFeedAccount: ", usdcUsdPriceFeedAccount);

    //     const depositUSDC = await program.methods
    //     .deposit(new BN(10)) // Deposit 20,000 units
    //     .accounts({
    //       signer: signer.publicKey,
    //       mint: mintUSDC,
    //       tokenProgram: TOKEN_PROGRAM_ID,
    //       priceUpdate: usdcUsdPriceFeedAccount,
    //     })
    //     .rpc({ commitment: 'confirmed' });
    
    //     console.log('Deposit USDC successful:', depositUSDC);
    //   });

    // it('Fetch Account Data', async () => { 
    //     await DisplayUser(program, userAccount);
    //     await DisplayUserTokenAccount(program, userTokenAccount);
    //     await DisplayUserTokenAccount(program, userTokenAccount2);
    //     await DisplayBankData(program, usdcBankAccount);
    //  });


        // it('Test Borrow SOL', async () => {

        //     console.log("solUsdPriceFeedAccount: ", solUsdPriceFeedAccount);

        //     const borrowSOL = await program.methods
        //       .borrow(new BN(10))
        //       .accounts({
        //         signer: signer.publicKey,
        //         user_account: userAccount,
        //         mintBorrow: mintSOL,
        //         bankBorrow: solBankAccount, 
        //         bankBorrowTokenAccount: solBankTokenAccount,

        //         userTokenAccountBorrow: userTokenAccount2,
        //         tokenProgram: TOKEN_PROGRAM_ID,
        //         priceUpdate: solUsdPriceFeedAccount,
        //       })
        //       .rpc({ commitment: 'confirmed' });
        
        //     console.log('Borrow SOL', borrowSOL);
        //   });

    //       it('Test Borrow USDC', async () => {

    //         const borrowUSDC = await program.methods
    //           .borrow(new BN(5))
    //           .accounts({
    //             signer: signer.publicKey,
    //             user_account: userAccount,
    //             mintBorrow: mintUSDC,
    //             bankBorrow: usdcBankAccount, 
    //             bankBorrowTokenAccount: usdcBankTokenAccount,

    //             userTokenAccountBorrow: userTokenAccount,
    //             tokenProgram: TOKEN_PROGRAM_ID,
    //             priceUpdate: usdcUsdPriceFeedAccount,
    //           })
    //           .rpc({ commitment: 'confirmed' });
        
    //         console.log('Borrow USDC', borrowUSDC);
    //       });


    //       it('Fetch Account Data', async () => { 
    //         await DisplayUser(program, userAccount);
    //         await DisplayUserTokenAccount(program, userTokenAccount);
    //         await DisplayBankData(program, usdcBankAccount);
    //      });

    // it('Test Withdraw', async () => {

    //     const withdrawUSDC = await program.methods
    //     .withdraw(new BN(1))
    //     .accounts({
    //       signer: signer.publicKey,
    //       mint: mintUSDC,
    //       userAccount: userAccount,
    //       userTokenAccount: userTokenAccount,
    //       bank: usdcBankAccount,
    //       bankTokenAccount: usdcBankTokenAccount,   
    //       tokenProgram: TOKEN_PROGRAM_ID,
    //       priceUpdate: usdcUsdPriceFeedAccount,
    //     })
    //     .rpc({ commitment: 'confirmed' });
  
    //   console.log('Withdraw USDC', withdrawUSDC);

    // });

    // it('Fetch Account Data', async () => { 
    //     await DisplayUser(program, userAccount);
    //     await DisplayUserTokenAccount(program, userTokenAccount);
    //     await DisplayUserTokenAccount(program, userTokenAccount2);
    //     await DisplayBankData(program, usdcBankAccount);
    //  });


    // it('Test Repay', async () => {

    //     const repaySOL = await program.methods
    //     .repay(new BN(2))
    //     .accounts({
    //       signer: signer.publicKey,
    //       mint: mintSOL,
    //       userAccount: userAccount,
    //       userTokenAccount: userTokenAccount2,
    //       bank: solBankAccount,
    //       bankTokenAccount: solBankTokenAccount,   
    //       tokenProgram: TOKEN_PROGRAM_ID,
    //       priceUpdate: solUsdPriceFeedAccount,
    //     })
    //     .rpc({ commitment: 'confirmed' });
  
    //   console.log('Repay SOL', repaySOL);

    // });

    // it('Test Repay USDC', async () => {

    //     const repayUSDC = await program.methods
    //     .repay(new BN(4))
    //     .accounts({
    //       signer: signer.publicKey,
    //       mint: mintUSDC,
    //       userAccount: userAccount,
    //       userTokenAccount: userTokenAccount,
    //       bank: usdcBankAccount,
    //       bankTokenAccount: usdcBankTokenAccount,   
    //       tokenProgram: TOKEN_PROGRAM_ID,
    //       priceUpdate: usdcUsdPriceFeedAccount,
    //     })
    //     .rpc({ commitment: "confirmed", skipPreflight: true });
  
    //   console.log('Repay SOL', repayUSDC);

    // });

    // it('Fetch Account Data', async () => { 
    //     await DisplayUser(program, userAccount);
    //     await DisplayUserTokenAccount(program, userTokenAccount);
    //     await DisplayBankData(program, usdcBankAccount);
    //  });
    
  });