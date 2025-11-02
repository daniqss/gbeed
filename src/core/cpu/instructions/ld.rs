use std::fmt::Write;

use crate::{
    core::cpu::{
        flags::{check_carry, check_half_carry},
        instructions::{
            Instruction, InstructionDestination as ID, InstructionEffect, InstructionError, InstructionResult,
            InstructionTarget as IT,
        },
        registers::{Register8 as R8, Register16 as R16},
    },
    utils::{low, to_u8, to_u16, with_u16},
};

pub struct LD<'a> {
    dst: ID<'a>,
    src: IT<'a>,
}

impl<'a> LD<'a> {
    pub fn new(dst: ID<'a>, src: IT<'a>) -> Box<Self> { Box::new(LD { dst, src }) }
}

impl<'a> Instruction<'a> for LD<'a> {
    fn exec(&mut self) -> InstructionResult {
        // handle cases where srcs are increased and decreased after load
        if let (ID::Register8(dst, reg), IT::PointedByHLD(src, hl)) = (&mut self.dst, &mut self.src) {
            if *reg != R8::A {
                return Err(InstructionError::MalformedInstruction);
            }

            **dst = *src;

            with_u16(hl.1, hl.0, |hl| hl.wrapping_add(1));
            return Ok(InstructionEffect::new(2, 1, None));
        }
        if let (ID::Register8(dst, reg), IT::PointedByHLI(src, hl)) = (&mut self.dst, &mut self.src) {
            if *reg != R8::A {
                return Err(InstructionError::MalformedInstruction);
            }

            **dst = *src;

            with_u16(hl.1, hl.0, |hl| hl.wrapping_sub(1));
            return Ok(InstructionEffect::new(2, 1, None));
        }

        // u8 loads
        let (dst, src, cycles, len): (&mut u8, u8, u8, u8) = match (&mut self.dst, &self.src) {
            (ID::Register8(dst, _), IT::Register8(src, _)) => (*dst, *src, 1, 1),
            (ID::Register8(dst, _), IT::Immediate8(src)) => (*dst, *src, 2, 2),
            (ID::Register16(dst, _), IT::Immediate16(src)) => {
                let (high, low) = to_u8(*src);
                *dst.0 = high;
                *dst.1 = low;

                return Ok(InstructionEffect::new(3, 3, None));
            }
            (ID::PointedByHL(dst), IT::Register8(src, _)) => (*dst, *src, 2, 1),
            (ID::PointedByHL(dst), IT::Immediate8(src)) => (*dst, *src, 3, 2),
            (ID::Register8(dst, _), IT::PointedByHL(src)) => (*dst, *src, 2, 1),
            (ID::PointedByRegister16(dst, _), IT::Register8(src, reg)) if *reg == R8::A => (dst, *src, 2, 1),
            (ID::PointedByN16(dst, _), IT::Register8(src, reg)) if *reg == R8::A => (*dst, *src, 4, 3),
            (ID::Register8(dst, _), IT::PointedByRegister16(src, _)) => (*dst, *src, 2, 1),
            (ID::Register8(dst, reg), IT::PointedByN16(src, _)) if *reg == R8::A => (*dst, *src, 4, 3),
            // sometimes written as `LD [HL+],A`, or `LDI [HL],A`
            (ID::PointedByHLI(dst, hl), IT::Register8(src, reg)) if *reg == R8::A => {
                with_u16(hl.1, hl.0, |hl| hl.wrapping_add(1));
                (dst, *src, 2, 1)
            }
            // sometimes written as `LD [HL-],A`, or `LDD [HL],A`
            (ID::PointedByHLD(dst, hl), IT::Register8(src, reg)) if *reg == R8::A => {
                with_u16(hl.1, hl.0, |hl| hl.wrapping_sub(1));
                (*dst, *src, 2, 1)
            }

            // stack manipulation load instructions

            // we'll do this load hear surpass the generic handling
            // with u8 destinations
            (ID::StackPointer(dst), IT::Immediate16(src)) => {
                **dst = *src;
                return Ok(InstructionEffect::new(3, 3, None));
            }
            (ID::PointedByN16AndNext(dst, _), IT::StackPointer(src)) => {
                let (high, low) = to_u8(*src);
                *dst.0 = high;
                *dst.1 = low;

                return Ok(InstructionEffect::new(5, 3, None));
            }
            // add the 8 bit signed immediate to the SP register and store the result in HL register pair
            // half carries come from Z80 with binary coded decimal, that worked with nibbles (4 bits)
            // also surpass the generic handling
            (ID::Register16(dst, reg), IT::StackPointerPlusE8(sp, e8)) if *reg == R16::HL => {
                let src = sp.wrapping_add(*e8 as i16 as u16);
                with_u16(dst.1, dst.0, |_| src);

                // the carries are computed on the low byte only, not the full u16
                let flags = check_half_carry(low(src), low(*sp)) | check_carry(low(src), low(*sp));

                return Ok(InstructionEffect::new(3, 2, Some(flags)));
            }
            (ID::StackPointer(dst), IT::Register16(src, reg)) if *reg == R16::HL => {
                **dst = to_u16(src.0, src.1);
                return Ok(InstructionEffect::new(2, 1, None));
            }

            _ => return Err(InstructionError::MalformedInstruction),
        };

        *dst = src;

        Ok(InstructionEffect::new(cycles, len, None))
    }

    fn disassembly(&self, w: &mut dyn Write) -> Result<(), std::fmt::Error> {
        write!(w, "ld {},{}", self.dst, self.src)
    }
}
