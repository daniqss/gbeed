use std::fmt::Write;

use crate::{
    Dmg,
    core::{
        cpu::{
            flags::{Flags, check_borrow_hc, check_zero},
            instructions::{
                Instruction, InstructionDestination as ID, InstructionEffect, InstructionError,
                InstructionResult,
            },
            registers::{Reg8 as Reg, Reg16 as Reg},
        },
        memory::Accessable,
    },
};

/// decrement the dst value by one
pub struct Dec {
    dst: ID,
}

impl Dec {
    pub fn new(dst: ID) -> Box<Self> { Box::new(Dec { dst }) }
}

impl Instruction for Dec {
    fn exec(&mut self, gb: &mut Dmg) -> InstructionResult {
        let len = 1;

        let (dst, cycles): (&mut u8, u8) = match &mut self.dst {
            ID::Reg8(reg) if *reg != Reg::F => (&mut gb[&*reg], 1),
            ID::PointedByHL(addr) => (&mut gb[*addr], 3),
            ID::Reg16(reg) if *reg != Reg::AF => {
                let r16 = gb.load(&*reg);
                gb.store(&*reg, r16.wrapping_sub(1));

                return Ok(InstructionEffect::new(2, len, Flags::none()));
            }
            ID::StackPointer => {
                gb.cpu.sp = gb.cpu.sp.wrapping_sub(1);

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
