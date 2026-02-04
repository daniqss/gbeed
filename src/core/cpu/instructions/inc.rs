use std::fmt::Write;

use crate::{
    Dmg,
    core::{
        cpu::{
            flags::{Flags, check_overflow_hc, check_zero},
            instructions::{
                Instruction, InstructionDestination as ID, InstructionEffect, InstructionError,
                InstructionResult,
            },
            {R8, R16},
        },
        memory::{Accessible, Accessible16},
    },
};

/// increment the dst value by one
pub struct Inc {
    dst: ID,
}

impl Inc {
    pub fn new(dst: ID) -> Box<Self> { Box::new(Inc { dst }) }
}

impl Instruction for Inc {
    fn exec(&mut self, gb: &mut Dmg) -> InstructionResult {
        let len = 1;

        let (dst, cycles): (&mut u8, u8) = match &mut self.dst {
            ID::Reg8(reg) if *reg != R8::F => (&mut gb[&*reg], 1),
            ID::PointedByHL(addr) => (&mut gb[*addr], 3),
            ID::Reg16(reg) if *reg != R16::AF => {
                let r16 = gb.load(&*reg);
                gb.write(&*reg, r16.wrapping_add(1));

                return Ok(InstructionEffect::new(2, len, Flags::none()));
            }
            ID::StackPointer => {
                gb.cpu.sp = gb.cpu.sp.wrapping_add(1);

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
