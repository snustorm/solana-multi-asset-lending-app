

use anchor_lang::prelude::*;
use anchor_spl::{
    associated_token::AssociatedToken,
    token_interface::{self, Mint, TokenAccount, TokenInterface, TransferChecked},
};
use pyth_solana_receiver_sdk::price_update::{get_feed_id_from_hex, PriceUpdateV2};

use crate::constants::{AMOUNT_SCALE, MAX_AGE, MAX_LTV_RATE_SCALE};
use crate::{error::ErrorCode, state::UserTokenAccount};
use crate::state::{Bank, User};

#[derive(Accounts)]
pub struct Withdraw<'info> {
    #[account(mut)]
    pub signer: Signer<'info>,

    pub mint: InterfaceAccount<'info, Mint>,

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
        init_if_needed,
        payer = signer,
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

pub fn process_withdraw(ctx: Context<Withdraw>, amount: u64) -> Result<()> {
    
    let user_token_account = &mut ctx.accounts.user_token_account;
    let user_account= &mut ctx.accounts.user_account;
    let bank = &mut ctx.accounts.bank;
    let price_update = &ctx.accounts.price_update;
    
    let price_feed_id = get_feed_id_from_hex(&bank.price_feed_id)?;

    let price_data = price_update.get_price_no_older_than(
        &Clock::get()?,
        MAX_AGE,
        &price_feed_id,
    )?;

    let token_price = price_data.price as f64 * 10f64.powi(price_data.exponent);
    msg!("Token Price: {}", token_price);

    // Convert withdrawal amount to its USD value

    let withdrawal_value = (amount as f64 * token_price).round() as u64;

    msg!("Withdrawal Value: {}", withdrawal_value);

    // Ensure user has enough deposit to withdraw
    require!(
        user_token_account.deposit_amount >= amount,
        ErrorCode::InsufficientCollateral
    );

    // Calculate user's remaining collateral value after withdrawal
    let remaining_collateral_value = user_account
        .total_deposit_value
        .checked_sub(withdrawal_value)
        .ok_or(ErrorCode::MathOverflow)?;

    msg!("Remaining Collateral Value: {}", remaining_collateral_value);

    msg!("Max LTV: {}", bank.max_ltv);

    // Ensure the remaining collateral meets the max LTV requirement
    let max_allowed_borrow_value = remaining_collateral_value
        .checked_mul(bank.max_ltv)
        .ok_or_else(|| ErrorCode::MathOverflow)?
        .checked_div(MAX_LTV_RATE_SCALE)
        .ok_or_else(|| ErrorCode::MathOverflow)?;

    msg!("Max Allowed Borrow Value: {}", max_allowed_borrow_value);
    
    require!(
        user_account.total_borrow_value <= max_allowed_borrow_value,
        ErrorCode::ExceedsMaxLTV
    );

    // Perform all state updates first
    user_token_account.deposit_amount = user_token_account
        .deposit_amount
        .checked_sub(amount * AMOUNT_SCALE)
        .ok_or(ErrorCode::MathOverflow)?;
    user_token_account.deposit_shares = user_token_account
        .deposit_shares
        .checked_sub(amount * AMOUNT_SCALE)
        .ok_or(ErrorCode::MathOverflow)?;
    user_account.total_deposit_value = user_account
        .total_deposit_value
        .checked_sub(withdrawal_value)
        .ok_or(ErrorCode::MathOverflow)?;
    bank.total_deposits = bank
        .total_deposits
        .checked_sub(amount * AMOUNT_SCALE)
        .ok_or(ErrorCode::MathOverflow)?;
    bank.total_deposits_shares = bank
        .total_deposits_shares
        .checked_sub(amount * AMOUNT_SCALE)
        .ok_or(ErrorCode::MathOverflow)?;
    

    // Now call `transfer_tokens` after all mutable borrows are finished
    transfer_tokens(ctx, amount)?;
    msg!("Withdrawal successful");

    Ok(())
}

fn transfer_tokens(ctx: Context<Withdraw>, amount: u64) -> Result<()> {
    let transfer_cpi_accounts = TransferChecked {
        from: ctx.accounts.bank_token_account.to_account_info(),
        to: ctx.accounts.user_associated_token_account.to_account_info(),
        authority: ctx.accounts.bank_token_account.to_account_info(),
        mint: ctx.accounts.mint.to_account_info(),
    };

    let mint_key = ctx.accounts.mint.key();

    let signer_seeds: &[&[&[u8]]] = &[&[
        b"treasury",
        mint_key.as_ref(),
        &[ctx.bumps.bank_token_account],
    ]];

    let cpi_ctx = CpiContext::new_with_signer(
        ctx.accounts.token_program.to_account_info(),
        transfer_cpi_accounts,
        signer_seeds,
    );

    let decimals = ctx.accounts.mint.decimals;

    token_interface::transfer_checked(cpi_ctx, amount, decimals)?;

    Ok(())
}