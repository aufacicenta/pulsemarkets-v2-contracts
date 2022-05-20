
pub fn complex_mul_u128(base: u128, a: u128, b: u128) -> u128 {
    let new_a = a / base;
    let new_b = b / base;
    new_a * new_b * base
}

pub fn complex_div_u128(base: u128, a: u128, b: u128) -> u128 {
    let new_a = a / base;
    let new_b = b / base;
    new_a / new_b * base
}
