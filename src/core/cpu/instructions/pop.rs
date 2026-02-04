use crate::{
    Dmg,
    core::{
        cpu::{
            R16,
            flags::{CARRY_FLAG_MASK, Flags, HALF_CARRY_FLAG_MASK, SUBTRACTION_FLAG_MASK, ZERO_FLAG_MASK},
            instructions::{Instruction, InstructionEffect, InstructionResult},
        },
        memory::Accessible16,
    },
    utils::low,
};

#[inline(always)]
fn flags_pop(dst: R16, src: u16) -> Flags {
    match dst {
        R16::AF => Flags {
            z: Some(low(src) & ZERO_FLAG_MASK != 0),
            n: Some(low(src) & SUBTRACTION_FLAG_MASK != 0),
            h: Some(low(src) & HALF_CARRY_FLAG_MASK != 0),
            c: Some(low(src) & CARRY_FLAG_MASK != 0),
        },
        _ => Flags::none(),
    }
}

/// Pop a 16 bit register from the stack.
/// Should behave like the following non-real instructions (for AF register, but its the same for the other 16 bit registers):
/// ```asm
/// ld f, [sp]
/// inc sp
/// ld a, [sp]
/// inc sp
/// ``````
pub struct Pop {
    dst: R16,
}

impl Pop {
    pub fn new(dst: R16) -> Box<Self> { Box::new(Self { dst }) }
}
impl Instruction for Pop {
    fn exec(&mut self, gb: &mut Dmg) -> InstructionResult {
        let src = gb.load(gb.cpu.sp);

        // pop from stack to register
        gb.store(self.dst, src);

        // increment stack pointer by 2, one for each byte popped
        gb.cpu.sp = gb.cpu.sp.wrapping_add(2);

        Ok(InstructionEffect::new(self.info(gb), flags_pop(self.dst, src)))
    }
    fn info(&self, _: &mut Dmg) -> (u8, u8) { (3, 1) }
    fn disassembly(&self) -> String { format!("pop {}", self.dst) }
}

#[cfg(test)]
mod tests {

    use crate::core::Accessible;

    use super::*;

    #[test]
    fn test_pop_to_af() {
        let mut gb = Dmg::default();
        gb.cpu.f = 0;
        gb.cpu.a = 0;
        gb.cpu.sp = 0xFF00;

        let sp = gb.cpu.sp;
        gb.write(sp, ZERO_FLAG_MASK | CARRY_FLAG_MASK);
        gb.write(sp + 1, 1);

        let mut instr = Pop::new(R16::AF);
        let effect = instr.exec(&mut gb).unwrap();

        assert_eq!(gb.cpu.a, 1);
        assert_eq!(gb.cpu.f, ZERO_FLAG_MASK | CARRY_FLAG_MASK);
        assert_eq!(gb.cpu.sp, 0xFF02);
        assert_eq!(effect.cycles, 3);
        assert_eq!(effect.len(), 1);
        assert_eq!(
            effect.flags,
            Flags {
                z: Some(true),
                n: Some(false),
                h: Some(false),
                c: Some(true),
            }
        );
        assert_eq!(
            effect.flags,
            Flags {
                z: Some(gb.cpu.f & ZERO_FLAG_MASK != 0),
                n: Some(gb.cpu.f & SUBTRACTION_FLAG_MASK != 0),
                h: Some(gb.cpu.f & HALF_CARRY_FLAG_MASK != 0),
                c: Some(gb.cpu.f & CARRY_FLAG_MASK != 0),
            }
        );
    }
}
