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
pub const GAS_AGGREGATOR_READ: Gas = Gas(8_000_000_000_000);

pub const BALANCE_PROPOSAL_BOND: Balance = 100_000_000_000_000_000_000_000; // 0.1 Near
pub const FT_TRANSFER_BOND: Balance = ONE_YOCTO;

pub const FORMATTED_STRING_LOCALE: Locale = Locale::en;

pub const FEED_PARSER_V2_MAINNET: &str = "feed-parser.pulsemarkets.near";
#[cfg(not(near_env = "testnet"))]
pub const FEED_PARSER_ACCOUNT_ID: &str = FEED_PARSER_V2_MAINNET;

pub const FEED_PARSER_V2_TESTNET: &str = "feed-1.pulsemarkets.testnet";
#[cfg(near_env = "testnet")]
pub const FEED_PARSER_ACCOUNT_ID: &str = FEED_PARSER_V2_TESTNET;
