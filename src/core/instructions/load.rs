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

/// copy the src value in register to the byte at address 0xFF00 + value in register C
/// sometimes written as `LD [$FF00+C],A`
pub fn ldh_c_a(dst: &mut u8, src: u8) -> InstructionEffect {
    *dst = src;
    InstructionEffect::new(2, 1, None)
}

/// copy the src byte addressed by a pair of registers into dst register a
pub fn ld_a_r16(dst: &mut u8, src: u8) -> InstructionEffect {
    *dst = src;
    InstructionEffect::new(2, 1, None)
}

/// copy the src byte addressed by a 16 bits immediate value into dst register a
pub fn ld_a_n16(dst: &mut u8, src: u8) -> InstructionEffect {
    *dst = src;
    InstructionEffect::new(3, 2, None)
}

/// copy the src byte addressed by a 16 bits immediate value
/// (that must be between 0xFF00 and 0xFFFF),
/// into dst register a
pub fn ldh_a_n16(address: u16, dst: &mut u8, src: u8) -> InstructionResult {
    match address {
        IO_REGISTERS_START..=INTERRUPT_ENABLE_REGISTER => {
            *dst = src;
            Ok(InstructionEffect::new(3, 2, None))
        }
        _ => Err(InstructionError::AddressOutOfRange(address, None, None)),
    }
}

/// copy the src byte addressed by 0xFF00 + C into dst register A
/// sometimes written as `LD A,[$FF00+C]`
pub fn ldh_a_c(dst: &mut u8, src: u8) -> InstructionEffect {
    *dst = src;
    InstructionEffect::new(2, 1, None)
}

/// copy the src value in register A into the byte addressed by HL, then increment HL
/// sometimes written as `LD [HL+],A`, or `LDI [HL],A`
pub fn ld_hli_a(h: &mut u8, l: &mut u8, dst: &mut u8, src: u8) -> InstructionEffect {
    *dst = src;
    todo!("move instructions to impl Cpu");
    InstructionEffect::new(2, 2, None)
}
