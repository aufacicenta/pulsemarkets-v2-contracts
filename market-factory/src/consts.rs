use near_sdk::{Balance, Gas};

pub const MARKET_CODE: &[u8] = include_bytes!("../res/amm.wasm");

pub const GAS_FOR_CREATE_MARKET: Gas = Gas(17_000_000_000_000); // at current gas price per byte. AMM size is 252492 "wc -c res/amm.wasm"
pub const GAS_FOR_PROCESS_MARKET_OPTIONS: Gas = Gas(19_000_000_000_000);
pub const GAS_FOR_CREATE_MARKET_CALLBACK: Gas = Gas(2_000_000_000_000);

pub const BALANCE_PROPOSAL_BOND: Balance = 100_000_000_000_000_000_000_000; // 0.1 Near
