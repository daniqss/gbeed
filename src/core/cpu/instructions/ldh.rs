use std::fmt::Write;

use super::{
    InstructionDestination as ID, InstructionEffect, InstructionError, InstructionResult,
    InstructionTarget as IT,
};
use crate::core::{
    cpu::{R8, instructions::Instruction},
    memory::is_high_address,
};

/// Load from/to high memory area instruction
/// Usually used to access memory mapped IO and HRAM,
/// so the used addresses are between 0xFF00 and 0xFFFF
pub struct LDH<'a> {
    dst: ID<'a>,
    src: IT<'a>,
}

impl<'a> LDH<'a> {
    pub fn new(dst: ID<'a>, src: IT<'a>) -> Box<Self> { Box::new(Self { dst, src }) }
}

impl<'a> Instruction<'a> for LDH<'a> {
    fn exec(&mut self) -> InstructionResult {
        let (dst, src, address, cycles, len): (&mut u8, u8, Option<u16>, u8, u8) =
            match (&mut self.dst, &self.src) {
                // copy the src value in register A to the byte at 16 bits immediate address (that must be between 0xFF00 and 0xFFFF)
                (ID::PointedByN16(bus, address), IT::Register8(src, reg)) if *reg == R8::A => {
                    (&mut bus.borrow_mut()[*address], *src, Some(*address), 3, 2)
                }
                // copy the src value in register A to the byte at address 0xFF00 + value in register C
                (ID::PointedByCPlusFF00(bus, address), IT::Register8(src, reg)) if *reg == R8::A => {
                    (&mut bus.borrow_mut()[*address], *src, Some(*address), 2, 1)
                }
                // copy the src byte addressed by 16 bits immediate (that must be between 0xFF00 and 0xFFFF) into dst register A
                (ID::Register8(dst, reg), IT::PointedByN16(src, address)) if *reg == R8::A => {
                    (dst, *src, Some(*address), 3, 2)
                }
                // copy the src byte addressed by 0xFF00 + C into dst register A
                (ID::Register8(dst, reg), IT::PointedByCPlusFF00(src, address)) if *reg == R8::A => {
                    (dst, *src, Some(*address), 2, 1)
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

    fn disassembly(&self, w: &mut dyn Write) -> Result<(), std::fmt::Error> {
        write!(w, "ldh {},{}", self.dst, self.src)
    }
}
