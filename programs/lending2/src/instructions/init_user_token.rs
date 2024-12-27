use anchor_lang::prelude::*;
use anchor_spl::token_interface::Mint;

use crate::state::UserTokenAccount;


#[derive(Accounts)]
pub struct InitUserTokenAccount<'info> {
    #[account(mut)]
    pub signer: Signer<'info>, 

    pub mint: InterfaceAccount<'info, Mint>,   

    #[account(
        init,
        payer = signer,
        space = 8 + UserTokenAccount::INIT_SPACE,
        seeds = [b"user-token", signer.key().as_ref(), mint.key().as_ref()],
        bump,
    )]
    pub user_token_account: Account<'info, UserTokenAccount>, // PDA for the user-token account

    pub system_program: Program<'info, System>, // System program for account initialization
}

pub fn process_init_user_token_account(ctx: Context<InitUserTokenAccount>, name: String, mint: Pubkey,) -> Result<()> {
    let user_token_account = &mut ctx.accounts.user_token_account;

    // Initialize the user-token account fields
    println!("Init User Token Account");
    user_token_account.owner = ctx.accounts.signer.key(); // User's address
    user_token_account.mint = mint.key();    
    user_token_account.name = name;
    user_token_account.bump = ctx.bumps.user_token_account;
    
    
    Ok(())
}