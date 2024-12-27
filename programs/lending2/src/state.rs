use anchor_lang::prelude::*;

#[account]
#[derive(InitSpace)]
pub struct User {

    pub owner: Pubkey,  
    pub total_deposit_value: u64,
    pub total_borrow_value: u64,
}

#[account]
#[derive(InitSpace)]
pub struct Bank {
    pub authority: Pubkey,
    pub mint_address: Pubkey,
    pub total_deposits: u64,
    pub total_deposits_shares: u64,
    pub total_borrowed: u64,
    pub total_borrowed_shares: u64,
    pub liquidation_threshold: u64,
    pub liquidation_bonus: u64,
    pub liquidation_close_factor: u64,
    pub max_ltv: u64,
    pub last_updated: i64,
    pub interest_rate: u64,
    #[max_len(84)]
    pub price_feed_id: String,
}


#[account]
#[derive(InitSpace)]
pub struct UserTokenAccount {
    pub owner: Pubkey,          // The user's address
    #[max_len(8)]
    pub name: String,           // The token name
    pub mint: Pubkey,           // The token's mint address (e.g., USDC or SOL)
    pub deposit_amount: u64,    // Actual deposit amount (for simplicity, optional)
    pub deposit_shares: u64,    // User's deposit shares
    pub borrowed_amount: u64,   // Actual borrowed amount (for simplicity, optional)
    pub borrowed_shares: u64,   // User's borrow shares
    pub last_update: i64, 
    pub last_update_borrow: i64, 
    pub bump: u8,     
}