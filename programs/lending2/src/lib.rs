use anchor_lang::prelude::*;

declare_id!("AjnXUaDfPD88JyARjMkYaCDnpbWuGiRZvHdvKfQbGZnt");

use instructions::*;
mod state;
mod instructions;
mod error;  
mod constants;


#[program]
pub mod lending2 {
    use super::*;

    pub fn init_user_token_account(
         ctx: Context<InitUserTokenAccount>,
         name: String,
         mint_address: Pubkey
        ) -> Result<()> {
        msg!("Start Init User Token Account");
        process_init_user_token_account(ctx, name, mint_address)
    }

    pub fn init_bank(
        ctx: Context<InitBank>,
        liquidation_threshold: u64,
        max_ltv: u64,
        price_feed_id: String,
    ) -> Result<()> {
        process_init_bank(ctx, liquidation_threshold, max_ltv, price_feed_id)
    }


    pub fn deposit(ctx: Context<Deposit>, amount: u64) -> Result<()> {
        process_deposit(ctx, amount)
    }

    pub fn borrow(ctx: Context<Borrow>, amount: u64) -> Result<()> {
        process_borrow(ctx, amount)
    }

    pub fn withdraw(ctx: Context<Withdraw>, amount: u64) -> Result<()> {
        process_withdraw(ctx, amount)
    }

    pub fn repay(ctx: Context<Repay>, amount: u64) -> Result<()> {
        process_repay(ctx, amount)
    }


}

