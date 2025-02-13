use anchor_lang::prelude::*;
#[constant]
pub const MAX_AGE: u64 = 100;
#[constant]
pub const INTEREST_RATE_SCALE: u64 = 10_000;
#[constant]
pub const LIQUIDATION_THRESHOLD_RATE_SCALE: u64 = 10_000;
#[constant]
pub const MAX_LTV_RATE_SCALE: u64 = 10_000;

#[constant]
pub const AMOUNT_SCALE: u64 = 1_000_000_000;

