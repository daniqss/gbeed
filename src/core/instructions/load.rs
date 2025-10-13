use crate::core::{
    instructions::{InstructionEffect, InstructionError, InstructionResult},
    memory::{INTERRUPT_ENABLE_REGISTER, IO_REGISTERS_START},
};

/// copy the value stored in A src register to dst register
pub fn ld_r8_r8(dst: &mut u8, src: u8) -> InstructionEffect {
    *dst = src;
    InstructionEffect::new(1, 1, None)
}

/// copy immediate value into dst register
pub fn ld_r8_n8(dst: &mut u8, src: u8) -> InstructionEffect {
    *dst = src;
    InstructionEffect::new(2, 2, None)
}

/// copy 16 bits immediate value into dst pair of registers
pub fn ld_r16_n16(dst: &mut u16, src: u16) -> InstructionEffect {
    *dst = src;
    InstructionEffect::new(3, 3, None)
}

/// copy the value stored in A src register to the memory addressed by hl register pair
pub fn ld_hl_r8(dst: &mut u8, src: u8) -> InstructionEffect {
    *dst = src;
    InstructionEffect::new(2, 1, None)
}

/// copy immediate value into the memory addressed by hl register pair
pub fn ld_hl_n8(dst: &mut u8, src: u8) -> InstructionEffect {
    *dst = src;
    InstructionEffect::new(3, 2, None)
}

/// copy the value pointed by hl into A dst register
pub fn ld_r8_hl(dst: &mut u8, src: u8) -> InstructionEffect {
    *dst = src;
    InstructionEffect::new(2, 1, None)
}

/// copy the value in src register A to the memory pointed by the dst 16 bits pair of registers
pub fn ld_r8_a(dst: &mut u8, src: u8) -> InstructionEffect {
    *dst = src;
    InstructionEffect::new(2, 1, None)
}

/// copy the src value in register A to the memory pointed by the 16 bits immediate value
pub fn ld_n16_a(dst: &mut u8, src: u8) -> InstructionEffect {
    *dst = src;
    InstructionEffect::new(4, 3, None)
}

/// copy the src value in register A to the memory pointed by the 16 bits immediate value
/// this address is between 0xFF00 and 0xFFFF (memory mapped IO and HRAM)
pub fn ldh_n16_a(address: u16, dst: &mut u8, src: u8) -> InstructionResult {
    match address {
        IO_REGISTERS_START..=INTERRUPT_ENABLE_REGISTER => {
            *dst = src;
            Ok(InstructionEffect::new(3, 2, None))
        }
        _ => Err(InstructionError::AddressOutOfRange(address, None, None)),
    }
}
