use std::fmt::Write;

use crate::core::cpu::{
    R16,
    flags::{CARRY_FLAG_MASK, Flags, HALF_CARRY_FLAG_MASK, SUBTRACTION_FLAG_MASK, ZERO_FLAG_MASK},
    instructions::{
        Instruction, InstructionDestination as ID, InstructionEffect, InstructionError, InstructionResult,
        InstructionTarget as IT,
    },
};

/// Pop a 16 bit register from the stack.
/// Should behave like the following non-real instructions:
/// ```asm
/// ld f, [sp]
/// inc sp
/// ld a, [sp]
/// inc sp
/// ``````
pub struct Pop<'a> {
    src: IT<'a>,
    dst: ID<'a>,
}

impl<'a> Pop<'a> {
    pub fn new(dst: ID<'a>, src: IT<'a>) -> Box<Self> { Box::new(Self { dst, src }) }
}
impl<'a> Instruction<'a> for Pop<'a> {
    fn exec(&mut self) -> InstructionResult {
        let (dst, src, sp, flags) = match (&mut self.dst, &mut self.src) {
            (ID::Register16(dst, reg), IT::PointedByStackPointer(val, sp)) if *reg == R16::AF => (
                dst,
                *val,
                sp,
                // set flags according to the bits that are going to pop into F
                Flags {
                    z: Some(val.0 & ZERO_FLAG_MASK != 0),
                    n: Some(val.0 & SUBTRACTION_FLAG_MASK != 0),
                    h: Some(val.0 & HALF_CARRY_FLAG_MASK != 0),
                    c: Some(val.0 & CARRY_FLAG_MASK != 0),
                },
            ),
            (ID::Register16(dst, _), IT::PointedByStackPointer(val, sp)) => (dst, *val, sp, Flags::none()),
            _ => return Err(InstructionError::MalformedInstruction),
        };

        // pop [sp] to low register, such as F in AF
        *dst.0 = src.0;

        // pop [sp+1] to high register, such as A in AF
        *dst.1 = src.1;
        println!("a after pop: {}", dst.1);

        // increment stack pointer by 2, one for each byte popped
        **sp += 2;

        Ok(InstructionEffect::new(3, 1, flags))
    }

    fn disassembly(&self, w: &mut dyn Write) -> Result<(), std::fmt::Error> { write!(w, "pop {}", self.dst) }
}

#[cfg(test)]
mod tests {
    use crate::core::memory::Memory;

    use super::*;

    #[test]
    fn test_pop_to_af() {
        let mut f = 0;
        let mut a = 0;
        let mut sp = 0xFF00;
        let bus = Memory::new(None, None);

        bus.borrow_mut()[sp] = ZERO_FLAG_MASK | CARRY_FLAG_MASK;
        bus.borrow_mut()[sp + 1] = 1;

        let mut instr = Pop::new(
            ID::Register16((&mut f, &mut a), R16::AF),
            IT::PointedByStackPointer((bus.borrow()[sp], bus.borrow()[sp + 1]), &mut sp),
        );
        let effect = instr.exec().unwrap();

        assert_eq!(a, 1);
        assert_eq!(f, ZERO_FLAG_MASK | CARRY_FLAG_MASK);
        assert_eq!(sp, 0xFF02);
        assert_eq!(effect.cycles, 3);
        assert_eq!(effect.len, 1);
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
                z: Some(f & ZERO_FLAG_MASK != 0),
                n: Some(f & SUBTRACTION_FLAG_MASK != 0),
                h: Some(f & HALF_CARRY_FLAG_MASK != 0),
                c: Some(f & CARRY_FLAG_MASK != 0),
            }
        );
    }
}
