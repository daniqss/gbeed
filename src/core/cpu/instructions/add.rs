use std::fmt::Write;

use crate::{
    Dmg,
    core::{
        cpu::{
            R8, R16,
            flags::{Flags, check_overflow_cy, check_overflow_hc, check_zero},
            instructions::{
                Instruction, InstructionDestination as ID, InstructionEffect, InstructionError,
                InstructionResult, InstructionTarget as IT,
            },
        },
        memory::Accessable,
    },
    utils::{high, low},
};

/// Add instruction
pub struct Add {
    dst: ID,
    addend: IT,
}

impl Add {
    pub fn new(dst: ID, addend: IT) -> Box<Self> { Box::new(Add { dst, addend }) }
}

impl Instruction for Add {
    fn exec(&mut self, gb: &mut Dmg) -> InstructionResult {
        let (dst, addend, cycles, len): (&mut u8, u8, u8, u8) = match (&mut self.dst, &self.addend) {
            (ID::Reg8(R8::A), IT::Reg8(r8, reg)) if *reg != R8::F => (&mut gb.cpu.a, *r8, 1, 1),
            (ID::Reg8(R8::A), IT::Imm8(n8)) => (&mut gb.cpu.a, *n8, 2, 2),
            (ID::Reg8(R8::A), IT::PointedByHL(val)) => (&mut gb.cpu.a, *val, 2, 1),
            (ID::Reg16(R16::HL), IT::Reg16(r16, src_reg)) if *src_reg != R16::HL => {
                let hl = gb.cpu.hl();
                gb.write16(&R16::HL, hl + *r16);

                let flags = Flags {
                    z: None,
                    n: Some(false),
                    h: Some(check_overflow_hc(high(hl), high(*r16))),
                    c: Some(check_overflow_cy(high(hl), high(*r16))),
                };

                return Ok(InstructionEffect::new(2, 1, flags));
            }
            (ID::Reg16(R16::HL), IT::StackPointer(sp)) => {
                let hl = gb.cpu.hl();
                gb.write16(&R16::HL, hl + sp);
                let flags = Flags {
                    z: None,
                    n: Some(false),
                    h: Some(check_overflow_hc(high(hl), high(*sp))),
                    c: Some(check_overflow_cy(high(hl), high(*sp))),
                };

                return Ok(InstructionEffect::new(2, 1, flags));
            }
            (ID::StackPointer, IT::SignedImm(e8)) => {
                let result = gb.cpu.sp.wrapping_add(*e8 as u16);
                let flags = Flags {
                    z: Some(false),
                    n: Some(false),
                    h: Some(check_overflow_hc(low(result), low(gb.cpu.sp))),
                    c: Some(check_overflow_cy(low(result), low(gb.cpu.sp))),
                };

                gb.cpu.sp = result;

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
