use std::fmt::Write;

use crate::{
    core::{
        cpu::{
            R16,
            flags::{CARRY_FLAG_MASK, Flags, HALF_CARRY_FLAG_MASK, SUBTRACTION_FLAG_MASK, ZERO_FLAG_MASK},
            instructions::{
                Instruction, InstructionEffect, InstructionError, InstructionResult, InstructionTarget as IT,
            },
        },
        memory::MemoryBus,
    },
    utils::{high, low, to_u8},
};

/// Push a 16 bit register onto the stack. It is roughly equivalent to the following imaginary instructions:
/// ```asm
/// dec sp
/// ld [sp], a
/// dec sp
/// ld [sp], F.Z << 7 | F.N << 6 | F.H << 5 | F.C << 4
/// ```
/// for other 16 bits registers, it'll look the same, but without the flags logic, like this:
/// ```asm
///  dec sp
/// ld [sp], b  ; B, D or H
/// dec sp
/// ld [sp], c   ; C, E or L
pub struct Push<'a> {
    sp: &'a mut u16,
    bus: MemoryBus,
    src: IT<'a>,
}

impl<'a> Push<'a> {
    pub fn new(sp: &'a mut u16, bus: MemoryBus, src: IT<'a>) -> Box<Self> { Box::new(Self { sp, bus, src }) }
}

impl<'a> Instruction<'a> for Push<'a> {
    fn exec(&mut self) -> InstructionResult {
        let src = match &mut self.src {
            IT::Reg16(src, R16::AF) => {
                // this is probably useless because no other bit of F should be set
                let f = low(*src)
                    & (ZERO_FLAG_MASK | SUBTRACTION_FLAG_MASK | HALF_CARRY_FLAG_MASK | CARRY_FLAG_MASK);
                (f, high(*src))
            }
            IT::Reg16(src, _) => to_u8(*src),

            _ => return Err(InstructionError::MalformedInstruction),
        };

        *self.sp -= 1;
        self.bus.borrow_mut()[*self.sp] = src.1;
        *self.sp -= 1;
        self.bus.borrow_mut()[*self.sp] = src.0;

        Ok(InstructionEffect::new(4, 1, Flags::none()))
    }

    fn disassembly(&self, w: &mut dyn Write) -> Result<(), std::fmt::Error> { write!(w, "push {}", self.src) }
}

#[cfg(test)]
mod tests {

    use crate::{core::memory::Memory, utils::to_u16};

    use super::*;

    #[test]
    fn test_push_af() {
        let f = ZERO_FLAG_MASK | CARRY_FLAG_MASK | 1;
        let a: u8 = 1;
        let af = to_u16(f, a);
        let mut sp = 0xFFA0;
        let bus = Memory::new(None, None, None);

        let mut push = Push::new(&mut sp, bus.clone(), IT::Reg16(af, R16::AF));

        let effect = push.exec().unwrap();

        assert_eq!(effect.cycles, 4);
        assert_eq!(effect.len, 1);
        assert_eq!(bus.borrow()[sp], ZERO_FLAG_MASK | CARRY_FLAG_MASK);
        assert_eq!(bus.borrow()[sp + 1], a);
        assert_eq!(sp, 0xFFA0 - 2);
        assert_eq!(effect.flags, Flags::none());
    }

    #[test]
    fn test_push_bc() {
        let c = ZERO_FLAG_MASK | CARRY_FLAG_MASK | 1;
        let b = 1;
        let bc = to_u16(c, b);
        let mut sp = 0xFFA0;
        let bus = Memory::new(None, None, None);

        let mut push = Push::new(&mut sp, bus.clone(), IT::Reg16(bc, R16::BC));

        let effect = push.exec().unwrap();

        assert_eq!(effect.cycles, 4);
        assert_eq!(effect.len, 1);
        assert_eq!(bus.borrow()[sp], c);
        assert_eq!(bus.borrow()[sp + 1], b);
        assert_eq!(sp, 0xFFA0 - 2);
        assert_eq!(effect.flags, Flags::none());
    }
}
