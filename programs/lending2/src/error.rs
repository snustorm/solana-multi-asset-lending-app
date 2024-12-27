use anchor_lang::prelude::*;

#[error_code]
pub enum ErrorCode {
    #[msg("The user does not have enough funds to withdraw")]
    InsufficientFunds,
    #[msg("Requested amount exceeds borrowable amount")]
    OverBorrowableAmount,
    #[msg("Over Repay!")]
    OverRepay,
    #[msg("User is not under collateralized! can't be liquidated")]
    NotUnderCollateralized,
    #[msg("Math Over Flow")]
    MathOverflow,
    #[msg("Insufficient Amount ")]
    InsufficientBorrow,
    #[msg("Exceeds MAX LTV")]
    ExceedsMaxLTV,
    #[msg("Insufficient Collateral")]
    InsufficientCollateral,
    #[msg("Invalid Decimals")]
    InvalidDecimals
}
