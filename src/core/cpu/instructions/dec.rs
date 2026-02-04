use std::fmt::{self, Display, Write};

use crate::{
    Dmg,
    core::{
        cpu::{
            flags::{Flags, check_borrow_hc, check_zero},
            instructions::{Instruction, InstructionEffect, InstructionResult},
            {R8, R16},
        },
        memory::{Accessible, Accessible16},
    },
};

// --- Operands (TODO: Move to instructions/mod.rs) ---

pub trait Operand8: Display {
    fn read(&self, gb: &Dmg) -> u8;
    fn write(&self, gb: &mut Dmg, value: u8);
    fn cycles(&self) -> u8;
}

pub trait Operand16: Display {
    fn read(&self, gb: &Dmg) -> u16;
    fn write(&self, gb: &mut Dmg, value: u16);
    fn cycles(&self) -> u8;
}

pub struct OpReg8(pub R8);
impl Display for OpReg8 {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result { write!(f, "{}", self.0) }
}
impl Operand8 for OpReg8 {
    fn read(&self, gb: &Dmg) -> u8 { gb.read(self.0) }
    fn write(&self, gb: &mut Dmg, value: u8) { gb.write(self.0, value) }
    fn cycles(&self) -> u8 { 0 }
}

pub struct OpHL;
impl Display for OpHL {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result { write!(f, "[hl]") }
}
impl Operand8 for OpHL {
    fn read(&self, gb: &Dmg) -> u8 { gb.read(gb.cpu.hl()) }
    fn write(&self, gb: &mut Dmg, value: u8) { gb.write(gb.cpu.hl(), value) }
    fn cycles(&self) -> u8 { 2 }
}

pub struct OpReg16(pub R16);
impl Display for OpReg16 {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result { write!(f, "{}", self.0) }
}
impl Operand16 for OpReg16 {
    fn read(&self, gb: &Dmg) -> u16 { gb.load(self.0) }
    fn write(&self, gb: &mut Dmg, value: u16) { gb.store(self.0, value) }
    fn cycles(&self) -> u8 { 0 }
}

pub struct OpSP;
impl Display for OpSP {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result { write!(f, "sp") }
}
impl Operand16 for OpSP {
    fn read(&self, gb: &Dmg) -> u16 { gb.cpu.sp }
    fn write(&self, gb: &mut Dmg, value: u16) { gb.cpu.sp = value }
    fn cycles(&self) -> u8 { 0 }
}

// --- Instructions ---

/// Decrement the 8-bit operand value by one
pub struct Dec8<O>(pub O);

impl<O: Operand8> Instruction for Dec8<O> {
    fn exec(&mut self, gb: &mut Dmg) -> InstructionResult {
        let len = 1;
        let val = self.0.read(gb);
        let result = val.wrapping_sub(1);
        self.0.write(gb, result);

        let flags = Flags {
            z: Some(check_zero(result)),
            n: Some(true),
            h: Some(check_borrow_hc(val, 1)),
            c: None,
        };

        Ok(InstructionEffect::new(1 + self.0.cycles(), len, flags))
    }

    fn disassembly(&self, w: &mut dyn Write) -> fmt::Result {
        write!(w, "dec {}", self.0)
    }
}

/// Decrement the 16-bit operand value by one
pub struct Dec16<O>(pub O);

impl<O: Operand16> Instruction for Dec16<O> {
    fn exec(&mut self, gb: &mut Dmg) -> InstructionResult {
        let len = 1;
        let val = self.0.read(gb);
        self.0.write(gb, val.wrapping_sub(1));

        Ok(InstructionEffect::new(2 + self.0.cycles(), len, Flags::none()))
    }

    fn disassembly(&self, w: &mut dyn Write) -> fmt::Result {
        write!(w, "dec {}", self.0)
    }
}