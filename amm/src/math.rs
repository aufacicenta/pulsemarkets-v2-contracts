#![allow(clippy::ptr_offset_with_cast, clippy::assign_op_pattern)]

use uint::construct_uint;
construct_uint! {
    /// 256-bit unsigned integer.
    pub struct u256(4);
}


/** 
 * @notice complex multiplication function that takes decimals into account, e.g. if a token has 18 decimals:
 *         5e17 * 1e18 = 5e17 -> 0.5 * 1 = 0.5
*/
pub fn complex_mul_u128(base: u128, a: u128, b: u128) -> u128 {
    // Convert all parameters to u256 to prevent overflow
    let a_u256 = u256::from(a);
    let b_u256 = u256::from(b);
    let base_u256 = u256::from(base);

    // Complex mul
    let c0 = a_u256 * b_u256;
    let c1 = c0 + (base_u256 / 2);
    (c1 / base_u256).as_u128()
}

/** 
 * @notice complex division function that takes decimals into account, e.g. if a token has 18 decimals:
 *         1e18 / 2e18 = 5e17 -> 1 / 2 = 0.5
*/
pub fn complex_div_u128(base: u128, a: u128, b: u128) -> u128 {
    // Convert all parameters to u256 to prevent overflow
    let a_u256 = u256::from(a);
    let b_u256 = u256::from(b);
    let base_u256 = u256::from(base);

    // Complex div
    let c0 = a_u256 * base_u256;
    let c1 = c0 + (b_u256 / 2);
    (c1 / b_u256).as_u128()
}

/**
 * @notice rust native mul but for larger numbers
 */
pub fn simple_mul_u128(base: u128, a: u128, b: u128) -> u128 {
    // Convert all parameters to u256 to prevent overflow
    let a_u256 = u256::from(a);
    let b_u256 = u256::from(b);
    let base_u256 = u256::from(base);

    (a_u256 * b_u256 / base_u256).as_u128()
}

/**
 * @notice rust native division but for larger numbers
 */
pub fn simple_div_u128(base: u128, a: u128, b: u128) -> u128 {
    // Convert all parameters to u256 to prevent overflow
    let a_u256 = u256::from(a);
    let b_u256 = u256::from(b);
    let base_u256 = u256::from(base);

    (a_u256 * base_u256 / b_u256).as_u128()
}