use crate::{
    Dmg,
    core::{
        Accessible16,
        cpu::{
            R8, R16,
            flags::{Flags, check_overflow_hc, check_zero},
            instructions::{Instruction, InstructionEffect, InstructionResult},
        },
        memory::Accessible,
    },
};

#[inline(always)]
fn inc_u8_flags(old: u8, result: u8) -> Flags {
    Flags {
        z: Some(check_zero(result)),
        n: Some(false),
        h: Some(check_overflow_hc(result, old)),
        c: None,
    }
}

pub struct IncR8 {
    dst: R8,
}
impl IncR8 {
    pub fn new(dst: R8) -> Box<Self> { Box::new(Self { dst }) }
}

impl Instruction for IncR8 {
    fn exec(&mut self, gb: &mut Dmg) -> InstructionResult {
        let r8 = gb.read(self.dst);
        let result = r8.wrapping_add(1);
        gb.write(self.dst, result);

        Ok(InstructionEffect::new(self.info(gb), inc_u8_flags(r8, result)))
    }
    fn info(&self, _: &mut Dmg) -> (u8, u8) { (1, 1) }
    fn disassembly(&self) -> String { format!("inc {}", self.dst) }
}

pub struct IncPointedByHL;
impl IncPointedByHL {
    pub fn new() -> Box<Self> { Box::new(Self) }
}

impl Instruction for IncPointedByHL {
    fn exec(&mut self, gb: &mut Dmg) -> InstructionResult {
        let n8 = gb.read(gb.cpu.hl());
        let result = n8.wrapping_add(1);
        gb.write(gb.cpu.hl(), result);

        Ok(InstructionEffect::new(self.info(gb), inc_u8_flags(n8, result)))
    }
    fn info(&self, _: &mut Dmg) -> (u8, u8) { (3, 1) }
    fn disassembly(&self) -> String { format!("inc [hl]") }
}

pub struct IncR16 {
    dst: R16,
}
impl IncR16 {
    pub fn new(dst: R16) -> Box<Self> { Box::new(Self { dst }) }
}

impl Instruction for IncR16 {
    fn exec(&mut self, gb: &mut Dmg) -> InstructionResult {
        let r16 = gb.load(self.dst);
        let result = r16.wrapping_add(1);
        gb.store(self.dst, result);

        Ok(InstructionEffect::new(self.info(gb), Flags::none()))
    }
    fn info(&self, _: &mut Dmg) -> (u8, u8) { (2, 1) }
    fn disassembly(&self) -> String { format!("inc {}", self.dst) }
}

pub struct IncStackPointer;
impl IncStackPointer {
    pub fn new() -> Box<Self> { Box::new(Self) }
}

impl Instruction for IncStackPointer {
    fn exec(&mut self, gb: &mut Dmg) -> InstructionResult {
        gb.cpu.sp = gb.cpu.sp.wrapping_add(1);

        Ok(InstructionEffect::new(self.info(gb), Flags::none()))
    }
    fn info(&self, _: &mut Dmg) -> (u8, u8) { (2, 1) }
    fn disassembly(&self) -> String { format!("inc sp") }
}
