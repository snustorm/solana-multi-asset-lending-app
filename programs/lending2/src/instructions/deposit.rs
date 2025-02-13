use anchor_lang::prelude::*;
use anchor_spl::{
    associated_token::AssociatedToken, 
    token_interface::{
        self, 
        Mint, 
        TokenAccount, 
        TokenInterface, 
        TransferChecked
    }
};
use pyth_solana_receiver_sdk::price_update::{get_feed_id_from_hex, PriceUpdateV2};

use crate::{constants::{AMOUNT_SCALE, MAX_AGE}, state::{Bank, User, UserTokenAccount}};

#[derive(Accounts)]
pub struct Deposit<'info> {
    #[account(mut)]
    pub signer: Signer<'info>,

    pub mint: InterfaceAccount<'info, Mint>,

    #[account(
        init_if_needed,
        payer = signer,
        space = 8 + User::INIT_SPACE,
        seeds = [b"user", signer.key().as_ref()],
        bump,
    )]
    pub user_account: Account<'info, User>, 

    #[account(
        mut,
        seeds = [mint.key().as_ref()],
        bump,
    )]
    pub bank: Account<'info, Bank>,

    #[account(
        mut,
        seeds = [b"treasury", mint.key().as_ref()],
        bump,
    )]
    pub bank_token_account: InterfaceAccount<'info, TokenAccount>,

    #[account(
        mut,
        seeds = [b"user-token", signer.key().as_ref(), mint.key().as_ref()],
        bump,
    )]
    pub user_token_account: Account<'info, UserTokenAccount>, 

    #[account(
        mut,
        associated_token::mint = mint,
        associated_token::authority = signer,
        associated_token::token_program = token_program,
    )]
    pub user_token_associated_account: InterfaceAccount<'info, TokenAccount>,

    pub price_update: Account<'info, PriceUpdateV2>,

    pub token_program: Interface<'info, TokenInterface>,
    pub system_program: Program<'info, System>,
    pub associated_token_program: Program<'info, AssociatedToken>,
}

pub fn process_deposit(ctx: Context<Deposit>, amount: u64) -> Result<()> {
    // Scale the deposit amount (e.g., scale up by 10^6)

    let user = & mut ctx.accounts.user_account; 
    let bank = &mut ctx.accounts.bank;
    
    let price_update = &ctx.accounts.price_update;
    let price_feed_id = get_feed_id_from_hex(&bank.price_feed_id)?;

    let price_data = price_update.get_price_no_older_than(
        &Clock::get()?,
        MAX_AGE,
        &price_feed_id,
    )?;

    let actual_price = price_data.price as f64 * 10f64.powi(price_data.exponent);

    user.total_deposit_value += (actual_price * amount as f64 ) as u64;   

    let scaled_amount = amount.checked_mul(AMOUNT_SCALE).unwrap();
    msg!("Deposit amount: {:?}", scaled_amount);    
    // Transfer the scaled deposit amount from the user's token account to the bank's treasury
    let transfer_cpi_accounts = TransferChecked {
        from: ctx.accounts.user_token_associated_account.to_account_info(),
        to: ctx.accounts.bank_token_account.to_account_info(),
        authority: ctx.accounts.signer.to_account_info(),
        mint: ctx.accounts.mint.to_account_info(),
    };

    let cpi_program = ctx.accounts.token_program.to_account_info();

    let cpi_ctx = CpiContext::new(cpi_program, transfer_cpi_accounts);

    let decimals = ctx.accounts.mint.decimals;
    token_interface::transfer_checked(cpi_ctx, scaled_amount, decimals)?;

    // Update the bank's total deposits and shares
    let bank = &mut ctx.accounts.bank;

    if bank.total_deposits == 0 {
        // Initialize with the scaled deposit amount on the first deposit
        bank.total_deposits = scaled_amount;
        bank.total_deposits_shares = scaled_amount;
    } else {
        // Only update the total deposits and shares after the first deposit
        let deposit_ratio = scaled_amount.checked_div(bank.total_deposits).unwrap();
        let user_share = bank.total_deposits_shares.checked_mul(deposit_ratio).unwrap();

        bank.total_deposits += scaled_amount;
        bank.total_deposits_shares += user_share;
    }

    // Update the user's token account with the scaled deposit amount and shares
    let user_token_account = &mut ctx.accounts.user_token_account;

    // Update the deposit and deposit shares for the specific token
    user_token_account.deposit_amount += scaled_amount;
    user_token_account.deposit_shares += scaled_amount;

    user_token_account.last_update = Clock::get()?.unix_timestamp;



    Ok(())
}