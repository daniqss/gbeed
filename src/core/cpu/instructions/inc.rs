use std::fmt::Write;

use crate::{
    core::cpu::{
        flags::{Flags, check_overflow_hc, check_zero},
        instructions::{
            Instruction, InstructionDestination as ID, InstructionEffect, InstructionError, InstructionResult,
        },
        registers::{Reg8 as R8, Reg16 as R16},
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
            ID::Reg8(dst, reg) if *reg != R8::F => (*dst, 1),
            ID::PointedByHL(bus, addr) => (&mut bus.borrow_mut()[*addr], 3),
            ID::Reg16(dst, reg) if *reg != R16::AF => {
                with_u16(dst.0, dst.1, |val| val.wrapping_add(1));

                return Ok(InstructionEffect::new(2, len, Flags::none()));
            }
            ID::StackPointer(dst) => {
                **dst = dst.wrapping_add(1);

                return Ok(InstructionEffect::new(2, len, Flags::none()));
            }

            _ => return Err(InstructionError::MalformedInstruction),
        };

        let result = dst.wrapping_add(1);
        let flags = Flags {
            z: Some(check_zero(result)),
            n: Some(false),
            h: Some(check_overflow_hc(result, *dst)),
            c: None,
        };
        *dst = result;

        Ok(InstructionEffect::new(cycles, len, flags))
    }

    fn disassembly(&self, w: &mut dyn Write) -> Result<(), std::fmt::Error> { write!(w, "dec {}", self.dst) }
}
