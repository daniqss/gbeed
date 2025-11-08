use std::fmt::Write;

use crate::{
    core::cpu::{
        flags::{check_overflow_hc, check_zero},
        instructions::{
            Instruction, InstructionDestination as ID, InstructionEffect, InstructionError, InstructionResult,
        },
        registers::{Register8 as R8, Register16 as R16},
    },
    utils::with_u16,
};

/// increment the dst value by one
pub struct Inc<'a> {
    dst: ID<'a>,
}

impl<'a> Inc<'a> {
    pub fn new(dst: ID<'a>) -> Box<Self> { Box::new(Inc { dst }) }
}

impl<'a> Instruction<'a> for Inc<'a> {
    fn exec(&mut self) -> InstructionResult {
        let len = 1;

        let (dst, cycles): (&mut u8, u8) = match &mut self.dst {
            ID::Register8(dst, reg) if *reg != R8::F => (*dst, 1),
            ID::PointedByHL(bus, addr) => (&mut bus.borrow_mut()[*addr], 3),
            ID::Register16(dst, reg) if *reg != R16::AF => {
                with_u16(dst.1, dst.0, |val| val.wrapping_add(1));

                return Ok(InstructionEffect::new(2, len, None));
            }
            ID::StackPointer(dst) => {
                **dst = dst.wrapping_add(1);

                return Ok(InstructionEffect::new(2, len, None));
            }

            _ => return Err(InstructionError::MalformedInstruction),
        };

        let result = dst.wrapping_add(1);
        let flags = check_zero(result) | check_overflow_hc(*dst, 1);
        *dst = result;

        Ok(InstructionEffect::new(cycles, len, Some(flags)))
    }

    fn disassembly(&self, w: &mut dyn Write) -> Result<(), std::fmt::Error> { write!(w, "dec {}", self.dst) }
}
