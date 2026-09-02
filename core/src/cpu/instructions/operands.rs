use crate::{
    IO_REGISTERS_START,
    cpu::{R8, R16},
    prelude::*,
};
use core::fmt::{Debug, Display, Formatter};

/// # 8 bit operand
/// From where a instruction takes bytes to operate with.
/// Different operands takes different amount of cycles and instruction length
pub trait Operand: Copy + Default + Debug + Display {
    const READ_CYCLES: u8;
    const LEN: u8;

    fn read(&self, gb: &Dmg) -> u8;
}

impl Operand for R8 {
    const READ_CYCLES: u8 = 0;
    const LEN: u8 = 0;

    #[inline(always)]
    fn read(&self, gb: &Dmg) -> u8 { gb.read(*self) }
}

/// # 8 bit writable operand
/// Some operands can be written
pub trait WritableOperand: Operand {
    const WRITE_CYCLES: u8;

    fn write(&self, gb: &mut Dmg, value: u8);
}

impl WritableOperand for R8 {
    const WRITE_CYCLES: u8 = 0;

    #[inline(always)]
    fn write(&self, gb: &mut Dmg, value: u8) { gb.write(*self, value) }
}

/// # 16 bit operand
/// Same, but bigger, and no instruction lenght cost, because they aren't immediate values
pub trait Operand16: Copy + Default + Debug + Display {
    fn load(&self, gb: &Dmg) -> u16;
    fn store(&self, gb: &mut Dmg, value: u16);
}

impl Operand16 for R16 {
    #[inline(always)]
    fn load(&self, gb: &Dmg) -> u16 { gb.load(*self) }

    #[inline(always)]
    fn store(&self, gb: &mut Dmg, value: u16) { gb.store(*self, value) }
}

#[derive(Debug, Default, Clone, Copy)]
pub struct PointedByHL;
impl Display for PointedByHL {
    fn fmt(&self, f: &mut Formatter) -> core::fmt::Result { write!(f, "[hl]") }
}
impl Operand for PointedByHL {
    const READ_CYCLES: u8 = 1;
    const LEN: u8 = 0;

    #[inline(always)]
    fn read(&self, gb: &Dmg) -> u8 { gb.read(gb.cpu.hl()) }
}

#[derive(Debug, Default, Clone, Copy)]
pub struct Imm8(pub u8);
impl Display for Imm8 {
    fn fmt(&self, f: &mut Formatter) -> core::fmt::Result { write!(f, "${:02X}", self.0) }
}
impl Operand for Imm8 {
    const READ_CYCLES: u8 = 1;
    const LEN: u8 = 1;

    #[inline(always)]
    fn read(&self, _gb: &Dmg) -> u8 { self.0 }
}
impl WritableOperand for PointedByHL {
    const WRITE_CYCLES: u8 = 1;

    #[inline(always)]
    fn write(&self, gb: &mut Dmg, value: u8) { gb.write(gb.cpu.hl(), value) }
}

#[derive(Debug, Default, Clone, Copy)]
pub struct PointedByR16(pub R16);
impl Display for PointedByR16 {
    fn fmt(&self, f: &mut Formatter) -> core::fmt::Result { write!(f, "[{}]", self.0) }
}
impl Operand for PointedByR16 {
    const READ_CYCLES: u8 = 1;
    const LEN: u8 = 0;

    #[inline(always)]
    fn read(&self, gb: &Dmg) -> u8 { gb.read(gb.load(self.0)) }
}

impl WritableOperand for PointedByR16 {
    const WRITE_CYCLES: u8 = 1;

    #[inline(always)]
    fn write(&self, gb: &mut Dmg, value: u8) {
        let addr = gb.load(self.0);
        gb.write(addr, value)
    }
}

#[derive(Debug, Default, Clone, Copy)]
pub struct PointedByImm16(pub u16);
impl Display for PointedByImm16 {
    fn fmt(&self, f: &mut Formatter) -> core::fmt::Result { write!(f, "[${:04X}]", self.0) }
}
impl Operand for PointedByImm16 {
    const READ_CYCLES: u8 = 3;
    const LEN: u8 = 2;

    #[inline(always)]
    fn read(&self, gb: &Dmg) -> u8 { gb.read(self.0) }
}

impl WritableOperand for PointedByImm16 {
    const WRITE_CYCLES: u8 = 3;

    #[inline(always)]
    fn write(&self, gb: &mut Dmg, value: u8) { gb.write(self.0, value) }
}

#[derive(Debug, Default, Clone, Copy)]
pub struct PointedByC;
impl Display for PointedByC {
    fn fmt(&self, f: &mut Formatter) -> core::fmt::Result { write!(f, "[c]") }
}
impl Operand for PointedByC {
    const READ_CYCLES: u8 = 1;
    const LEN: u8 = 0;

    #[inline(always)]
    fn read(&self, gb: &Dmg) -> u8 { gb.read(IO_REGISTERS_START + gb.cpu.c as u16) }
}
impl WritableOperand for PointedByC {
    const WRITE_CYCLES: u8 = 1;

    #[inline(always)]
    fn write(&self, gb: &mut Dmg, value: u8) { gb.write(IO_REGISTERS_START + gb.cpu.c as u16, value) }
}

#[derive(Debug, Default, Clone, Copy)]
pub struct PointedByHighImm8(pub u8);
impl PointedByHighImm8 {
    #[inline(always)]
    fn addr(&self) -> u16 { IO_REGISTERS_START + self.0 as u16 }
}
impl Display for PointedByHighImm8 {
    fn fmt(&self, f: &mut Formatter) -> core::fmt::Result { write!(f, "[${:04X}]", self.addr()) }
}
impl Operand for PointedByHighImm8 {
    const READ_CYCLES: u8 = 2;
    const LEN: u8 = 1;

    #[inline(always)]
    fn read(&self, gb: &Dmg) -> u8 { gb.read(self.addr()) }
}
impl WritableOperand for PointedByHighImm8 {
    const WRITE_CYCLES: u8 = 2;

    #[inline(always)]
    fn write(&self, gb: &mut Dmg, value: u8) { gb.write(self.addr(), value) }
}

#[derive(Debug, Default, Clone, Copy)]
pub struct StackPointer;
impl Display for StackPointer {
    fn fmt(&self, f: &mut Formatter) -> core::fmt::Result { write!(f, "sp") }
}
impl Operand16 for StackPointer {
    #[inline(always)]
    fn load(&self, gb: &Dmg) -> u16 { gb.cpu.sp }

    #[inline(always)]
    fn store(&self, gb: &mut Dmg, value: u16) { gb.cpu.sp = value }
}
