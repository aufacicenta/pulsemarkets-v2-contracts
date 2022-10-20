use near_sdk::{Balance, Gas};

pub const MARKET_CODE: &[u8] = include_bytes!("../res/amm.wasm");

// at current gas price per byte. AMM size is 348101 "wc -c res/amm.wasm"
pub const GAS_FOR_CREATE_MARKET: Gas = Gas(30_000_000_000_000);
pub const GAS_FOR_CREATE_MARKET_CALLBACK: Gas = Gas(150_000_000_000_000);

pub const GAS_FOR_CREATE_OUTCOME_TOKENS: Gas = Gas(15_000_000_000_000);
pub const GAS_FOR_CREATE_OUTCOME_TOKENS_CALLBACK: Gas = Gas(15_000_000_000_000);

pub const STORAGE_DEPOSIT_BOND: Balance = 2_350_000_000_000_000_000_000; // 0.00235 NEAR
pub const GAS_FOR_FT_STORAGE_DEPOSIT: Gas = Gas(15_000_000_000_000);
