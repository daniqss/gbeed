use crate::impl_static_box;

/// # Flag masks
/// The F register use its 4 most significant bits to store information about the result of the previous operation
pub const ZERO_FLAG_MASK: u8 = 0b1000_0000;
pub const SUBTRACTION_FLAG_MASK: u8 = 0b0100_0000;
pub const HALF_CARRY_FLAG_MASK: u8 = 0b0010_0000;
pub const CARRY_FLAG_MASK: u8 = 0b0001_0000;

pub const ALL_FLAGS_MASK: u8 =
    ZERO_FLAG_MASK | SUBTRACTION_FLAG_MASK | HALF_CARRY_FLAG_MASK | CARRY_FLAG_MASK;

#[inline(always)]
pub fn check_zero(result: u8) -> bool { result == 0 }

#[inline(always)]
pub fn check_overflow_hc(result: u8, old: u8) -> bool { (result & 0x0F) < (old & 0x0F) }
/// Check borrow in bit 4
#[inline(always)]
pub fn check_borrow_hc(old: u8, substrahend: u8) -> bool { (old & 0x0F) < (substrahend & 0x0F) }

#[inline(always)]
pub fn check_overflow_cy(result: u8, old: u8) -> bool { result < old }

#[inline(always)]
pub fn check_overflow_hc16(result: u16, old: u16) -> bool { (result & 0x0FFF) < (old & 0x0FFF) }

#[inline(always)]
pub fn check_overflow_cy16(result: u16, old: u16) -> bool { result < old }

/// # Lazy Flags
/// A trait that can be implemented by instructions to compute flags lazily, only when they are needed.
/// Instructions will return their `LazyFlags` implementation in their `InstructionEffect`
/// The flags will be computed when the CPU needs to read them, for example during a conditional jump.
pub trait LazyFlags: core::fmt::Debug {
    fn zero(&self) -> bool { false }
    fn subtraction(&self) -> bool { false }
    fn half_carry(&self) -> bool { false }
    fn carry(&self) -> bool { false }

    fn updated_flags(&self) -> u8;
}

impl_static_box!(LazyFlags);

#[derive(Debug, Clone, Copy)]
pub struct ConstantFlags {
    zero: bool,
    subtraction: bool,
    half_carry: bool,
    carry: bool,
    byte: u8,
}

impl ConstantFlags {
    pub fn new(byte: u8) -> Self {
        Self {
            zero: byte & ZERO_FLAG_MASK != 0,
            subtraction: byte & SUBTRACTION_FLAG_MASK != 0,
            half_carry: byte & HALF_CARRY_FLAG_MASK != 0,
            carry: byte & CARRY_FLAG_MASK != 0,
            byte,
        }
    }
}

impl LazyFlags for ConstantFlags {
    fn zero(&self) -> bool { self.zero }
    fn subtraction(&self) -> bool { self.subtraction }
    fn half_carry(&self) -> bool { self.half_carry }
    fn carry(&self) -> bool { self.carry }
    fn updated_flags(&self) -> u8 { self.byte & ALL_FLAGS_MASK }
}
