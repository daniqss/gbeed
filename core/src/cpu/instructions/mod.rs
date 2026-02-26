mod adc;
mod add;
mod and;
mod bit;
mod ccf;
mod cp;
mod cpl;
mod daa;
mod dec;
mod di;
mod ei;
mod halt;
mod inc;
mod jumps;
mod ld;
mod ldh;
mod nop;
mod or;
mod pop;
mod push;
mod res;
mod rl;
mod rla;
mod rlc;
mod rlca;
mod rr;
mod rra;
mod rrc;
mod rrca;
mod sbc;
mod scf;
mod set;
mod sla;
mod sra;
mod srl;
mod stop;
mod sub;
mod swap;
mod xor;

use std::fmt::Display;

pub use adc::*;
pub use add::*;
pub use and::*;
pub use bit::*;
pub use ccf::Ccf;
pub use cp::*;
pub use cpl::Cpl;
pub use daa::Daa;
pub use dec::*;
pub use di::Di;
pub use ei::Ei;
pub use halt::Halt;
pub use inc::*;
pub use jumps::*;
pub use ld::*;
pub use ldh::*;
pub use nop::Nop;
pub use or::*;
pub use pop::Pop;
pub use push::Push;
pub use res::*;
pub use rl::*;
pub use rla::Rla;
pub use rlc::*;
pub use rlca::Rlca;
pub use rr::*;
pub use rra::Rra;
pub use rrc::*;
pub use rrca::Rrca;
pub use sbc::*;
pub use scf::Scf;
pub use set::*;
pub use sla::*;
pub use sra::*;
pub use srl::*;
pub use stop::Stop;
pub use sub::*;
pub use swap::*;
pub use xor::*;

use crate::{cpu::flags::Flags, prelude::*};

/// Represents a CPU instruction.
/// The instruction can be executed and can provide its disassembly representation
pub trait Instruction {
    fn exec(&mut self, gb: &mut Dmg) -> InstructionResult;
    fn info(&self) -> (u8, u8);
    fn disassembly(&self) -> String;
}

impl Display for dyn Instruction {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result { write!(f, "{}", self.disassembly()) }
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

impl std::fmt::Display for InstructionError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
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

pub type InstructionResult = std::result::Result<InstructionEffect, InstructionError>;
