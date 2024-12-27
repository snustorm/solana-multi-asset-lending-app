use anchor_lang::prelude::*;
use anchor_spl::token_interface::{
    Mint, 
    TokenAccount, 
    TokenInterface
};

use crate::{constants::INTEREST_RATE_SCALE, state::Bank};

#[derive(Accounts)] 
pub struct InitBank<'info> {
    #[account(mut)]
    pub signer: Signer<'info>,

    pub mint: InterfaceAccount<'info, Mint>,   

    #[account(
        init,
        payer = signer,
        space = 8 + Bank::INIT_SPACE,
        seeds = [mint.key().as_ref()],
        bump,
    )]
    pub bank: Account<'info, Bank>,

    #[account(
        init,
        token::mint = mint,
        token::authority = bank_token_account,
        payer = signer,
        seeds = [b"treasury", mint.key().as_ref()],
        bump,
    )]
    pub bank_token_account: InterfaceAccount<'info, TokenAccount>,
    
    pub token_program: Interface<'info, TokenInterface>,
    pub system_program: Program<'info, System>,
}

pub fn process_init_bank(ctx: Context<InitBank>,
    liquidation_threshold: u64,
    max_ltv: u64,
    price_feed_id: String,
) -> Result<()> {

    let bank = &mut ctx.accounts.bank;
    bank.mint_address = ctx.accounts.mint.key();
    bank.authority = ctx.accounts.signer.key();
    bank.liquidation_threshold = liquidation_threshold;
    bank.max_ltv = max_ltv;
    bank.interest_rate = (0.05 * INTEREST_RATE_SCALE as f64) as u64;
    bank.price_feed_id = price_feed_id;
    Ok(())
}