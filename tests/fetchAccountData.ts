const { AccountLayout } = require("@solana/spl-token");

async function DisplayUserTokenAccount( program, userAccount) {
    
    console.log("User Token Account: ", userAccount);
    const userAccountData = await program.account.userTokenAccount.fetch(userAccount);
    
    console.log();
    // Format User Account Data
    console.log("User Account Data:");
    console.log(`- Owner: ${userAccountData.owner.toBase58()}`);
    console.log(`- Name: ${userAccountData.name}`);
    console.log(`- Token Mint Address: ${userAccountData.mint.toBase58()}`);
    console.log(`- Deposit: ${(userAccountData.depositAmount.toNumber())} `);
    console.log(`- Deposit Shares: ${userAccountData.depositShares.toString()}`);
    console.log(`- Borrowed: ${(userAccountData.borrowedAmount.toNumber())} `);
    console.log(`- Borrowed Shares: ${userAccountData.borrowedShares.toString()}`);
    console.log(`- Last Update: ${new Date(userAccountData.lastUpdate.toNumber() * 1000).toLocaleString()}`);
    console.log(`- Last Update Borrow: ${new Date(userAccountData.lastUpdateBorrow.toNumber() * 1000).toLocaleString()}`);

}

async function DisplayBankData( program, bankAccount) {

    const bankData = await program.account.bank.fetch(bankAccount);

    console.log();
    // Format Bank Data
    console.log("Bank Data:");
    console.log(`Authority: ${bankData.authority.toString()}`);
    console.log(`- Mint Address: ${bankData.mintAddress.toBase58()}`);
    console.log(`Total Deposits: ${bankData.totalDeposits.toString()}`);
    console.log(`Total Deposit Shares: ${bankData.totalDepositsShares.toString()}`);
    console.log(`Total Borrowed: ${bankData.totalBorrowed.toString()}`);
    console.log(`Total Borrowed Shares: ${bankData.totalBorrowedShares.toString()}`);
    console.log(`Liquidation Threshold: ${bankData.liquidationThreshold.toString()}`);
    console.log(`Liquidation Bonus: ${bankData.liquidationBonus.toString()}`);
    console.log(`Liquidation Close Factor: ${bankData.liquidationCloseFactor.toString()}`);
    console.log(`Max LTV: ${bankData.maxLtv.toString()}`);
    console.log(`Last Updated: ${bankData.lastUpdated.toString()}`);
    console.log(`Interest Rate: ${(bankData.interestRate.toNumber() / 100).toFixed(2)}%`);
}

async function DisplayUser(program, userAccount ) {
    
    console.log("User Token Account: ", userAccount);
    const userAccountData = await program.account.user.fetch(userAccount);
    
    console.log();
    // Format User Account Data
    console.log("User Data:");
    console.log(`- Total Borrowed: $ ${userAccountData.totalBorrowValue.toString()}`);
    console.log(`- Total Supplied: $ ${userAccountData.totalDepositValue.toString()}`);

}


async function fetchAndDisplayAccountData(program, connection, accounts) {
    const { usdcBankAccount, solBankAccount, userAccount, usdcBankTokenAccount, solBankTokenAccount } = accounts;

    const usdcBankData = await program.account.bank.fetch(usdcBankAccount);
    const solBankData = await program.account.bank.fetch(solBankAccount);
    const userAccountData = await program.account.user.fetch(userAccount);

    console.log();
    // Format USDC Bank Data
    console.log("USDC Bank Data:");
    console.log(`Authority: ${usdcBankData.authority.toString()}`);
    console.log(`Mint Address: ${usdcBankData.mintAddress.toString()}`);
    console.log(`Total Deposits: ${usdcBankData.totalDeposits.toString()}`);
    console.log(`Total Deposit Shares: ${usdcBankData.totalDepositsShares.toString()}`);
    console.log(`Total Borrowed: ${usdcBankData.totalBorrowed.toString()}`);
    console.log(`Total Borrowed Shares: ${usdcBankData.totalBorrowedShares.toString()}`);
    console.log(`Liquidation Threshold: ${usdcBankData.liquidationThreshold.toString()}`);
    console.log(`Liquidation Bonus: ${usdcBankData.liquidationBonus.toString()}`);
    console.log(`Liquidation Close Factor: ${usdcBankData.liquidationCloseFactor.toString()}`);
    console.log(`Max LTV: ${usdcBankData.maxLtv.toString()}`);
    console.log(`Last Updated: ${usdcBankData.lastUpdated.toString()}`);
    console.log(`Interest Rate: ${(usdcBankData.interestRate.toNumber() / 100).toFixed(2)}%`);

    console.log();
    // Format SOL Bank Data
    console.log("SOL Bank Data:");
    console.log(`Authority: ${solBankData.authority.toString()}`);
    console.log(`- Mint Address: ${solBankData.mintAddress.toBase58()}`);
    console.log(`Total Deposits: ${solBankData.totalDeposits.toString()}`);
    console.log(`Total Deposit Shares: ${solBankData.totalDepositsShares.toString()}`);
    console.log(`Total Borrowed: ${solBankData.totalBorrowed.toString()}`);
    console.log(`Total Borrowed Shares: ${solBankData.totalBorrowedShares.toString()}`);
    console.log(`Liquidation Threshold: ${solBankData.liquidationThreshold.toString()}`);
    console.log(`Liquidation Bonus: ${solBankData.liquidationBonus.toString()}`);
    console.log(`Liquidation Close Factor: ${solBankData.liquidationCloseFactor.toString()}`);
    console.log(`Max LTV: ${solBankData.maxLtv.toString()}`);
    console.log(`Last Updated: ${solBankData.lastUpdated.toString()}`);
    console.log(`Interest Rate: ${(solBankData.interestRate.toNumber() / 100).toFixed(2)}%`);

    console.log();
    // Format User Account Data
    console.log("User Account Data:");
    console.log(`- Owner: ${userAccountData.owner.toBase58()}`);
    console.log(`- Deposit SOL: ${(userAccountData.depositSol.toNumber())} SOL`);
    console.log(`- Deposit SOL Shares: ${userAccountData.depositSolShares.toString()}`);
    console.log(`- Borrowed SOL: ${(userAccountData.borrowedSol.toNumber())} SOL`);
    console.log(`- Borrowed SOL Shares: ${userAccountData.borrowedSolShares.toString()}`);
    console.log(`- Deposit USDC: ${(userAccountData.depositUsdc.toNumber())} USDC`);
    console.log(`- Deposit USDC Shares: ${userAccountData.depositUsdcShares.toString()}`);
    console.log(`- Borrowed USDC: ${(userAccountData.borrowedUsdc.toNumber())} USDC`);
    console.log(`- Borrowed USDC Shares: ${userAccountData.borrowedUsdcShares.toString()}`);
    console.log(`- USDC Address: ${userAccountData.usdcAddress.toBase58()}`);
    console.log(`- Last Update: ${new Date(userAccountData.lastUpdate.toNumber() * 1000).toLocaleString()}`);
    console.log(`- Last Update Borrow: ${new Date(userAccountData.lastUpdateBorrow.toNumber() * 1000).toLocaleString()}`);

    const accountInfoUsdc = await connection.getAccountInfo(usdcBankTokenAccount);
    const usdcBalance = AccountLayout.decode(accountInfoUsdc.data).amount;

    const accountInfoSol = await connection.getAccountInfo(solBankTokenAccount);
    const solBalance = AccountLayout.decode(accountInfoSol.data).amount;

    console.log();
    console.log(`USDC Bank Token Account Balance: ${usdcBalance.toString()}`);
    console.log(`SOL Bank Token Account Balance: ${solBalance.toString()}`);
}

module.exports = { fetchAndDisplayAccountData, DisplayUserTokenAccount, DisplayBankData, DisplayUser };