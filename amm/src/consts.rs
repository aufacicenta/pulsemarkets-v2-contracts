use near_sdk::{Balance, Gas, ONE_YOCTO};
use num_format::Locale;

pub const GAS_CREATE_DAO_PROPOSAL: Gas = Gas(8_000_000_000_000);
pub const GAS_CREATE_DAO_PROPOSAL_CALLBACK: Gas = Gas(8_000_000_000_000);
pub const GAS_FT_TRANSFER: Gas = Gas(3_000_000_000_000);
pub const GAS_FT_BALANCE_OF: Gas = Gas(3_000_000_000_000);
pub const GAS_FT_BALANCE_OF_CALLBACK: Gas = Gas(3_000_000_000_000);
pub const GAS_FT_TRANSFER_CALLBACK: Gas = Gas(3_000_000_000_000);
pub const GAS_FT_TOTAL_SUPPLY: Gas = Gas(2_000_000_000_000);
pub const GAS_FT_TOTAL_SUPPLY_CALLBACK: Gas = Gas(2_000_000_000_000);
pub const GAS_FT_METADATA: Gas = Gas(2_000_000_000_000);
pub const GAS_FT_METADATA_CALLBACK: Gas = Gas(2_000_000_000_000);

pub const BALANCE_PROPOSAL_BOND: Balance = 100_000_000_000_000_000_000_000; // 0.1 Near
pub const FT_TRANSFER_BOND: Balance = ONE_YOCTO;

pub const FORMATTED_STRING_LOCALE: Locale = Locale::en;

/// Mainnet program id for Switchboard v2
pub const PULSE_V2_MAINNET: &str = "52a047ee205701895ee06a375492490ec9c597ce.factory.bridge.near";
#[cfg(not(feature = "testnet"))]
/// Switchboard Program ID.
pub const STAKING_TOKEN_ACCOUNT_ID: &str = PULSE_V2_MAINNET;

/// Devnet program id for Switchboard v2
pub const PULSE_V2_TESTNET: &str = "pulse.fakes.testnet";
#[cfg(feature = "testnet")]
/// Switchboard Program ID.
pub const STAKING_TOKEN_ACCOUNT_ID: &str = PULSE_V2_TESTNET;
