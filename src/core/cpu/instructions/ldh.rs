use std::fmt::Write;

use super::{InstructionEffect, InstructionError, InstructionResult, InstructionTarget as IT};
use crate::core::{cpu::instructions::Instruction, memory::is_high_address};

/// Load from/to high memory area instruction
/// Usually used to access memory mapped IO and HRAM,
/// so the used addresses are between 0xFF00 and 0xFFFF
pub struct LDH<'a> {
    dst: IT<'a>,
    src: IT<'a>,
}

impl<'a> LDH<'a> {
    pub fn new(dst: IT<'a>, src: IT<'a>) -> Self { LDH { dst, src } }
}

impl<'a> Instruction<'a> for LDH<'a> {
    fn exec(&mut self) -> InstructionResult {
        let (dst, src, address, cycles, len): (&mut u8, u8, Option<u16>, u8, u16) =
            match (&mut self.dst, &self.src) {
                // copy the src value in register A to the byte at 16 bits immediate address (that must be between 0xFF00 and 0xFFFF)
                (IT::DstPointedByN16(dst, address), &IT::RegisterA(src)) => {
                    (*dst, src, Some(*address), 3, 2)
                }
                // copy the src value in register A to the byte at address 0xFF00 + value in register C
                (IT::DstPointedByCPlusFF00(dst, address), &IT::RegisterA(src)) => {
                    (*dst, src, Some(*address), 2, 1)
                }
                // copy the src byte addressed by 16 bits immediate (that must be between 0xFF00 and 0xFFFF) into dst register A
                (IT::DstRegisterA(dst), &IT::PointedByN16(src, address)) => {
                    (*dst, src, Some(address), 3, 2)
                }
                // copy the src byte addressed by 0xFF00 + C into dst register A
                (IT::DstRegisterA(dst), &IT::PointedByCPlusFF00(src, address)) => {
                    (*dst, src, Some(address), 2, 1)
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

    fn disassembly(&self, w: &mut dyn Write) -> Result<(), InstructionError> {
        let dst_asm = match self.dst {
            IT::RegisterA(_) => "a".to_string(),
            IT::PointedByN16(_, address) => {
                if is_high_address(address) {
                    format!("[${:04X}]", address)
                } else {
                    return Err(InstructionError::AddressOutOfRange(address, None, None));
                }
            }
            // sometimes written as `LD [C+$FF00],A`
            IT::PointedByCPlusFF00(_, address) => {
                if is_high_address(address) {
                    "[c]".to_string()
                } else {
                    return Err(InstructionError::AddressOutOfRange(address, None, None));
                }
            }
            _ => return Err(InstructionError::MalformedInstruction),
        };

        let src_asm = match self.src {
            IT::RegisterA(_) => "a".to_string(),
            IT::PointedByN16(_, address) => {
                if is_high_address(address) {
                    format!("[${:04X}]", address)
                } else {
                    return Err(InstructionError::AddressOutOfRange(address, None, None));
                }
            }
            // sometimes written as `LD A,[C+$FF00]`
            IT::PointedByCPlusFF00(_, address) => {
                if is_high_address(address) {
                    "[c]".to_string()
                } else {
                    return Err(InstructionError::AddressOutOfRange(address, None, None));
                }
            }
            _ => return Err(InstructionError::MalformedInstruction),
        };

        write!(w, "ldh {},{}", dst_asm, src_asm).map_err(|_| InstructionError::MalformedInstruction)
    }
}
