use crate::{
    cpu::{
        R8, R16,
        flags::{
            HALF_CARRY_FLAG_MASK, LazyFlags, SUBTRACTION_FLAG_MASK, ZERO_FLAG_MASK, check_overflow_hc,
            check_zero,
        },
        instructions::{Instruction, InstructionEffect, InstructionResult},
    },
    prelude::*,
};

#[derive(Debug, Default, Clone, Copy)]
pub struct IncR8 {
    dst: R8,
}
impl IncR8 {
    pub fn new(dst: R8) -> StaticBox<Self> { StaticBox::new(Self { dst }) }
}

impl Instruction for IncR8 {
    fn exec(&mut self, gb: &mut Dmg) -> InstructionResult {
        let r8 = gb.read(self.dst);
        let result = r8.wrapping_add(1);
        gb.write(self.dst, result);

        Ok(InstructionEffect::new(
            self.info(),
            Some(IncU8Flags::new(r8, result).into()),
        ))
    }
    fn info(&self) -> (u8, u8) { (1, 1) }
    fn disassembly(&self) -> String { format!("inc {}", self.dst) }
}

#[derive(Debug, Default, Clone, Copy)]
pub struct IncPointedByHL;
impl IncPointedByHL {
    pub fn new() -> StaticBox<Self> { StaticBox::new(Self) }
}

impl Instruction for IncPointedByHL {
    fn exec(&mut self, gb: &mut Dmg) -> InstructionResult {
        let n8 = gb.read(gb.cpu.hl());
        let result = n8.wrapping_add(1);
        gb.write(gb.cpu.hl(), result);

        Ok(InstructionEffect::new(
            self.info(),
            Some(IncU8Flags::new(n8, result).into()),
        ))
    }
    fn info(&self) -> (u8, u8) { (3, 1) }
    fn disassembly(&self) -> String { "inc [hl]".to_string() }
}

#[derive(Debug, Default, Clone, Copy)]
pub struct IncR16 {
    dst: R16,
}
impl IncR16 {
    pub fn new(dst: R16) -> StaticBox<Self> { StaticBox::new(Self { dst }) }
}

impl Instruction for IncR16 {
    fn exec(&mut self, gb: &mut Dmg) -> InstructionResult {
        let r16 = gb.load(self.dst);
        let result = r16.wrapping_add(1);
        gb.store(self.dst, result);

        Ok(InstructionEffect::new(self.info(), None))
    }
    fn info(&self) -> (u8, u8) { (2, 1) }
    fn disassembly(&self) -> String { format!("inc {}", self.dst) }
}

#[derive(Debug, Default, Clone, Copy)]
pub struct IncStackPointer;
impl IncStackPointer {
    pub fn new() -> StaticBox<Self> { StaticBox::new(Self) }
}

impl Instruction for IncStackPointer {
    fn exec(&mut self, gb: &mut Dmg) -> InstructionResult {
        gb.cpu.sp = gb.cpu.sp.wrapping_add(1);

        Ok(InstructionEffect::new(self.info(), None))
    }
    fn info(&self) -> (u8, u8) { (2, 1) }
    fn disassembly(&self) -> String { "inc sp".to_string() }
}

#[derive(Debug, Default, Clone, Copy)]
struct IncU8Flags {
    old: u8,
    result: u8,
}

impl IncU8Flags {
    fn new(old: u8, result: u8) -> StaticBox<Self> { StaticBox::new(Self { old, result }) }
}

impl LazyFlags for IncU8Flags {
    fn updated_flags(&self) -> u8 { ZERO_FLAG_MASK | SUBTRACTION_FLAG_MASK | HALF_CARRY_FLAG_MASK }

    fn zero(&self) -> bool { check_zero(self.result) }
    fn subtraction(&self) -> bool { false }
    fn half_carry(&self) -> bool { check_overflow_hc(self.result, self.old) }
}
