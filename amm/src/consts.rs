use near_sdk::{Balance, Gas};

pub const GAS_CREATE_DAO_PROPOSAL: Gas = Gas(10_000_000_000_000);
pub const GAS_CREATE_DAO_PROPOSAL_CALLBACK: Gas = Gas(2_000_000_000_000);

pub const BALANCE_PROPOSAL_BOND: Balance = 100_000_000_000_000_000_000_000; // 0.1 Near
