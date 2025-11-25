mod adc;
mod add;
mod and;
mod bit;
mod carry;
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
mod set;
mod sla;
mod sra;
mod srl;
mod stop;
mod sub;
mod swap;
mod xor;

use std::fmt::{Display, Write};

pub use adc::Adc;
pub use add::Add;
pub use and::And;
pub use bit::Bit;
pub use carry::*;
pub use cp::Cp;
pub use cpl::Cpl;
pub use daa::Daa;
pub use dec::Dec;
pub use di::Di;
pub use ei::Ei;
pub use halt::Halt;
pub use inc::Inc;
pub use jumps::*;
pub use ld::Ld;
pub use ldh::Ldh;
pub use nop::Nop;
pub use or::Or;
pub use pop::Pop;
pub use push::Push;
pub use res::Res;
pub use rl::Rl;
pub use rla::Rla;
pub use rlc::Rlc;
pub use rlca::Rlca;
pub use rr::Rr;
pub use rra::Rra;
pub use rrc::Rrc;
pub use rrca::Rrca;
pub use sbc::Sbc;
pub use set::Set;
pub use sla::Sla;
pub use sra::Sra;
pub use srl::Srl;
pub use stop::Stop;
pub use sub::Sub;
pub use swap::Swap;
pub use xor::Xor;

use crate::{
    Dmg,
    core::cpu::{R8, R16, flags::Flags},
};

/// Represents a CPU instruction.
/// The instruction can be executed and can provide its disassembly representation
pub trait Instruction {
    fn exec(&mut self, gb: &mut Dmg) -> InstructionResult;
    fn disassembly(&self, w: &mut dyn Write) -> Result<(), std::fmt::Error>;
}

impl Display for dyn Instruction {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result { self.disassembly(f) }
}

/// Instructions possible operands and targets.
/// Only used when various operand types are possible for the same instruction.
/// They store neccesary data to identify then in their display and the value that will be used during execution, and we can know by the fetching, to avoid innecesary matching and reading during instruction execution.
/// We won't pass values when they can be accessed directly without matching cpu (e.g named registers)
#[derive(Debug, PartialEq)]
pub enum InstructionTarget {
    Imm8(u8),
    Imm16(u16),
    SignedImm(i8),
    Reg8(u8, R8),
    Reg16(u16, R16),
    PointedByHL,
    PointedByN16(u16),
    PointedByCPlusFF00(u8),
    PointedByReg16(u16, R16),
    PointedByHLI,
    PointedByHLD,
    StackPointer,
    StackPointerPlusE8(i8),
    JumpToImm16(JumpCondition, u16),
    JumpToHL,
    JumpToImm8(JumpCondition, i8),
}

impl Display for InstructionTarget {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            InstructionTarget::Imm8(n8) => write!(f, "${:02X}", n8),
            InstructionTarget::Imm16(n16) => write!(f, "${:04X}", n16),
            InstructionTarget::SignedImm(e8) => write!(f, "{:+}", e8),
            InstructionTarget::Reg8(_, reg) => write!(f, "{}", reg),
            InstructionTarget::Reg16(_, reg) => write!(f, "{}", reg),
            InstructionTarget::PointedByHL => write!(f, "[hl]"),
            InstructionTarget::PointedByN16(addr) => write!(f, "[${:04X}]", addr),
            InstructionTarget::PointedByCPlusFF00(addr) => write!(f, "[${:04X}]", addr),
            InstructionTarget::PointedByReg16(_, reg) => write!(f, "[{}]", reg),
            InstructionTarget::PointedByHLI => write!(f, "[hli]"),
            InstructionTarget::PointedByHLD => write!(f, "[hld]"),
            InstructionTarget::StackPointer => write!(f, "sp"),
            InstructionTarget::StackPointerPlusE8(e8) => write!(f, "sp{:+}", e8),
            InstructionTarget::JumpToImm16(jc, n16) => write!(f, "{}${:04X}", jc, n16),
            InstructionTarget::JumpToHL => write!(f, "hl"),
            InstructionTarget::JumpToImm8(jc, e8) => write!(f, "{}{:+}", jc, e8),
        }
    }
}

#[derive(Debug)]
pub enum InstructionDestination {
    PointedByHL,
    PointedByN16(u16),
    PointedByCPlusFF00(u16),
    Reg8(R8),
    Reg16(R16),
    PointedByReg16(u16, R16),
    PointedByHLI,
    PointedByHLD,
    StackPointer,
}

impl Display for InstructionDestination {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            InstructionDestination::PointedByHL => write!(f, "[hl]"),
            InstructionDestination::PointedByN16(addr) => write!(f, "[${:04X}]", addr),
            InstructionDestination::PointedByCPlusFF00(addr) => {
                write!(f, "[${:04X}]", addr)
            }
            InstructionDestination::Reg8(reg) => write!(f, "{}", reg),
            InstructionDestination::Reg16(reg) => write!(f, "{}", reg),
            InstructionDestination::PointedByReg16(_, reg) => write!(f, "[{}]", reg),
            InstructionDestination::PointedByHLI => write!(f, "[hli]"),
            InstructionDestination::PointedByHLD => write!(f, "[hld]"),
            InstructionDestination::StackPointer => write!(f, "sp"),
        }
    }
}

/// Effect of executing a instruction
/// Instructions also "effect" their operands but those are represented as parameters using references
pub struct InstructionEffect {
    pub cycles: u8,
    pub len: u8,
    pub flags: Flags,
}

impl InstructionEffect {
    pub fn new(cycles: u8, len: u8, flags: Flags) -> Self { Self { cycles, len, flags } }
}

/// Errors that can occur during instruction execution
#[derive(Debug)]
pub enum InstructionError {
    UnusedOpcode(u8, u16),
    OutOfRangeOpcode(u8, u16),
    OutOfRangeCBOpcode(u8, u16),
    AddressOutOfRange(u16, Option<u8>, Option<u16>),
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
            InstructionError::AddressOutOfRange(address, opcode, pc) => write!(
                f,
                "Address out of range {:04X} for opcode {:02X} at PC {:04X}",
                address,
                opcode.unwrap_or(0x00),
                pc.unwrap_or(0x0000),
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

pub type InstructionResult = Result<InstructionEffect, InstructionError>;
