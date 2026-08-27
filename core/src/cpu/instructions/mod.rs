mod arithmetic;
mod bits;
mod flags;
mod interrupts;
mod jumps;
mod load;
mod logic;
mod misc;
mod operands;
mod shift;
mod stack;

pub use arithmetic::*;
pub use bits::*;
pub use flags::*;
pub use interrupts::*;
pub use jumps::*;
pub use load::*;
pub use logic::*;
pub use misc::*;
pub use operands::*;
pub use shift::*;
pub use stack::*;

use crate::{
    cpu::{R8, R16, flags::Flags},
    prelude::*,
};

/// Represents a CPU instruction.
/// The instruction can be executed and can provide its disassembly representation
pub trait Instruction {
    /// Executes the instruction, changing the gb state and returning the effect of the instruction
    fn exec(&mut self, gb: &mut Dmg) -> InstructionResult;
    /// Tuple of cycles and length of the instruction in bytes
    fn info(&self) -> (u8, u8);
    /// Returns the disassembly representation of the instruction
    fn disassembly(&self) -> String;
}

instruction_dispatch! {
    pub enum Instructions {
        Adc<Imm8>, Adc<PointedByHL>, Adc<R8>, AddA<Imm8>, AddA<PointedByHL>, AddA<R8>,
        AddHL<R16>, AddHL<StackPointer>, AddSPImm8, And<Imm8>, And<PointedByHL>, And<R8>,
        Bit<PointedByHL>, Bit<R8>, Call, Ccf, Cp<Imm8>, Cp<PointedByHL>, Cp<R8>, Cpl, Daa,
        Dec<PointedByHL>, Dec<R8>, Dec16<R16>, Dec16<StackPointer>, Di, Ei, Halt,
        Inc<PointedByHL>, Inc<R8>, Inc16<R16>, Inc16<StackPointer>, JpToHL, JpToImm16, Jr,
        Ld<PointedByHL, Imm8>, Ld<PointedByHL, R8>, Ld<PointedByImm16, R8>, Ld<PointedByR16, R8>,
        Ld<R8, Imm8>, Ld<R8, PointedByHL>, Ld<R8, PointedByImm16>, Ld<R8, PointedByR16>,
        Ld<R8, R8>, Ld16<R16>, Ld16<StackPointer>, LdAPointedByHLDec, LdAPointedByHLInc,
        Ldh<PointedByC, R8>, Ldh<PointedByHighImm8, R8>, Ldh<R8, PointedByC>,
        Ldh<R8, PointedByHighImm8>, LdHLSPPlusImm8, LdImm16SP, LdPointedByHLDecA,
        LdPointedByHLIncA, LdSPHL, Nop, Or<Imm8>, Or<PointedByHL>, Or<R8>, Pop, Push,
        Res<PointedByHL>, Res<R8>, Ret, Reti,
        Rl<PointedByHL>, Rl<R8>, Rla, Rlc<PointedByHL>, Rlc<R8>, Rlca, Rr<PointedByHL>, Rr<R8>,
        Rra, Rrc<PointedByHL>, Rrc<R8>, Rrca, Rst, Sbc<Imm8>, Sbc<PointedByHL>, Sbc<R8>, Scf,
        Set<PointedByHL>, Set<R8>, Sla<PointedByHL>, Sla<R8>, Sra<PointedByHL>, Sra<R8>,
        Srl<PointedByHL>, Srl<R8>, Stop, Sub<Imm8>, Sub<PointedByHL>, Sub<R8>,
        Swap<PointedByHL>, Swap<R8>, Xor<Imm8>, Xor<PointedByHL>, Xor<R8>,
    }
}

/// solves the issue of overriding jumps with instruction length addition to pc
#[derive(Debug)]
pub enum Len {
    Jump(u8),
    AddLen(u8),
}

/// Effect of executing a instruction
/// Instructions also "effect" their operands but those are represented as parameters using references
#[derive(Debug)]
pub struct InstructionEffect {
    pub cycles: u8,
    pub len: Len,
    pub flags: Flags,
}

impl InstructionEffect {
    pub fn new(info: (u8, u8), flags: Flags) -> Self {
        let (cycles, len) = info;
        Self {
            cycles,
            len: Len::AddLen(len),
            flags,
        }
    }

    pub fn with_jump(info: (u8, u8), flags: Flags) -> Self {
        let (cycles, len) = info;
        Self {
            cycles,
            len: Len::Jump(len),
            flags,
        }
    }

    /// Only read by the instruction tests, the interpreter matches on `len` directly because
    /// a jump has to leave the program counter alone
    #[allow(dead_code)]
    pub fn len(&self) -> u8 {
        match &self.len {
            Len::Jump(len) => *len,
            Len::AddLen(len) => *len,
        }
    }
}

/// Errors that can occur during instruction execution
#[derive(Debug)]
pub enum InstructionError {
    UnusedOpcode(u8, u16),
    OutOfRangeOpcode(u8, u16),
    OutOfRangeCBOpcode(u8, u16),
    AddressOutOfRange { addr: u16, op: u8, pc: u16 },
    NotImplemented(u8, u16),
    MalformedInstruction,
}

impl core::fmt::Display for InstructionError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            InstructionError::UnusedOpcode(opcode, pc) => {
                write!(f, "Unused opcode {:02X} at PC {:04X}", opcode, pc)
            }
            InstructionError::OutOfRangeOpcode(opcode, pc) => {
                write!(f, "Out of range opcode {:02X} at PC {:04X}", opcode, pc)
            }
            InstructionError::OutOfRangeCBOpcode(opcode, pc) => {
                write!(f, "Out of range CB opcode {:02X} at PC {:04X}", opcode, pc)
            }
            InstructionError::AddressOutOfRange { addr, op, pc } => write!(
                f,
                "Address out of range {:04X} for opcode {:02X} at PC {:04X}",
                addr, op, pc
            ),
            InstructionError::NotImplemented(opcode, pc) => {
                write!(f, "Opcode not implemented {:02X} at PC {:04X}", opcode, pc)
            }
            InstructionError::MalformedInstruction => write!(
                f,
                "Opcode corresponds to a valid instruction, but illegal operands were used"
            ),
        }
    }
}

pub type InstructionResult = core::result::Result<InstructionEffect, InstructionError>;

#[cfg(feature = "std")]
impl std::error::Error for InstructionError {}
