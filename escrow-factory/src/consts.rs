use near_sdk::Gas;

pub const CONDITIONAL_ESCROW_CODE: &[u8] = include_bytes!("./conditional_escrow.wasm");

/// Gas spent on the call & account creation.
pub const CREATE_CALL_GAS: Gas = Gas(75_000_000_000_000);

/// Gas allocated on the callback.
pub const ON_CREATE_CALL_GAS: Gas = Gas(10_000_000_000_000);
