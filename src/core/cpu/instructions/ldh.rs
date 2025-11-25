use std::fmt::Write;

use super::{
    InstructionDestination as ID, InstructionEffect, InstructionError, InstructionResult,
    InstructionTarget as IT,
};
use crate::{
    Dmg,
    core::{
        IO_REGISTERS_START,
        cpu::{R8, flags::Flags, instructions::Instruction},
        memory::is_high_address,
    },
};

/// Load from/to high memory area instruction
/// Usually used to access memory mapped IO and HRAM,
/// so the used addresses are between 0xFF00 and 0xFFFF
pub struct Ldh {
    dst: ID,
    src: IT,
}

impl Ldh {
    pub fn new(dst: ID, src: IT) -> Box<Self> { Box::new(Self { dst, src }) }
}

impl Instruction for Ldh {
    fn exec(&mut self, gb: &mut Dmg) -> InstructionResult {
        let (dst, src, addr, cycles, len): (&mut u8, u8, Option<u16>, u8, u8) = match (&self.dst, &self.src) {
            // copy the src value in register A to the byte at 16 bits immediate address (that must be between 0xFF00 and 0xFFFF)
            (ID::PointedByN16(addr), IT::Reg8(src, reg)) if *reg == R8::A => {
                (&mut gb[*addr], *src, Some(*addr), 3, 2)
            }
            // copy the src value in register A to the byte at address 0xFF00 + value in register C
            (ID::PointedByCPlusFF00(addr), IT::Reg8(src, reg)) if *reg == R8::A => {
                (&mut gb[*addr], *src, Some(*addr + IO_REGISTERS_START), 2, 1)
            }
            // copy the src byte addressed by 16 bits immediate (that must be between 0xFF00 and 0xFFFF) into dst register A
            (ID::Reg8(reg), IT::PointedByN16(src, addr)) if *reg == R8::A => {
                (&mut gb[reg], *src, Some(*addr), 3, 2)
            }
            // copy the src byte addressed by 0xFF00 + C into dst register A
            (ID::Reg8(reg), IT::PointedByCPlusFF00(src, addr)) if *reg == R8::A => {
                (&mut gb[reg], *src, Some(*addr + IO_REGISTERS_START), 2, 1)
            }

            _ => return Err(InstructionError::MalformedInstruction),
        };

        // if the destination target is a memory byte, check if the address is in range
        // otherwise return an error
        if let Some(addr) = addr
            && !is_high_address(addr)
        {
            return Err(InstructionError::AddressOutOfRange(addr, None, None));
        }

        *dst = src;

        Ok(InstructionEffect::new(cycles, len, Flags::none()))
    }

    fn disassembly(&self, w: &mut dyn Write) -> Result<(), std::fmt::Error> {
        write!(w, "ldh {},{}", self.dst, self.src)
    }
}
