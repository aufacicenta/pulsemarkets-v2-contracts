use near_sdk::{Balance, Gas, ONE_YOCTO};

pub const GAS_CREATE_DAO_PROPOSAL: Gas = Gas(10_000_000_000_000);
pub const GAS_CREATE_DAO_PROPOSAL_CALLBACK: Gas = Gas(2_000_000_000_000);
pub const GAS_FT_TRANSFER: Gas = Gas(2_000_000_000_000);

pub const BALANCE_PROPOSAL_BOND: Balance = 100_000_000_000_000_000_000_000; // 0.1 Near
pub const FT_TRANSFER_BOND: Balance = ONE_YOCTO;
