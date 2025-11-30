use std::fmt::Write;

use crate::{
    Dmg,
    core::{
        cpu::{
            flags::{Flags, check_overflow_cy, check_overflow_hc},
            instructions::{
                Instruction, InstructionDestination as ID, InstructionEffect, InstructionError,
                InstructionResult, InstructionTarget as IT,
            },
            registers::{Reg8 as R8, Reg16 as R16},
        },
        memory::Accessable,
    },
    utils::low,
};

pub struct Ld {
    dst: ID,
    src: IT,
}

impl Ld {
    pub fn new(dst: ID, src: IT) -> Box<Self> { Box::new(Self { dst, src }) }
}

impl Instruction for Ld {
    fn exec(&mut self, gb: &mut Dmg) -> InstructionResult {
        // u8 loads
        let (dst, src, cycles, len): (&mut u8, u8, u8, u8) = match (&mut self.dst, &self.src) {
            (ID::Reg8(reg), IT::Reg8(src, _)) => (&mut gb[&*reg], *src, 1, 1),
            (ID::Reg8(reg), IT::Imm8(src)) => (&mut gb[&*reg], *src, 2, 2),
            (ID::Reg16(reg), IT::Imm16(src)) => {
                gb.write16(&*reg, *src);

                return Ok(InstructionEffect::new(3, 3, Flags::none()));
            }
            (ID::PointedByHL(addr), IT::Reg8(src, _)) => (&mut gb[*addr], *src, 2, 1),
            (ID::PointedByHL(addr), IT::Imm8(src)) => (&mut gb[*addr], *src, 3, 2),
            (ID::Reg8(reg), IT::PointedByHL(src)) => (&mut gb[&*reg], *src, 2, 1),
            (ID::PointedByReg16(addr, _), IT::Reg8(src, reg)) if *reg == R8::A => {
                (&mut gb[*addr], *src, 2, 1)
            }
            (ID::PointedByN16(addr), IT::Reg8(src, reg)) if *reg == R8::A => (&mut gb[*addr], *src, 4, 3),
            (ID::Reg8(R8::A), IT::PointedByReg16(src, _)) => (&mut gb[&R8::A], *src, 2, 1),
            (ID::Reg8(R8::A), IT::PointedByN16(addr)) => {
                let src = gb[*addr];
                (&mut gb[&R8::A], src, 4, 3)
            }
            (ID::Reg8(R8::A), IT::PointedByHLI(src)) => {
                gb.write16(&R16::HL, gb.cpu.hl().wrapping_add(1));
                (&mut gb[&R8::A], *src, 2, 1)
            }
            (ID::Reg8(R8::A), IT::PointedByHLD(src)) => {
                gb.write16(&R16::HL, gb.cpu.hl().wrapping_sub(1));
                (&mut gb[&R8::A], *src, 2, 1)
            }
            // sometimes written as `Ld [HL+],A`, or `LDI [HL],A`
            (ID::PointedByHLI(addr), IT::Reg8(src, reg)) if *reg == R8::A => {
                gb.write16(&R16::HL, gb.cpu.hl().wrapping_add(1));
                (&mut gb[*addr], *src, 2, 1)
            }
            // sometimes written as `Ld [HL-],A`, or `LDD [HL],A`
            (ID::PointedByHLD(addr), IT::Reg8(src, reg)) if *reg == R8::A => {
                gb.write16(&R16::HL, gb.cpu.hl().wrapping_sub(1));
                (&mut gb[*addr], *src, 2, 1)
            }

            // stack manipulation load instructions

            // we'll do this load hear surpass the generic handling
            // with u8 destinations
            (ID::StackPointer, IT::Imm16(src)) => {
                gb.cpu.sp = *src;
                return Ok(InstructionEffect::new(3, 3, Flags::none()));
            }
            (ID::PointedByN16(addr), IT::StackPointer(src)) => {
                gb.write16(*addr, *src);
                return Ok(InstructionEffect::new(5, 3, Flags::none()));
            }
            // add the 8 bit signed immediate to the SP register and store the result in HL register pair
            // half carries come from Z80 with binary coded decimal, that worked with nibbles (4 bits)
            // also surpass the generic handling
            (ID::Reg16(R16::HL), IT::StackPointerPlusE8(sp, e8)) => {
                let src = sp.wrapping_add(*e8 as i16 as u16);
                gb.write16(&R16::HL, src);

                // the carries are computed on the low byte only, not the full u16
                let flags = Flags {
                    z: Some(false),
                    n: Some(false),
                    h: Some(check_overflow_hc(low(src), low(*sp))),
                    c: Some(check_overflow_cy(low(src), low(*sp))),
                };

                return Ok(InstructionEffect::new(3, 2, flags));
            }
            (ID::StackPointer, IT::Reg16(src, R16::HL)) => {
                gb.cpu.sp = *src;
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
