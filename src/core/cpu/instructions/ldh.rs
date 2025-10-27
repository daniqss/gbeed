use super::{InstructionEffect, InstructionError, InstructionResult, InstructionTarget};
use crate::core::memory::{INTERRUPT_ENABLE_REGISTER, IO_REGISTERS_START};

// /// copy the src byte addressed by a 16 bits immediate value
// /// copy the src value in register A to the memory pointed by the 16 bits immediate value
// /// this address is between 0xFF00 and 0xFFFF (memory mapped IO and HRAM)
// pub fn ldh_n16_a(address: u16, dst: &mut u8, src: u8) -> InstructionResult {
//     match address {
//         IO_REGISTERS_START..=INTERRUPT_ENABLE_REGISTER => {
//             *dst = src;
//             Ok(InstructionEffect::new(3, 2, None))
//         }
//         _ => Err(InstructionError::AddressOutOfRange(address, None, None)),
//     }
// }

// /// copy the src value in register A to the byte at address 0xFF00 + value in register C
// /// sometimes written as `LD [$FF00+C],A`
// pub fn ldh_c_a(dst: &mut u8, src: u8) -> InstructionEffect {
//     *dst = src;
//     InstructionEffect::new(2, 1, None)
// }

// /// (that must be between 0xFF00 and 0xFFFF),
// /// into dst register a
// pub fn ldh_a_n16(address: u16, dst: &mut u8, src: u8) -> InstructionResult {
//     match address {
//         IO_REGISTERS_START..=INTERRUPT_ENABLE_REGISTER => {
//             *dst = src;
//             Ok(InstructionEffect::new(3, 2, None))
//         }
//         _ => Err(InstructionError::AddressOutOfRange(address, None, None)),
//     }
// }

// /// copy the src byte addressed by 0xFF00 + C into dst register A
// /// sometimes written as `LD A,[$FF00+C]`
// pub fn ldh_a_c(dst: &mut u8, src: u8) -> InstructionEffect {
//     *dst = src;
//     InstructionEffect::new(2, 1, None)
// }

enum Destination<'a> {
    PointedByN16(&'a mut u8, u16),
    PointedByCPlusFF00(&'a mut u8, u16),
    RegisterA(&'a mut u8),
}

enum Source {
    RegisterA(u8),
    PointedByN16(u8, u16),
    PointedByCPlusFF00(u8, u16),
}

fn is_high_address(address: u16) -> bool {
    address >= IO_REGISTERS_START && address <= INTERRUPT_ENABLE_REGISTER
}

/// Load from/to high memory area instruction
/// Usually used to access memory mapped IO and HRAM,
/// so the used addresses are between 0xFF00 and 0xFFFF
pub struct LDH<'a> {
    dst: Destination<'a>,
    src: Source,
}

impl<'a> LDH<'a> {
    pub fn new(src: Source, dst: Destination<'a>) -> Self { LDH { src, dst } }

    pub fn exec(src: Source, dst: Destination<'a>) -> InstructionResult {
        let (dst, src, address, cycles, len) = match (dst, src) {
            (Destination::PointedByN16(dst, address), Source::RegisterA(src)) => {
                (dst, src, Some(address), 3, 2)
            }
            (Destination::PointedByCPlusFF00(dst, address), Source::RegisterA(src)) => {
                (dst, src, Some(address), 2, 1)
            }
            (Destination::RegisterA(dst), Source::PointedByN16(src, address)) => {
                (dst, src, Some(address), 3, 2)
            }

            (Destination::RegisterA(dst), Source::PointedByCPlusFF00(src, address)) => {
                (dst, src, Some(address), 2, 1)
            }

            _ => return Err(InstructionError::MalformedInstruction),
        };

        // if the destination target is a memory byte, check if the address is in range
        // otherwise return an error
        if let Some(addr) = address
            && !is_high_address(addr)
        {
            return Err(InstructionError::AddressOutOfRange(addr, None, None));
        }

        *dst = src;

        Ok(InstructionEffect::new(cycles, len, None))
    }
}

impl std::fmt::Display for LDH<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "LDH {},{}",
            match self.dst {
                Destination::RegisterA(_) => "A".to_string(),
                Destination::PointedByN16(_, address) if is_high_address(address) =>
                    format!("[${:04X}]", address),
                // sometimes written as `LD [C+$FF00],A`
                Destination::PointedByCPlusFF00(_, address) if is_high_address(address) =>
                    "[C]".to_string(),
                _ => return Err(std::fmt::Error),
                // _ => return Err(InstructionError::AddressOutOfRange(addr, None, None)),
            },
            match self.src {
                Source::RegisterA(_) => "A".to_string(),
                Source::PointedByN16(_, address) if is_high_address(address) =>
                    format!("[${:04X}]", address),
                Source::PointedByCPlusFF00(_, address) if is_high_address(address) =>
                // sometimes written as `LD A,[C+$FF00]`
                    "[C]".to_string(),
                _ => return Err(std::fmt::Error),
                // _ => return Err(InstructionError::AddressOutOfRange(addr, None, None)),
            },
        )
    }
}
