use crate::{
    cpu::{
        R16,
        flags::{Flags, check_overflow_cy, check_overflow_hc},
        instructions::{
            Instruction, InstructionEffect, InstructionResult, Operand, Operand16, WritableOperand,
        },
    },
    prelude::*,
};

/// LD dst, src
/// Load a byte from any 8 bit operand into any writable one
#[derive(Debug, Default, Clone, Copy)]
pub struct Ld<D: WritableOperand, S: Operand> {
    dst: D,
    src: S,
}
impl<D: WritableOperand, S: Operand> Ld<D, S> {
    pub fn new(dst: D, src: S) -> Self { Self { dst, src } }
}
impl<D: WritableOperand, S: Operand> Instruction for Ld<D, S> {
    fn exec(&mut self, gb: &mut Dmg) -> InstructionResult {
        let value = self.src.read(gb);
        self.dst.write(gb, value);

        Ok(InstructionEffect::new(self.info(), Flags::none()))
    }
    fn info(&self) -> (u8, u8) { (1 + S::READ_CYCLES + D::WRITE_CYCLES, 1 + S::LEN + D::LEN) }
    fn disassembly(&self) -> String { format!("ld {},{}", self.dst, self.src) }
}

/// LD r16, n16
/// Load an immediate 16-bit value into a register pair or the stack pointer
#[derive(Debug, Default, Clone, Copy)]
pub struct Ld16<D: Operand16> {
    dst: D,
    val: u16,
}
impl<D: Operand16> Ld16<D> {
    pub fn new(dst: D, val: u16) -> Self { Self { dst, val } }
}
impl<D: Operand16> Instruction for Ld16<D> {
    fn exec(&mut self, gb: &mut Dmg) -> InstructionResult {
        self.dst.store(gb, self.val);

        Ok(InstructionEffect::new(self.info(), Flags::none()))
    }
    fn info(&self) -> (u8, u8) { (3, 3) }
    fn disassembly(&self) -> String { format!("ld {},${:04X}", self.dst, self.val) }
}

/// LD [HL+], A
/// Load A into byte pointed by HL, then increment HL
#[derive(Debug, Default, Clone, Copy)]
pub struct LdPointedByHLIncA;

impl LdPointedByHLIncA {
    pub fn new() -> Self { Self }
}

impl Instruction for LdPointedByHLIncA {
    fn exec(&mut self, gb: &mut Dmg) -> InstructionResult {
        let hl = gb.cpu.hl();
        gb.write(hl, gb.cpu.a);
        gb.store(R16::HL, hl.wrapping_add(1));
        Ok(InstructionEffect::new(self.info(), Flags::none()))
    }

    fn info(&self) -> (u8, u8) { (2, 1) }
    fn disassembly(&self) -> String { "ld [hli],a".to_string() }
}

/// LD [HL-], A
/// Load A into byte pointed by HL, then decrement HL
#[derive(Debug, Default, Clone, Copy)]
pub struct LdPointedByHLDecA;

impl LdPointedByHLDecA {
    pub fn new() -> Self { Self }
}

impl Instruction for LdPointedByHLDecA {
    fn exec(&mut self, gb: &mut Dmg) -> InstructionResult {
        let hl = gb.cpu.hl();
        gb.write(hl, gb.cpu.a);
        gb.store(R16::HL, hl.wrapping_sub(1));
        Ok(InstructionEffect::new(self.info(), Flags::none()))
    }

    fn info(&self) -> (u8, u8) { (2, 1) }
    fn disassembly(&self) -> String { "ld [hld],a".to_string() }
}

/// LD A, [HL+]
/// Load byte pointed by HL into A, then increment HL
#[derive(Debug, Default, Clone, Copy)]
pub struct LdAPointedByHLInc;

impl LdAPointedByHLInc {
    pub fn new() -> Self { Self }
}

impl Instruction for LdAPointedByHLInc {
    fn exec(&mut self, gb: &mut Dmg) -> InstructionResult {
        let hl = gb.cpu.hl();
        gb.cpu.a = gb.read(hl);
        gb.store(R16::HL, hl.wrapping_add(1));
        Ok(InstructionEffect::new(self.info(), Flags::none()))
    }

    fn info(&self) -> (u8, u8) { (2, 1) }
    fn disassembly(&self) -> String { "ld a,[hli]".to_string() }
}

/// LD A, [HL-]
/// Load byte pointed by HL into A, then decrement HL
#[derive(Debug, Default, Clone, Copy)]
pub struct LdAPointedByHLDec;

impl LdAPointedByHLDec {
    pub fn new() -> Self { Self }
}

impl Instruction for LdAPointedByHLDec {
    fn exec(&mut self, gb: &mut Dmg) -> InstructionResult {
        let hl = gb.cpu.hl();
        gb.cpu.a = gb.read(hl);
        gb.store(R16::HL, hl.wrapping_sub(1));
        Ok(InstructionEffect::new(self.info(), Flags::none()))
    }

    fn info(&self) -> (u8, u8) { (2, 1) }
    fn disassembly(&self) -> String { "ld a,[hld]".to_string() }
}

/// LD [nn], SP
/// Load SP into 16-bit address nn (little endian)
#[derive(Debug, Default, Clone, Copy)]
pub struct LdImm16SP {
    pub addr: u16,
}

impl LdImm16SP {
    pub fn new(addr: u16) -> Self { Self { addr } }
}

impl Instruction for LdImm16SP {
    fn exec(&mut self, gb: &mut Dmg) -> InstructionResult {
        gb.store(self.addr, gb.cpu.sp);
        Ok(InstructionEffect::new(self.info(), Flags::none()))
    }

    fn info(&self) -> (u8, u8) { (5, 3) }
    fn disassembly(&self) -> String { format!("ld [${:04X}],sp", self.addr) }
}

/// LD HL, SP+e8
/// Add signed 8-bit immediate to SP and store in HL
#[derive(Debug, Default, Clone, Copy)]
pub struct LdHLSPPlusImm8 {
    pub e8: i8,
}

impl LdHLSPPlusImm8 {
    pub fn new(e8: i8) -> Self { Self { e8 } }
}

impl Instruction for LdHLSPPlusImm8 {
    fn exec(&mut self, gb: &mut Dmg) -> InstructionResult {
        let sp = gb.cpu.sp;
        let result = sp.wrapping_add(self.e8 as i16 as u16);
        gb.store(R16::HL, result);

        let flags = Flags {
            z: Some(false),
            n: Some(false),
            h: Some(check_overflow_hc(utils::low(result), utils::low(sp))),
            c: Some(check_overflow_cy(utils::low(result), utils::low(sp))),
        };

        Ok(InstructionEffect::new(self.info(), flags))
    }

    fn info(&self) -> (u8, u8) { (3, 2) }
    fn disassembly(&self) -> String { format!("ld hl,sp{:+}", self.e8) }
}

/// LD SP, HL
/// Load HL into SP
#[derive(Debug, Default, Clone, Copy)]
pub struct LdSPHL;

impl LdSPHL {
    pub fn new() -> Self { Self }
}

impl Instruction for LdSPHL {
    fn exec(&mut self, gb: &mut Dmg) -> InstructionResult {
        gb.cpu.sp = gb.cpu.hl();
        Ok(InstructionEffect::new(self.info(), Flags::none()))
    }

    fn info(&self) -> (u8, u8) { (2, 1) }
    fn disassembly(&self) -> String { "ld sp,hl".to_string() }
}
