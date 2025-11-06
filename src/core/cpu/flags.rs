/// # Flag masks
/// The F register use its 4 most significant bits to store information about the result of the previous operation
pub const ZERO_FLAG_MASK: u8 = 0b1000_0000;
pub const SUBTRACTION_FLAG_MASK: u8 = 0b0100_0000;
pub const HALF_CARRY_FLAG_MASK: u8 = 0b0010_0000;
pub const CARRY_FLAG_MASK: u8 = 0b0001_0000;

#[inline]
pub fn check_zero(result: u8) -> u8 { if result == 0 { ZERO_FLAG_MASK } else { 0 } }

#[inline]
pub fn check_overflow_hc(result: u8, old: u8) -> u8 {
    if (result & 0x0F) < (old & 0x0F) {
        HALF_CARRY_FLAG_MASK
    } else {
        0
    }
}
/// Check borrow in bit 4
#[inline]
pub fn check_borrow_hc(result: u8, substrahend: u8) -> u8 {
    if (result & 0x0F) < (substrahend & 0x0F) {
        HALF_CARRY_FLAG_MASK
    } else {
        0
    }
}

#[inline]
pub fn check_overflow_cy(result: u8, old: u8) -> u8 { if result < old { CARRY_FLAG_MASK } else { 0 } }
/// check borrow
#[inline]
pub fn check_borrow_cy(old: u8, subtrahend: u8) -> u8 { if subtrahend > old { CARRY_FLAG_MASK } else { 0 } }
