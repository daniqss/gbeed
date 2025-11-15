use std::fmt::Write;

use crate::{
    core::cpu::{
        R8, R16,
        flags::{Flags, check_overflow_cy, check_overflow_hc, check_zero},
        instructions::{
            Instruction, InstructionDestination as ID, InstructionEffect, InstructionError,
            InstructionResult, InstructionTarget as IT,
        },
    },
    utils::{high, low, to_u16, with_u16},
};

/// Add instruction
pub struct Add<'a> {
    dst: ID<'a>,
    addend: IT<'a>,
}

impl<'a> Add<'a> {
    pub fn new(dst: ID<'a>, addend: IT<'a>) -> Box<Self> { Box::new(Add { dst, addend }) }
}

impl<'a> Instruction<'a> for Add<'a> {
    fn exec(&mut self) -> InstructionResult {
        let (dst, addend, cycles, len): (&mut u8, u8, u8, u8) = match (&mut self.dst, &self.addend) {
            (ID::Reg8(a, _), IT::Reg8(r8, reg)) if *reg != R8::F => (a, *r8, 1, 1),
            (ID::Reg8(a, _), IT::Imm8(n8)) => (a, *n8, 2, 2),
            (ID::Reg8(a, _), IT::PointedByHL(val)) => (a, *val, 2, 1),
            (ID::Reg16(hl, R16::HL), IT::Reg16(r16, src_reg)) if *src_reg != R16::HL => {
                with_u16(hl.0, hl.1, |hl| hl.wrapping_add(*r16));
                let flags = Flags {
                    z: None,
                    n: Some(false),
                    h: Some(check_overflow_hc(*hl.1, high(*r16))),
                    c: Some(check_overflow_cy(*hl.1, high(*r16))),
                };

                return Ok(InstructionEffect::new(2, 1, flags));
            }
            (ID::Reg16(hl, dst_reg), IT::StackPointer(sp)) if *dst_reg == R16::HL => {
                with_u16(hl.0, hl.1, |hl| hl.wrapping_add(*sp));
                let flags = Flags {
                    z: None,
                    n: Some(false),
                    h: Some(check_overflow_hc(*hl.1, high(*sp))),
                    c: Some(check_overflow_cy(*hl.1, high(*sp))),
                };

                return Ok(InstructionEffect::new(2, 1, flags));
            }
            (ID::StackPointer(sp), IT::SignedImm(e8)) => {
                let result = sp.wrapping_add(*e8 as u16);
                let flags = Flags {
                    z: Some(false),
                    n: Some(false),
                    h: Some(check_overflow_hc(low(result), low(**sp))),
                    c: Some(check_overflow_cy(low(result), low(**sp))),
                };

                **sp = result;

                return Ok(InstructionEffect::new(4, 2, flags));
            }

            _ => return Err(InstructionError::MalformedInstruction),
        };

        // perform the addition for most of the cases
        let result = dst.wrapping_add(addend);
        let flags = Flags {
            z: Some(check_zero(result)),
            n: Some(false),
            h: Some(check_overflow_hc(result, *dst)),
            c: Some(check_overflow_cy(result, *dst)),
        };
        *dst = result;

        Ok(InstructionEffect::new(cycles, len, flags))
    }

    fn disassembly(&self, w: &mut dyn Write) -> Result<(), std::fmt::Error> { write!(w, "add, ") }
}
