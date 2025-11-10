/// # Flag masks
/// The F register use its 4 most significant bits to store information about the result of the previous operation
pub const ZERO_FLAG_MASK: u8 = 0b1000_0000;
pub const SUBTRACTION_FLAG_MASK: u8 = 0b0100_0000;
pub const HALF_CARRY_FLAG_MASK: u8 = 0b0010_0000;
pub const CARRY_FLAG_MASK: u8 = 0b0001_0000;

#[derive(Debug, PartialEq)]
pub struct Flags {
    pub z: Option<bool>,
    pub n: Option<bool>,
    pub h: Option<bool>,
    pub c: Option<bool>,
}

impl Flags {
    pub fn none() -> Self {
        Self {
            z: None,
            n: None,
            h: None,
            c: None,
        }
    }
}

#[inline]
pub fn check_zero(result: u8) -> bool { result == 0 }

#[inline]
pub fn check_overflow_hc(result: u8, old: u8) -> bool { (result & 0x0F) < (old & 0x0F) }
/// Check borrow in bit 4
#[inline]
pub fn check_borrow_hc(old: u8, substrahend: u8) -> bool { (old & 0x0F) < (substrahend & 0x0F) }

#[inline]
pub fn check_overflow_cy(result: u8, old: u8) -> bool { result < old }
