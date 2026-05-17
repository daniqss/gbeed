mod arithmetic;
mod bitflag;
mod bitshift;
mod bitwise;
mod carry_related;
mod interrupt_related;
mod jumps;
mod loads;
mod misc;

use core::fmt::Display;

pub use arithmetic::*;
pub use bitflag::*;
pub use bitshift::*;
pub use bitwise::*;
pub use carry_related::*;
pub use interrupt_related::*;
pub use jumps::*;
pub use loads::*;
pub use misc::*;

use crate::{cpu::flags::LazyFlags, impl_static_box, prelude::*};

/// Represents a CPU instruction.
/// The instruction can be executed and can provide its disassembly representation
pub trait Instruction {
    fn exec(&mut self, gb: &mut Dmg) -> InstructionResult;
    fn info(&self) -> (u8, u8);
    fn disassembly(&self) -> String;
}

impl_static_box!(Instruction);

impl Display for dyn Instruction {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "{}", self.disassembly())
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
pub struct InstructionEffect {
    pub cycles: u8,
    pub len: Len,
    pub flags: Option<StaticBox<dyn LazyFlags>>,
}

impl InstructionEffect {
    pub fn new(info: (u8, u8), lazy_flags: Option<StaticBox<dyn LazyFlags>>) -> Self {
        let (cycles, len) = info;
        Self {
            cycles,
            len: Len::AddLen(len),
            flags: lazy_flags,
        }
    }

    pub fn with_jump(info: (u8, u8), lazy_flags: Option<StaticBox<dyn LazyFlags>>) -> Self {
        let (cycles, len) = info;
        Self {
            cycles,
            len: Len::Jump(len),
            flags: lazy_flags,
        }
    }

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
