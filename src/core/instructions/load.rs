use crate::{
    core::{
        instructions::{InstructionEffect, InstructionError, InstructionResult},
        memory::{INTERRUPT_ENABLE_REGISTER, IO_REGISTERS_START},
    },
    utils::{to_u8, to_u16},
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

    let hl = to_u16(*h, *l).wrapping_add(1);
    (*h, *l) = to_u8(hl);

    InstructionEffect::new(2, 1, None)
}

/// copy the src value in register A into the byte addressed by HL, then decrement HL
/// sometimes written as `LD [HL-],A`, or `LDD [HL],A`
pub fn ld_hld_a(h: &mut u8, l: &mut u8, dst: &mut u8, src: u8) -> InstructionEffect {
    *dst = src;

    let hl = to_u16(*h, *l).wrapping_sub(1);
    (*h, *l) = to_u8(hl);

    InstructionEffect::new(2, 1, None)
}

/// copy the src byte addressed by HL into register A, then decrement HL
/// sometimes written as `LD A,[HL-]`, or `LDD A,[HL]
pub fn ld_a_hld(h: &mut u8, l: &mut u8, dst: &mut u8, src: u8) -> InstructionEffect {
    *dst = src;

    let hl = to_u16(*h, *l).wrapping_sub(1);
    (*h, *l) = to_u8(hl);

    InstructionEffect::new(2, 1, None)
}

/// copy the src byte addressed by HL into register A, then increment HL
/// sometimes written as `LD A,[HL+]`, or `LDI A,[HL]
pub fn ld_a_hli(h: &mut u8, l: &mut u8, dst: &mut u8, a: u8) -> InstructionEffect {
    *dst = a;

    let hl = to_u16(*h, *l).wrapping_add(1);
    (*h, *l) = to_u8(hl);

    InstructionEffect::new(2, 1, None)
}

/// copy the src value of a 16 bits immediate into SP register
pub fn ld_sp_n16(sp: &mut u16, src: u16) -> InstructionEffect {
    *sp = src;
    InstructionEffect::new(3, 3, None)
}

/// copy the srcs values of SP & 0xFF and SP >> 8 into the memory addressed by a 16 bits immediate value and its next byte
pub fn ld_n16_sp(dst_low: &mut u8, dst_high: &mut u8, src: u16) -> InstructionEffect {
    *dst_low = (src & 0x00FF) as u8;
    *dst_high = (src >> 8) as u8;
    InstructionEffect::new(5, 3, None)
}

/// add the 8 bit signed immediate to the SP register and store the result in HL register pair
/// half carries come from Z80 with binary coded decimal, that worked with nibbles (4 bits)
pub fn ld_hl_sp_plus_n8(h: &mut u8, l: &mut u8, sp: u16, e: i8) -> InstructionEffect {
    let result = sp.wrapping_add(e as i16 as u16);

    (*h, *l) = to_u8(result);

    let mut flags = 0;
    if ((sp & 0x0F) + ((e as u16) & 0x0F)) > 0x0F {
        flags |= 0x20;
    }
    if ((sp & 0xFF) + ((e as u16) & 0xFF)) > 0xFF {
        flags |= 0x10;
    }

    InstructionEffect::new(3, 3, Some(flags))
}

///copy the src pair of registers HL to the SP register
pub fn ld_sp_hl(sp: &mut u16, h: &mut u8, l: &u8) -> InstructionEffect {
    *sp = to_u16(*h, *l);
    InstructionEffect::new(2, 1, None)
}
