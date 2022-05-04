use near_sdk::Gas;

pub const MARKET_CODE: &[u8] = include_bytes!("./market.wasm");

pub const GAS_FOR_CREATE_MARKET: Gas = Gas(90_000_000_000_000);
pub const GAS_FOR_PROCESS_MARKET_OPTIONS: Gas = Gas(90_000_000_000_000);
pub const GAS_FOR_CREATE_MARKET_CALLBACK: Gas = Gas(2_000_000_000_000);
