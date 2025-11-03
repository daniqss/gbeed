use std::fmt::Write;

use crate::{
    core::cpu::{
        R8, R16,
        flags::{SUBTRACTION_FLAG_MASK, ZERO_FLAG_MASK, check_carry, check_half_carry, check_zero},
        instructions::{
            Instruction, InstructionDestination as ID, InstructionEffect, InstructionError, InstructionResult,
            InstructionTarget as IT,
        },
    },
    utils::{high, low, to_u16, with_u16},
};

/// Add instruction
pub struct ADD<'a> {
    dst: ID<'a>,
    addend: IT<'a>,
}

impl<'a> ADD<'a> {
    pub fn new(dst: ID<'a>, addend: IT<'a>) -> Box<Self> { Box::new(ADD { dst, addend }) }
}

impl<'a> Instruction<'a> for ADD<'a> {
    fn exec(&mut self) -> InstructionResult {
        let (dst, addend, cycles, len): (&mut u8, u8, u8, u8) = match (&mut self.dst, &self.addend) {
            (ID::Register8(a, _), IT::Register8(r8, reg)) if *reg != R8::F => (a, *r8, 1, 1),
            (ID::Register8(a, _), IT::Immediate8(n8)) => (a, *n8, 2, 2),
            (ID::Register8(a, _), IT::PointedByHL(val)) => (a, *val, 2, 1),
            (ID::Register16(hl, dst_reg), IT::Register16(r16, src_reg))
                if *dst_reg == R16::HL && *src_reg != R16::HL =>
            {
                with_u16(hl.1, hl.0, |hl| hl.wrapping_add(to_u16(r16.1, r16.0)));
                let flags = check_carry(*hl.1, r16.1) | check_half_carry(*hl.1, r16.1);
                return Ok(InstructionEffect::new(2, 1, Some(flags)));
            }
            (ID::Register16(hl, dst_reg), IT::StackPointer(sp)) if *dst_reg == R16::HL => {
                with_u16(hl.1, hl.0, |hl| hl.wrapping_add(*sp));
                let flags = check_carry(*hl.1, high(*sp)) | check_half_carry(*hl.1, high(*sp));
                return Ok(InstructionEffect::new(2, 1, Some(flags)));
            }
            (ID::StackPointer(sp), IT::SignedImm(e8)) => {
                let result = sp.wrapping_add(*e8 as u16);

                let flags = check_carry(low(result), low(**sp)) | check_half_carry(low(result), low(**sp));
                let flags = flags & !ZERO_FLAG_MASK | !SUBTRACTION_FLAG_MASK;
                **sp = result;

                return Ok(InstructionEffect::new(4, 2, Some(flags)));
            }

            _ => return Err(InstructionError::MalformedInstruction),
        };

        // perform the addition for most of the cases
        let result = dst.wrapping_add(addend);
        let flags = check_zero(result) | check_carry(result, *dst) | check_half_carry(result, *dst);
        *dst = result;

        Ok(InstructionEffect::new(len, cycles, Some(flags)))
    }

    fn disassembly(&self, w: &mut dyn Write) -> Result<(), std::fmt::Error> { write!(w, "add, ") }
}
