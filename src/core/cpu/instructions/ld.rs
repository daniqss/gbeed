use std::fmt::Write;

use crate::{
    core::cpu::{
        flags::{Flags, check_overflow_cy, check_overflow_hc},
        instructions::{
            Instruction, InstructionDestination as ID, InstructionEffect, InstructionError,
            InstructionResult, InstructionTarget as IT,
        },
        registers::{Reg8 as R8, Reg16 as R16},
    },
    utils::{low, to_u8, to_u16, with_u16},
};

pub struct Ld<'a> {
    dst: ID<'a>,
    src: IT<'a>,
}

impl<'a> Ld<'a> {
    pub fn new(dst: ID<'a>, src: IT<'a>) -> Box<Self> { Box::new(Self { dst, src }) }
}

impl<'a> Instruction<'a> for Ld<'a> {
    fn exec(&mut self, gb: &mut Dmg) -> InstructionResult {
        // handle cases where srcs are increased and decreased after load
        if let (ID::Reg8(dst, reg), IT::PointedByHLD(src, hl)) = (&mut self.dst, &mut self.src) {
            if *reg != R8::A {
                return Err(InstructionError::MalformedInstruction);
            }

            **dst = *src;

            with_u16(hl.0, hl.1, |hl| hl.wrapping_add(1));
            return Ok(InstructionEffect::new(2, 1, Flags::none()));
        }
        if let (ID::Reg8(dst, reg), IT::PointedByHLI(src, hl)) = (&mut self.dst, &mut self.src) {
            if *reg != R8::A {
                return Err(InstructionError::MalformedInstruction);
            }

            **dst = *src;

            with_u16(hl.0, hl.1, |hl| hl.wrapping_sub(1));
            return Ok(InstructionEffect::new(2, 1, Flags::none()));
        }

        // u8 loads
        let (dst, src, cycles, len): (&mut u8, u8, u8, u8) = match (&mut self.dst, &self.src) {
            (ID::Reg8(dst, _), IT::Reg8(src, _)) => (*dst, *src, 1, 1),
            (ID::Reg8(dst, _), IT::Imm8(src)) => (*dst, *src, 2, 2),
            (ID::Reg16(dst, _), IT::Imm16(src)) => {
                let (high, low) = to_u8(*src);
                *dst.0 = high;
                *dst.1 = low;

                return Ok(InstructionEffect::new(3, 3, Flags::none()));
            }
            (ID::PointedByHL(bus, addr), IT::Reg8(src, _)) => (&mut bus.borrow_mut()[*addr], *src, 2, 1),
            (ID::PointedByHL(bus, addr), IT::Imm8(src)) => (&mut bus.borrow_mut()[*addr], *src, 3, 2),
            (ID::Reg8(dst, _), IT::PointedByHL(src)) => (*dst, *src, 2, 1),
            (ID::PointedByReg16(bus, addr, _), IT::Reg8(src, reg)) if *reg == R8::A => {
                (&mut bus.borrow_mut()[*addr], *src, 2, 1)
            }
            (ID::PointedByN16(bus, addr), IT::Reg8(src, reg)) if *reg == R8::A => {
                (&mut bus.borrow_mut()[*addr], *src, 4, 3)
            }
            (ID::Reg8(dst, _), IT::PointedByReg16(src, _)) => (*dst, *src, 2, 1),
            (ID::Reg8(dst, reg), IT::PointedByN16(src, _)) if *reg == R8::A => (*dst, *src, 4, 3),
            // sometimes written as `Ld [HL+],A`, or `LDI [HL],A`
            (ID::PointedByHLI(bus, hl), IT::Reg8(src, reg)) if *reg == R8::A => {
                with_u16(hl.0, hl.1, |hl| hl.wrapping_add(1));
                (&mut bus.borrow_mut()[to_u16(*hl.0, *hl.1)], *src, 2, 1)
            }
            // sometimes written as `Ld [HL-],A`, or `LDD [HL],A`
            (ID::PointedByHLD(bus, hl), IT::Reg8(src, reg)) if *reg == R8::A => {
                with_u16(hl.0, hl.1, |hl| hl.wrapping_sub(1));
                (&mut bus.borrow_mut()[to_u16(*hl.0, *hl.1)], *src, 2, 1)
            }

            // stack manipulation load instructions

            // we'll do this load hear surpass the generic handling
            // with u8 destinations
            (ID::StackPointer(dst), IT::Imm16(src)) => {
                **dst = *src;
                return Ok(InstructionEffect::new(3, 3, Flags::none()));
            }
            (ID::PointedByN16(bus, addr), IT::StackPointer(src)) => {
                // bus.borrow_mut()[addr.wrapping_add(1)] = high(*src);
                // bus.borrow_mut()[*addr] = low(*src);
                bus.borrow_mut().write_word(*addr, *src);

                return Ok(InstructionEffect::new(5, 3, Flags::none()));
            }
            // add the 8 bit signed immediate to the SP register and store the result in HL register pair
            // half carries come from Z80 with binary coded decimal, that worked with nibbles (4 bits)
            // also surpass the generic handling
            (ID::Reg16(dst, R16::HL), IT::StackPointerPlusE8(sp, e8)) => {
                let src = sp.wrapping_add(*e8 as i16 as u16);
                with_u16(dst.0, dst.1, |_| src);

                // the carries are computed on the low byte only, not the full u16
                let flags = Flags {
                    z: Some(false),
                    n: Some(false),
                    h: Some(check_overflow_hc(low(src), low(*sp))),
                    c: Some(check_overflow_cy(low(src), low(*sp))),
                };

                return Ok(InstructionEffect::new(3, 2, flags));
            }
            (ID::StackPointer(dst), IT::Reg16(src, R16::HL)) => {
                **dst = *src;
                return Ok(InstructionEffect::new(2, 1, Flags::none()));
            }

            _ => return Err(InstructionError::MalformedInstruction),
        };

        *dst = src;

        Ok(InstructionEffect::new(cycles, len, Flags::none()))
    }

    fn disassembly(&self, w: &mut dyn Write) -> Result<(), std::fmt::Error> {
        write!(w, "ld {},{}", self.dst, self.src)
    }
}
