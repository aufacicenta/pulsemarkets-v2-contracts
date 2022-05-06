use near_sdk::{Balance, Gas};

pub const MARKET_CODE: &[u8] = include_bytes!("../../market/res/market.wasm");

pub const GAS_FOR_CREATE_MARKET: Gas = Gas(10_000_000_000_000);
pub const GAS_FOR_PROCESS_MARKET_OPTIONS: Gas = Gas(19_000_000_000_000);
pub const GAS_FOR_CREATE_MARKET_CALLBACK: Gas = Gas(2_000_000_000_000);

pub const BALANCE_PROPOSAL_BOND: Balance = 100_000_000_000_000_000_000_000; // 0.1 Near
pub const BALANCE_CREATE_MARKET: Balance = 3_000_000_000_000_000_000_000_000; // 3 Near
