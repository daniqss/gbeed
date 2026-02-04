use std::fmt::Write;

use crate::{
    Dmg,
    core::{
        Accessible16,
        cpu::{
            R8, R16,
            flags::{Flags, check_borrow_hc, check_zero},
            instructions::{Instruction, InstructionEffect, InstructionResult},
        },
        memory::Accessible,
    },
};

pub trait Dec: Instruction {
    type Args;
    type FlagsArgs;

    fn new(args: Self::Args) -> Box<Self>;
    fn flags(&self, args: Self::FlagsArgs) -> Flags;
    fn cycles(&self) -> u8;
    fn len(&self) -> u8;
}

pub struct DecR8 {
    dst: R8,
}

impl Dec for DecR8 {
    type Args = R8;
    type FlagsArgs = (u8, u8);

    #[inline]
    fn new(dst: R8) -> Box<Self> { Box::new(DecR8 { dst }) }
    #[inline]
    fn flags(&self, args: Self::FlagsArgs) -> Flags {
        let (old, result) = args;
        Flags {
            z: Some(check_zero(result)),
            n: Some(true),
            h: Some(check_borrow_hc(old, 1)),
            c: None,
        }
    }
    #[inline]
    fn cycles(&self) -> u8 { 1 }
    #[inline]
    fn len(&self) -> u8 { 1 }
}

impl Instruction for DecR8 {
    fn exec(&mut self, gb: &mut Dmg) -> InstructionResult {
        let r8 = gb.read(self.dst);
        let result = r8.wrapping_sub(1);
        gb.write(self.dst, result);

        Ok(InstructionEffect::new(
            self.cycles(),
            self.len(),
            self.flags((r8, result)),
        ))
    }

    fn disassembly(&self, w: &mut dyn Write) -> Result<(), std::fmt::Error> { write!(w, "dec {}", self.dst) }
}

pub struct DecPointedByHL;

impl Dec for DecPointedByHL {
    type Args = ();
    type FlagsArgs = (u8, u8);

    #[inline(always)]
    fn new(_: ()) -> Box<Self> { Box::new(DecPointedByHL) }
    #[inline(always)]
    fn flags(&self, args: Self::FlagsArgs) -> Flags {
        let (old, result) = args;
        Flags {
            z: Some(check_zero(result)),
            n: Some(true),
            h: Some(check_borrow_hc(old, 1)),
            c: None,
        }
    }
    #[inline(always)]
    fn cycles(&self) -> u8 { 3 }
    #[inline(always)]
    fn len(&self) -> u8 { 1 }
}

impl Instruction for DecPointedByHL {
    fn exec(&mut self, gb: &mut Dmg) -> InstructionResult {
        let n8 = gb.read(gb.cpu.hl());
        let result = n8.wrapping_sub(1);
        gb.write(gb.cpu.hl(), result);

        Ok(InstructionEffect::new(
            self.cycles(),
            self.len(),
            self.flags((n8, result)),
        ))
    }

    fn disassembly(&self, w: &mut dyn Write) -> Result<(), std::fmt::Error> { write!(w, "dec [hl]") }
}

pub struct DecR16 {
    dst: R16,
}

impl Dec for DecR16 {
    type Args = R16;
    type FlagsArgs = ();

    #[inline(always)]
    fn new(dst: R16) -> Box<Self> { Box::new(DecR16 { dst }) }
    #[inline(always)]
    fn flags(&self, _: Self::FlagsArgs) -> Flags { Flags::none() }
    #[inline(always)]
    fn cycles(&self) -> u8 { 2 }
    #[inline(always)]
    fn len(&self) -> u8 { 1 }
}

impl Instruction for DecR16 {
    fn exec(&mut self, gb: &mut Dmg) -> InstructionResult {
        let r16 = gb.load(self.dst);
        let result = r16.wrapping_sub(1);
        gb.store(self.dst, result);

        Ok(InstructionEffect::new(self.cycles(), self.len(), self.flags(())))
    }

    fn disassembly(&self, w: &mut dyn Write) -> Result<(), std::fmt::Error> { write!(w, "dec {}", self.dst) }
}

pub struct DecStackPointer;

impl Dec for DecStackPointer {
    type Args = ();
    type FlagsArgs = ();

    #[inline(always)]
    fn new(_: ()) -> Box<Self> { Box::new(DecStackPointer) }
    #[inline(always)]
    fn flags(&self, _: Self::FlagsArgs) -> Flags { Flags::none() }
    #[inline(always)]
    fn cycles(&self) -> u8 { 2 }
    #[inline(always)]
    fn len(&self) -> u8 { 1 }
}

impl Instruction for DecStackPointer {
    fn exec(&mut self, gb: &mut Dmg) -> InstructionResult {
        gb.cpu.sp = gb.cpu.sp.wrapping_sub(1);

        Ok(InstructionEffect::new(self.cycles(), self.len(), self.flags(())))
    }

    fn disassembly(&self, w: &mut dyn Write) -> Result<(), std::fmt::Error> { write!(w, "dec sp") }
}
