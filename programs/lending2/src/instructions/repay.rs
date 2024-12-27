use anchor_lang::prelude::*;
use anchor_spl::{
    associated_token::AssociatedToken,
    token_interface::{
       self,
       Mint, 
       TokenAccount, 
       TokenInterface, 
       TransferChecked
   }};
use pyth_solana_receiver_sdk::price_update::{get_feed_id_from_hex, PriceUpdateV2};

use crate::{constants::{AMOUNT_SCALE, MAX_AGE}, state::{Bank, User, UserTokenAccount}};
use crate::error::ErrorCode;

#[derive(Accounts)]
pub struct Repay<'info> {

    #[account(mut)]
    pub signer: Signer<'info>,

    pub mint: InterfaceAccount<'info, Mint>,

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
        seeds = [b"user", signer.key().as_ref()],
        bump,
    )]
    pub user_account: Account<'info, User>, 
 
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
    pub user_associated_token_account: InterfaceAccount<'info, TokenAccount>,

    pub price_update: Account<'info, PriceUpdateV2>,
    
    pub token_program: Interface<'info, TokenInterface>,    
    pub associated_token_program: Program<'info, AssociatedToken>,
    pub system_program: Program<'info, System>,
}

pub fn process_repay(ctx: Context<Repay>, amount: u64) -> Result<()> {

    let user_account = &mut ctx.accounts.user_account;
    let bank = &mut ctx.accounts.bank;
    let user_token_account = &mut ctx.accounts.user_token_account;
    let price_update = &ctx.accounts.price_update;
    let scaled_amount = amount.checked_mul(AMOUNT_SCALE).unwrap();

    let price_feed_id = get_feed_id_from_hex(&bank.price_feed_id)?;

    let price_data = price_update.get_price_no_older_than(
        &Clock::get()?,
        MAX_AGE,
        &price_feed_id,
    )?;
    let token_price = price_data.price as f64 * 10f64.powi(price_data.exponent);
    msg!("Token Price: {}", token_price);


    let repay_value = (amount as f64 * token_price).round() as u64;
    msg!("Withdrawal Value: {}", repay_value);


    require!(
        user_token_account.borrowed_amount >= amount,
        ErrorCode::OverRepay
    );
    

    // Perform all state updates first
    msg!("Borrowed Amount: {}", user_token_account.borrowed_amount);
    user_token_account.borrowed_amount = user_token_account
        .borrowed_amount
        .checked_sub(scaled_amount)
        .ok_or(ErrorCode::MathOverflow)?;
    user_token_account.borrowed_shares = user_token_account
        .borrowed_shares
        .checked_sub(scaled_amount)
        .ok_or(ErrorCode::MathOverflow)?;
    user_account.total_borrow_value = user_account
        .total_borrow_value
        .checked_sub(repay_value)
        .ok_or(ErrorCode::MathOverflow)?;
    bank.total_borrowed = bank
        .total_borrowed
        .checked_sub(scaled_amount)
        .ok_or(ErrorCode::MathOverflow)?;
    bank.total_borrowed_shares = bank
        .total_borrowed_shares
        .checked_sub(scaled_amount)
        .ok_or(ErrorCode::MathOverflow)?;

    transfer_tokens(ctx, amount)?;


    Ok(())   
}

fn transfer_tokens(ctx: Context<Repay>, amount: u64) -> Result<()> {
    
    let transfer_cpi_accounts = TransferChecked {
        from: ctx.accounts.user_associated_token_account.to_account_info(),
        to: ctx.accounts.bank_token_account.to_account_info(),
        authority: ctx.accounts.signer.to_account_info(),
        mint: ctx.accounts.mint.to_account_info(),
    };

    let cpi_program = ctx.accounts.token_program.to_account_info();

    let cpi_ctx = CpiContext::new(
        cpi_program,
        transfer_cpi_accounts,
    );

    let decimals = ctx.accounts.mint.decimals;
    require!(decimals > 0, ErrorCode::InvalidDecimals);

    token_interface::transfer_checked(cpi_ctx, amount, decimals)?;

    Ok(())
}