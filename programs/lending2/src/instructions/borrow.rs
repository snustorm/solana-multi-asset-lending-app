use anchor_lang::prelude::*;
use anchor_spl::{
    associated_token::AssociatedToken,
    token_interface::{self, Mint, TokenAccount, TokenInterface, TransferChecked},
};
use pyth_solana_receiver_sdk::price_update::{get_feed_id_from_hex, PriceUpdateV2};

use crate::{constants::{AMOUNT_SCALE, LIQUIDATION_THRESHOLD_RATE_SCALE, MAX_AGE}, state::User};
use crate::state::{Bank, UserTokenAccount};
use crate::error::ErrorCode;

// Collateral USDC to Borrow SOL / Collateral SOL to Borrow USDC
#[derive(Accounts)]
pub struct Borrow<'info> {
    #[account(mut)]
    pub signer: Signer<'info>,

    pub mint_borrow: InterfaceAccount<'info, Mint>,

    #[account(
        mut,
        seeds = [b"user", signer.key().as_ref()],
        bump,
    )]
    pub user_account: Account<'info, User>, 

    #[account(
        mut,
        seeds = [mint_borrow.key().as_ref()],
        bump,
    )]
    pub bank_borrow: Account<'info, Bank>,

    #[account(
        mut,
        seeds = [b"treasury", mint_borrow.key().as_ref()],
        bump,
    )]
    pub bank_token_account_borrow: InterfaceAccount<'info, TokenAccount>,

    #[account(
        mut,
        seeds = [b"user-token", signer.key().as_ref(), mint_borrow.key().as_ref()],
        bump,
    )]
    pub user_token_account_borrow: Account<'info, UserTokenAccount>, 

    #[account(
        init_if_needed,
        payer = signer,
        associated_token::mint = mint_borrow,
        associated_token::authority = signer,
        associated_token::token_program = token_program,
    )]
    pub user_associated_token_account: InterfaceAccount<'info, TokenAccount>, // ATA

    pub price_update: Account<'info, PriceUpdateV2>,

    pub token_program: Interface<'info, TokenInterface>,
    pub associated_token_program: Program<'info, AssociatedToken>,
    pub system_program: Program<'info, System>,
}

pub fn process_borrow(ctx: Context<Borrow>, amount: u64) -> Result<()> {
    
    let user = & mut ctx.accounts.user_account; 
    let bank_borrow = &mut ctx.accounts.bank_borrow;
    let user_token_account_borrow = &mut ctx.accounts.user_token_account_borrow;
    let price_update = &ctx.accounts.price_update;

    // Scale amount by 10^6 for precision
    let scaled_amount = amount.checked_mul(AMOUNT_SCALE).unwrap();
    let actual_price: f64;

    let total_collateral = user.total_deposit_value;
    
    let price_feed_id = get_feed_id_from_hex(&bank_borrow.price_feed_id)?;

    let price_data = price_update.get_price_no_older_than(
            &Clock::get()?,
            MAX_AGE,
            &price_feed_id,
        )?;

    actual_price = price_data.price as f64 * 10f64.powi(price_data.exponent);

    // Calculate borrowable amount
    let borrowable_amount = total_collateral as f64
        * (bank_borrow.liquidation_threshold as f64 / LIQUIDATION_THRESHOLD_RATE_SCALE as f64); 

    msg!("Borrowable Amount: {}", borrowable_amount);
    msg!("Amount: {}", amount);
    msg!("Scaled Amount: {}", scaled_amount);
    msg!("Actual Value: {}", actual_price * amount as f64);

    if borrowable_amount < actual_price * amount as f64  {
        return Err(ErrorCode::OverBorrowableAmount.into());
    }
    msg!("Transfer");
    // Transfer borrowed amount to user's ATA
    let transfer_cpi_accounts = TransferChecked {
        from: ctx.accounts.bank_token_account_borrow.to_account_info(),
        to: ctx.accounts.user_associated_token_account.to_account_info(),
        authority: ctx.accounts.bank_token_account_borrow.to_account_info(),
        mint: ctx.accounts.mint_borrow.to_account_info(),
    };

    let cpi_program = ctx.accounts.token_program.to_account_info();
    
    let mint_key = ctx.accounts.mint_borrow.key();
    let signer_seeds: &[&[&[u8]]] = &[
        &[
            b"treasury",
            mint_key.as_ref(),
            &[ctx.bumps.bank_token_account_borrow],
        ]
    ];
    
    let cpi_ctx = CpiContext::new_with_signer(cpi_program, transfer_cpi_accounts, signer_seeds);
    token_interface::transfer_checked(cpi_ctx, scaled_amount, ctx.accounts.mint_borrow.decimals)?;

    // Update bank's borrow state
    if bank_borrow.total_borrowed > 0 {
        let borrow_ratio = scaled_amount.checked_div(bank_borrow.total_borrowed).unwrap_or(0);
        let user_shares = bank_borrow.total_borrowed_shares.checked_mul(borrow_ratio).unwrap_or(0);
    
        bank_borrow.total_borrowed += scaled_amount;
        bank_borrow.total_borrowed_shares += user_shares;
    } else {
        bank_borrow.total_borrowed = scaled_amount;
        bank_borrow.total_borrowed_shares = scaled_amount;
    }

    // Update user_token_account's borrow state
    user_token_account_borrow.borrowed_amount += scaled_amount;
    user_token_account_borrow.borrowed_shares += scaled_amount;
    user_token_account_borrow.last_update = Clock::get()?.unix_timestamp;

    user.total_borrow_value += (actual_price * amount as f64 ) as u64;   

    Ok(())
}

// fn calculate_accrued_interest(deposit: u64, interest_rate: u64, last_update: i64) -> Result<u64> {
    
//     let current_time = Clock::get()?.unix_timestamp;
//     let new_last_update = 1731872856;
//     let time_diff = current_time - new_last_update;
//     if time_diff <= 0 {
//         return Ok(deposit); // No time passed, return the original deposit
//     }
//     // Interest formula with scaled-up precision
//     let interest_rate_scaled = interest_rate as f64 / 10_000.0; // Scale back to actual interest rate
//     let accrued_interest: f64 = deposit as f64 * (1.0 + (interest_rate_scaled * time_diff as f64 / 31_536_000.0)); // 1 year = 31_536_000 seconds
//     Ok(accrued_interest as u64)
// }

// pub fn old_process_borrow(ctx: Context<Borrow>, amount: u64) -> Result<()> {
    
//     let user = & mut ctx.accounts.user_account; 
//     let bank_borrow = &mut ctx.accounts.bank_borrow;
//     let user_token_account_lend = &mut ctx.accounts.user_token_account_lend;
//     let user_token_account_borrow = &mut ctx.accounts.user_token_account_borrow;
//     let price_update = &ctx.accounts.price_update;

//     // Scale amount by 10^6 for precision
//     let scaled_amount = amount.checked_mul(1_000_000).unwrap();
//     let actual_price: f64;

//     // Calculate collateral value
//     let total_collateral = {

//         let price_feed_id = get_feed_id_from_hex(&bank_borrow.price_feed_id)?;

//         let price_data = price_update.get_price_no_older_than(
//             &Clock::get()?,
//             MAX_AGE,
//             &price_feed_id,
//         )?;

//         actual_price = price_data.price as f64 * 10f64.powi(price_data.exponent);

//         msg!("Actual Price: {}", actual_price);
//         msg!("Deposit: {}", user_token_account_lend.deposit_amount);
//         msg!("interest_rate: {}", bank_borrow.interest_rate);
//         let accrued_interest = calculate_accrued_interest(
//             user_token_account_lend.deposit_amount,
//             bank_borrow.interest_rate,
//             user_token_account_lend.last_update,
//         )?;
//         msg!("Accrued Interest: {}", accrued_interest);
//         let collateral_value = accrued_interest as f64 * actual_price;
//         msg!("Collateral Value: {}", collateral_value);
//         collateral_value as u64
//     };

//     // Calculate borrowable amount
//     let borrowable_amount = total_collateral as f64
//         * (bank_borrow.liquidation_threshold as f64 / LIQUIDATION_THRESHOLD_RATE_SCALE as f64); 

//     msg!("Borrowable Amount: {}", borrowable_amount);

//     if borrowable_amount < scaled_amount as f64 {
//         return Err(ErrorCode::OverBorrowableAmount.into());
//     }

//     // Transfer borrowed amount to user's ATA
//     let transfer_cpi_accounts = TransferChecked {
//         from: ctx.accounts.bank_token_account_borrow.to_account_info(),
//         to: ctx.accounts.user_associated_token_account.to_account_info(),
//         authority: ctx.accounts.bank_token_account_borrow.to_account_info(),
//         mint: ctx.accounts.mint_borrow.to_account_info(),
//     };

//     let cpi_program = ctx.accounts.token_program.to_account_info();
    
//     let mint_key = ctx.accounts.mint_borrow.key();
//     let signer_seeds: &[&[&[u8]]] = &[
//         &[
//             b"treasury",
//             mint_key.as_ref(),
//             &[ctx.bumps.bank_token_account_borrow],
//         ]
//     ];
    
//     let cpi_ctx = CpiContext::new_with_signer(cpi_program, transfer_cpi_accounts, signer_seeds);
//     token_interface::transfer_checked(cpi_ctx, amount, ctx.accounts.mint_borrow.decimals)?;

//     // Update bank's borrow state
//     if bank_borrow.total_borrowed > 0 {
//         let borrow_ratio = scaled_amount.checked_div(bank_borrow.total_borrowed).unwrap_or(0);
//         let user_shares = bank_borrow.total_borrowed_shares.checked_mul(borrow_ratio).unwrap_or(0);
    
//         bank_borrow.total_borrowed += scaled_amount;
//         bank_borrow.total_borrowed_shares += user_shares;
//     } else {
//         bank_borrow.total_borrowed = scaled_amount;
//         bank_borrow.total_borrowed_shares = scaled_amount;
//     }

//     // Update user_token_account's borrow state
//     user_token_account_borrow.borrowed_amount += scaled_amount;
//     user_token_account_borrow.borrowed_shares += scaled_amount;
//     user_token_account_borrow.last_update = Clock::get()?.unix_timestamp;

//     user.total_borrow_value += (actual_price * amount as f64 ) as u64;   

//     Ok(())
// }
