use crate::core::cpu::{
    flags::{CARRY_FLAG_MASK, Flags, check_zero},
    instructions::{
        Instruction, InstructionDestination as ID, InstructionEffect, InstructionError, InstructionResult,
    },
};

/// rotate bits left between r8 and carry flag
///   ┏━ Flags ━┓ ┏━━━━━━━ r8 | [hl] ━━━━━━┓
/// ┌─╂─   C   ←╂─╂─  b7  ←   ...  ←  b0  ←╂─┐
/// │ ┗━━━━━━━━━┛ ┗━━━━━━━━━━━━━━━━━━━━━━━━┛ │
/// └────────────────────────────────────────┘
pub struct Rl<'a> {
    carry: u8,
    dst: ID<'a>,
}

impl<'a> Rl<'a> {
    pub fn new(carry: u8, dst: ID<'a>) -> Box<Self> { Box::new(Rl { carry, dst }) }
}

impl<'a> Instruction<'a> for Rl<'a> {
    fn exec(&mut self) -> InstructionResult {
        let (dst, cycles, len): (&mut u8, u8, u8) = match &mut self.dst {
            ID::Reg8(r8, _) => (r8, 2, 2),
            ID::PointedByHL(bus, addr) => (&mut bus.borrow_mut()[*addr], 4, 2),

            _ => return Err(InstructionError::MalformedInstruction),
        };

        let result = (*dst << 1) | if self.carry & CARRY_FLAG_MASK != 0 { 1 } else { 0 };
        let flags = Flags {
            z: Some(check_zero(result)),
            n: Some(false),
            h: Some(false),
            c: Some(*dst & 0b1000_0000 != 0),
        };
        *dst = result;

        Ok(InstructionEffect::new(cycles, len, flags))
    }

    fn disassembly(&self, w: &mut dyn std::fmt::Write) -> Result<(), std::fmt::Error> {
        write!(w, "rl {}", self.dst)
    }
}

#[cfg(test)]
mod tests {
    use crate::core::{
        cpu::{R8, flags::Flags},
        memory::Memory,
    };

    use super::*;

    #[test]
    fn test_rl_no_carry() {
        let mut a = 0b1000_0000;
        let mut instr = Rl::new(0, ID::Reg8(&mut a, R8::A));

        let result = instr.exec().unwrap();
        assert_eq!(a, 0b0000_0000);

        assert_eq!(result.cycles, 2);
        assert_eq!(result.len, 2);
        assert_eq!(
            result.flags,
            Flags {
                z: Some(true),
                n: Some(false),
                h: Some(false),
                c: Some(true),
            }
        );
    }

    #[test]
    fn test_rl_with_carry() {
        let addr = 0xFF00;
        let value = 0b0011_1000;
        let bus = Memory::new(None, None);
        bus.borrow_mut()[addr] = value;

        let mut instr = Rl::new(CARRY_FLAG_MASK, ID::PointedByHL(bus.clone(), addr));

        let result = instr.exec().unwrap();
        assert_eq!(bus.borrow()[addr], 0b0111_0001);

        assert_eq!(result.cycles, 4);
        assert_eq!(result.len, 2);
        assert_eq!(
            result.flags,
            Flags {
                z: Some(false),
                n: Some(false),
                h: Some(false),
                c: Some(false),
            }
        );
    }
}
