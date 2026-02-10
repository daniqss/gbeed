use crate::{
    Dmg,
    core::{
        Accessible16,
        cpu::{
            R8, R16,
            flags::{Flags, check_overflow_cy, check_overflow_hc},
            instructions::{Instruction, InstructionEffect, InstructionResult},
        },
        memory::Accessible,
    },
    utils::low,
};

/// LD r8, r8
/// Load value from src register into dst register
pub struct LdR8R8 {
    pub dst: R8,
    pub src: R8,
}

impl LdR8R8 {
    pub fn new(dst: R8, src: R8) -> Box<Self> { Box::new(Self { dst, src }) }
}

impl Instruction for LdR8R8 {
    fn exec(&mut self, gb: &mut Dmg) -> InstructionResult {
        let val = gb.read(self.src);
        gb.write(self.dst, val);
        Ok(InstructionEffect::new(self.info(), Flags::none()))
    }

    fn info(&self) -> (u8, u8) { (1, 1) }
    fn disassembly(&self) -> String { format!("ld {},{}", self.dst, self.src) }
}

/// LD r8, n8
/// Load immediate 8-bit value into dst register
pub struct LdR8Imm8 {
    pub dst: R8,
    pub val: u8,
}

impl LdR8Imm8 {
    pub fn new(dst: R8, val: u8) -> Box<Self> { Box::new(Self { dst, val }) }
}

impl Instruction for LdR8Imm8 {
    fn exec(&mut self, gb: &mut Dmg) -> InstructionResult {
        gb.write(self.dst, self.val);
        Ok(InstructionEffect::new(self.info(), Flags::none()))
    }

    fn info(&self) -> (u8, u8) { (2, 2) }
    fn disassembly(&self) -> String { format!("ld {},${:02X}", self.dst, self.val) }
}

/// LD r16, n16
/// Load immediate 16-bit value into dst register
pub struct LdR16Imm16 {
    pub dst: R16,
    pub val: u16,
}

impl LdR16Imm16 {
    pub fn new(dst: R16, val: u16) -> Box<Self> { Box::new(Self { dst, val }) }
}

impl Instruction for LdR16Imm16 {
    fn exec(&mut self, gb: &mut Dmg) -> InstructionResult {
        gb.store(self.dst, self.val);
        Ok(InstructionEffect::new(self.info(), Flags::none()))
    }

    fn info(&self) -> (u8, u8) { (3, 3) }
    fn disassembly(&self) -> String { format!("ld {},${:04X}", self.dst, self.val) }
}

/// LD SP, n16
/// Load immediate 16-bit value into Stack Pointer
pub struct LdSPImm16 {
    pub val: u16,
}

impl LdSPImm16 {
    pub fn new(val: u16) -> Box<Self> { Box::new(Self { val }) }
}

impl Instruction for LdSPImm16 {
    fn exec(&mut self, gb: &mut Dmg) -> InstructionResult {
        gb.cpu.sp = self.val;
        Ok(InstructionEffect::new(self.info(), Flags::none()))
    }

    fn info(&self) -> (u8, u8) { (3, 3) }
    fn disassembly(&self) -> String { format!("ld sp,${:04X}", self.val) }
}

/// LD [HL], r8
/// Load value from src register into byte pointed by HL
pub struct LdPointedByHLR8 {
    pub src: R8,
}

impl LdPointedByHLR8 {
    pub fn new(src: R8) -> Box<Self> { Box::new(Self { src }) }
}

impl Instruction for LdPointedByHLR8 {
    fn exec(&mut self, gb: &mut Dmg) -> InstructionResult {
        let val = gb.read(self.src);
        gb.write(gb.cpu.hl(), val);
        Ok(InstructionEffect::new(self.info(), Flags::none()))
    }

    fn info(&self) -> (u8, u8) { (2, 1) }
    fn disassembly(&self) -> String { format!("ld [hl],{}", self.src) }
}

/// LD [HL], n8
/// Load immediate 8-bit value into byte pointed by HL
pub struct LdPointedByHLImm8 {
    pub val: u8,
}

impl LdPointedByHLImm8 {
    pub fn new(val: u8) -> Box<Self> { Box::new(Self { val }) }
}

impl Instruction for LdPointedByHLImm8 {
    fn exec(&mut self, gb: &mut Dmg) -> InstructionResult {
        gb.write(gb.cpu.hl(), self.val);
        Ok(InstructionEffect::new(self.info(), Flags::none()))
    }

    fn info(&self) -> (u8, u8) { (3, 2) }
    fn disassembly(&self) -> String { format!("ld [hl],${:02X}", self.val) }
}

/// LD r8, [HL]
/// Load value from byte pointed by HL into dst register
pub struct LdR8PointedByHL {
    pub dst: R8,
}

impl LdR8PointedByHL {
    pub fn new(dst: R8) -> Box<Self> { Box::new(Self { dst }) }
}

impl Instruction for LdR8PointedByHL {
    fn exec(&mut self, gb: &mut Dmg) -> InstructionResult {
        let val = gb.read(gb.cpu.hl());
        gb.write(self.dst, val);
        Ok(InstructionEffect::new(self.info(), Flags::none()))
    }

    fn info(&self) -> (u8, u8) { (2, 1) }
    fn disassembly(&self) -> String { format!("ld {},[hl]", self.dst) }
}

/// LD [r16], A
/// Load value from A into byte pointed by BC or DE
pub struct LdPointedByR16A {
    pub dst: R16,
}

impl LdPointedByR16A {
    pub fn new(dst: R16) -> Box<Self> { Box::new(Self { dst }) }
}

impl Instruction for LdPointedByR16A {
    fn exec(&mut self, gb: &mut Dmg) -> InstructionResult {
        let addr = gb.load(self.dst);
        gb.write(addr, gb.cpu.a);
        Ok(InstructionEffect::new(self.info(), Flags::none()))
    }

    fn info(&self) -> (u8, u8) { (2, 1) }
    fn disassembly(&self) -> String { format!("ld [{}],a", self.dst) }
}

/// LD [nn], A
/// Load value from A into byte pointed by immediate 16-bit address
pub struct LdPointedByImm16A {
    pub addr: u16,
}

impl LdPointedByImm16A {
    pub fn new(addr: u16) -> Box<Self> { Box::new(Self { addr }) }
}

impl Instruction for LdPointedByImm16A {
    fn exec(&mut self, gb: &mut Dmg) -> InstructionResult {
        gb.write(self.addr, gb.cpu.a);
        Ok(InstructionEffect::new(self.info(), Flags::none()))
    }

    fn info(&self) -> (u8, u8) { (4, 3) }
    fn disassembly(&self) -> String { format!("ld [${:04X}],a", self.addr) }
}

/// LD A, [r16]
/// Load value from byte pointed by BC or DE into A
pub struct LdAPointedByR16 {
    pub src: R16,
}

impl LdAPointedByR16 {
    pub fn new(src: R16) -> Box<Self> { Box::new(Self { src }) }
}

impl Instruction for LdAPointedByR16 {
    fn exec(&mut self, gb: &mut Dmg) -> InstructionResult {
        let addr = gb.load(self.src);
        gb.cpu.a = gb.read(addr);
        Ok(InstructionEffect::new(self.info(), Flags::none()))
    }

    fn info(&self) -> (u8, u8) { (2, 1) }
    fn disassembly(&self) -> String { format!("ld a,[{}]", self.src) }
}

/// LD A, [nn]
/// Load value from byte pointed by immediate 16-bit address into A
pub struct LdAPointedByImm16 {
    pub addr: u16,
}

impl LdAPointedByImm16 {
    pub fn new(addr: u16) -> Box<Self> { Box::new(Self { addr }) }
}

impl Instruction for LdAPointedByImm16 {
    fn exec(&mut self, gb: &mut Dmg) -> InstructionResult {
        gb.cpu.a = gb.read(self.addr);
        Ok(InstructionEffect::new(self.info(), Flags::none()))
    }

    fn info(&self) -> (u8, u8) { (4, 3) }
    fn disassembly(&self) -> String { format!("ld a,[${:04X}]", self.addr) }
}

/// LD [HL+], A
/// Load A into byte pointed by HL, then increment HL
pub struct LdPointedByHLIncA;

impl LdPointedByHLIncA {
    pub fn new() -> Box<Self> { Box::new(Self) }
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
pub struct LdPointedByHLDecA;

impl LdPointedByHLDecA {
    pub fn new() -> Box<Self> { Box::new(Self) }
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
pub struct LdAPointedByHLInc;

impl LdAPointedByHLInc {
    pub fn new() -> Box<Self> { Box::new(Self) }
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
pub struct LdAPointedByHLDec;

impl LdAPointedByHLDec {
    pub fn new() -> Box<Self> { Box::new(Self) }
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
pub struct LdImm16SP {
    pub addr: u16,
}

impl LdImm16SP {
    pub fn new(addr: u16) -> Box<Self> { Box::new(Self { addr }) }
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
pub struct LdHLSPPlusImm8 {
    pub e8: i8,
}

impl LdHLSPPlusImm8 {
    pub fn new(e8: i8) -> Box<Self> { Box::new(Self { e8 }) }
}

impl Instruction for LdHLSPPlusImm8 {
    fn exec(&mut self, gb: &mut Dmg) -> InstructionResult {
        let sp = gb.cpu.sp;
        let result = sp.wrapping_add(self.e8 as i16 as u16);
        gb.store(R16::HL, result);

        let flags = Flags {
            z: Some(false),
            n: Some(false),
            h: Some(check_overflow_hc(low(result), low(sp))),
            c: Some(check_overflow_cy(low(result), low(sp))),
        };

        Ok(InstructionEffect::new(self.info(), flags))
    }

    fn info(&self) -> (u8, u8) { (3, 2) }
    fn disassembly(&self) -> String { format!("ld hl,sp{:+}", self.e8) }
}

/// LD SP, HL
/// Load HL into SP
pub struct LdSPHL;

impl LdSPHL {
    pub fn new() -> Box<Self> { Box::new(Self) }
}

impl Instruction for LdSPHL {
    fn exec(&mut self, gb: &mut Dmg) -> InstructionResult {
        gb.cpu.sp = gb.cpu.hl();
        Ok(InstructionEffect::new(self.info(), Flags::none()))
    }

    fn info(&self) -> (u8, u8) { (2, 1) }
    fn disassembly(&self) -> String { "ld sp,hl".to_string() }
}
