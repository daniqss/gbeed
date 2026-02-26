use crate::{
    cpu::{
        flags::Flags,
        instructions::{Instruction, InstructionEffect, InstructionResult},
    },
    prelude::*,
    IO_REGISTERS_START,
};

/// Load from/to high memory area instruction
/// Usually used to access memory mapped IO and HRAM,
/// so the used addresses are between 0xFF00 and 0xFFFF
///
/// copy the src value in register A to the byte at 16 bits immediate address (that must be between 0xFF00 and 0xFFFF)
pub struct LdhImm8A {
    pub addr_offset: u8,
}

impl LdhImm8A {
    pub fn new(addr_offset: u8) -> Box<Self> { Box::new(Self { addr_offset }) }
}

impl Instruction for LdhImm8A {
    fn exec(&mut self, gb: &mut Dmg) -> InstructionResult {
        let addr = IO_REGISTERS_START + (self.addr_offset as u16);
        gb.write(addr, gb.cpu.a);

        Ok(InstructionEffect::new(self.info(), Flags::none()))
    }

    fn info(&self) -> (u8, u8) { (3, 2) }
    fn disassembly(&self) -> String {
        format!("ldh [${:04X}],a", IO_REGISTERS_START + self.addr_offset as u16)
    }
}

/// Load from/to high memory area instruction
/// Usually used to access memory mapped IO and HRAM,
/// so the used addresses are between 0xFF00 and 0xFFFF
///
/// copy the src value in register A to the byte at address 0xFF00 + value in register C
pub struct LdhCA;

impl LdhCA {
    pub fn new() -> Box<Self> { Box::new(Self) }
}

impl Instruction for LdhCA {
    fn exec(&mut self, gb: &mut Dmg) -> InstructionResult {
        let addr = IO_REGISTERS_START + (gb.cpu.c as u16);
        gb.write(addr, gb.cpu.a);

        Ok(InstructionEffect::new(self.info(), Flags::none()))
    }

    fn info(&self) -> (u8, u8) { (2, 1) }
    fn disassembly(&self) -> String { "ldh [c],a".to_string() }
}

/// Load from/to high memory area instruction
/// Usually used to access memory mapped IO and HRAM,
/// so the used addresses are between 0xFF00 and 0xFFFF
///
/// copy the src byte addressed by 16 bits immediate (that must be between 0xFF00 and 0xFFFF) into dst register A
pub struct LdhAImm8 {
    pub addr_offset: u8,
}

impl LdhAImm8 {
    pub fn new(addr_offset: u8) -> Box<Self> { Box::new(Self { addr_offset }) }
}

impl Instruction for LdhAImm8 {
    fn exec(&mut self, gb: &mut Dmg) -> InstructionResult {
        let addr = IO_REGISTERS_START + (self.addr_offset as u16);
        gb.cpu.a = gb.read(addr);

        Ok(InstructionEffect::new(self.info(), Flags::none()))
    }

    fn info(&self) -> (u8, u8) { (3, 2) }
    fn disassembly(&self) -> String {
        format!("ldh a,[${:04X}]", IO_REGISTERS_START + self.addr_offset as u16)
    }
}

/// Load from/to high memory area instruction
/// Usually used to access memory mapped IO and HRAM,
/// so the used addresses are between 0xFF00 and 0xFFFF
///
/// copy the src byte addressed by 0xFF00 + C into dst register A
pub struct LdhAC;

impl LdhAC {
    pub fn new() -> Box<Self> { Box::new(Self) }
}

impl Instruction for LdhAC {
    fn exec(&mut self, gb: &mut Dmg) -> InstructionResult {
        let addr = IO_REGISTERS_START + (gb.cpu.c as u16);
        gb.cpu.a = gb.read(addr);

        Ok(InstructionEffect::new(self.info(), Flags::none()))
    }

    fn info(&self) -> (u8, u8) { (2, 1) }
    fn disassembly(&self) -> String { "ldh a,[c]".to_string() }
}
