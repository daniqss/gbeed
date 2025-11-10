use std::fmt::Write;

use crate::{
    core::cpu::{
        flags::{Flags, check_borrow_hc, check_zero},
        instructions::{
            Instruction, InstructionDestination as ID, InstructionEffect, InstructionError, InstructionResult,
        },
        registers::{Register8 as R8, Register16 as R16},
    },
    utils::with_u16,
};

/// decrement the dst value by one
pub struct Dec<'a> {
    dst: ID<'a>,
}

impl<'a> Dec<'a> {
    pub fn new(dst: ID<'a>) -> Box<Self> { Box::new(Dec { dst }) }
}

impl<'a> Instruction<'a> for Dec<'a> {
    fn exec(&mut self) -> InstructionResult {
        let len = 1;

        let (dst, cycles): (&mut u8, u8) = match &mut self.dst {
            ID::Register8(dst, reg) if *reg != R8::F => (*dst, 1),
            ID::PointedByHL(bus, addr) => (&mut bus.borrow_mut()[*addr], 3),
            ID::Register16(dst, reg) if *reg != R16::AF => {
                with_u16(dst.1, dst.0, |val| val.wrapping_sub(1));

                return Ok(InstructionEffect::new(2, len, Flags::none()));
            }
            ID::StackPointer(dst) => {
                **dst = dst.wrapping_sub(1);

                return Ok(InstructionEffect::new(2, len, Flags::none()));
            }

            _ => return Err(InstructionError::MalformedInstruction),
        };

        let result = dst.wrapping_sub(1);
        let flags = Flags {
            z: Some(check_zero(result)),
            n: Some(true),
            h: Some(check_borrow_hc(*dst, 1)),
            c: None,
        };
        *dst = result;

        Ok(InstructionEffect::new(cycles, len, flags))
    }

    fn disassembly(&self, w: &mut dyn Write) -> Result<(), std::fmt::Error> { write!(w, "dec {}", self.dst) }
}
