use std::fmt::Write;

use crate::{
    Dmg,
    core::cpu::{
        Reg,
        flags::{CARRY_FLAG_MASK, Flags, HALF_CARRY_FLAG_MASK, SUBTRACTION_FLAG_MASK, ZERO_FLAG_MASK},
        instructions::{
            Instruction, InstructionEffect, InstructionError, InstructionResult, InstructionTarget as IT,
        },
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
pub struct Push {
    src: IT,
}

impl Push {
    pub fn new(src: IT) -> Box<Self> { Box::new(Self { src }) }
}

impl Instruction for Push {
    fn exec(&mut self, gb: &mut Dmg) -> InstructionResult {
        let src = match &mut self.src {
            IT::Reg16(src, Reg::AF) => {
                // this is probably useless because no other bit of F should be set
                let f = low(*src)
                    & (ZERO_FLAG_MASK | SUBTRACTION_FLAG_MASK | HALF_CARRY_FLAG_MASK | CARRY_FLAG_MASK);
                (f, high(*src))
            }
            IT::Reg16(src, _) => to_u8(*src),

            _ => return Err(InstructionError::MalformedInstruction),
        };

        // let src = to_u16(src.0, src.1);
        let mut sp = gb.cpu.sp.wrapping_sub(1);
        gb[sp] = src.1;
        sp = sp.wrapping_sub(1);
        gb[sp] = src.0;
        gb.cpu.sp = sp;
        // with u16 functions
        // gb.cpu.sp = gb.cpu.sp.wrapping_sub(2);
        // gb.store(gb.cpu.sp, src);

        Ok(InstructionEffect::new(4, 1, Flags::none()))
    }

    fn disassembly(&self, w: &mut dyn Write) -> Result<(), std::fmt::Error> { write!(w, "push {}", self.src) }
}

#[cfg(test)]
mod tests {

    use super::*;

    #[test]
    fn test_push_af() {
        let mut gb = Dmg::default();
        gb.cpu.f = ZERO_FLAG_MASK | CARRY_FLAG_MASK | 1;
        gb.cpu.a = 1;
        gb.cpu.sp = 0xFFA0;

        let mut push = Push::new(IT::Reg16(gb.cpu.af(), Reg::AF));

        let effect = push.exec(&mut gb).unwrap();

        assert_eq!(effect.cycles, 4);
        assert_eq!(effect.len(), 1);
        assert_eq!(gb[gb.cpu.sp], ZERO_FLAG_MASK | CARRY_FLAG_MASK);
        assert_eq!(gb[gb.cpu.sp + 1], gb.cpu.a);
        assert_eq!(gb.cpu.sp, 0xFFA0 - 2);
        assert_eq!(effect.flags, Flags::none());
    }

    #[test]
    fn test_push_bc() {
        let mut gb = Dmg::default();
        gb.cpu.c = ZERO_FLAG_MASK | CARRY_FLAG_MASK | 1;
        gb.cpu.b = 1;
        gb.cpu.sp = 0xFFA0;

        let mut push = Push::new(IT::Reg16(gb.cpu.bc(), Reg::BC));

        let effect = push.exec(&mut gb).unwrap();

        assert_eq!(effect.cycles, 4);
        assert_eq!(effect.len(), 1);
        assert_eq!(gb[gb.cpu.sp], gb.cpu.c);
        assert_eq!(gb[gb.cpu.sp + 1], gb.cpu.b);
        assert_eq!(gb.cpu.sp, 0xFFA0 - 2);
        assert_eq!(effect.flags, Flags::none());
    }
}
