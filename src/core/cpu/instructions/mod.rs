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

pub use adc::Adc;
pub use add::Add;
pub use and::And;
pub use bit::Bit;
pub use ccf::Ccf;
pub use cp::Cp;
pub use cpl::Cpl;
pub use daa::Daa;
pub use dec::*;
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
pub use scf::Scf;
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
    core::{
        IO_REGISTERS_START,
        cpu::{R8, R16, flags::Flags},
    },
};

/// Represents a CPU instruction.
/// The instruction can be executed and can provide its disassembly representation
pub trait Instruction {
    fn exec(&mut self, gb: &mut Dmg) -> InstructionResult;
    fn info(&self, gb: &mut Dmg) -> (u8, u8);
    fn disassembly(&self) -> String;
}

impl Display for dyn Instruction {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result { write!(f, "{}", self.disassembly()) }
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
    PointedByHL(u8),
    PointedByN16(u16),
    PointedByA8(u8),
    PointedByCPlusFF00,
    PointedByReg16(u8, R16),
    PointedByHLI(u8),
    PointedByHLD(u8),
    StackPointer(u16),
    StackPointerPlusE8(u16, i8),
    JumpToImm16(JumpCondition, u16),
    JumpToHL(u16),
    JumpToImm8(JumpCondition, u8),
}

impl Display for InstructionTarget {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            InstructionTarget::Imm8(n8) => write!(f, "${:02X}", n8),
            InstructionTarget::Imm16(n16) => write!(f, "${:04X}", n16),
            InstructionTarget::SignedImm(e8) => write!(f, "{:+}", e8),
            InstructionTarget::Reg8(_, reg) => write!(f, "{}", reg),
            InstructionTarget::Reg16(_, reg) => write!(f, "{}", reg),
            InstructionTarget::PointedByHL(_) => write!(f, "[hl]"),
            InstructionTarget::PointedByN16(addr) => write!(f, "[${:04X}]", addr),
            InstructionTarget::PointedByA8(addr) => write!(f, "[${:04X}]", IO_REGISTERS_START + *addr as u16),
            InstructionTarget::PointedByCPlusFF00 => write!(f, "[c]"),
            InstructionTarget::PointedByReg16(_, reg) => write!(f, "[{}]", reg),
            InstructionTarget::PointedByHLI(_) => write!(f, "[hli]"),
            InstructionTarget::PointedByHLD(_) => write!(f, "[hld]"),
            InstructionTarget::StackPointer(_) => write!(f, "sp"),
            InstructionTarget::StackPointerPlusE8(_, e8) => write!(f, "sp{:+}", e8),
            InstructionTarget::JumpToImm16(jc, n16) => write!(f, "{}${:04X}", jc, n16),
            InstructionTarget::JumpToHL(_) => write!(f, "hl"),
            InstructionTarget::JumpToImm8(jc, e8) => write!(f, "{}{}", jc, *e8 as i8),
        }
    }
}

#[derive(Debug)]
pub enum InstructionDestination {
    PointedByHL(u16),
    PointedByN16(u16),
    PointedByA8(u8),
    PointedByCPlusFF00,
    Reg8(R8),
    Reg16(R16),
    PointedByReg16(u16, R16),
    PointedByHLI(u16),
    PointedByHLD(u16),
    StackPointer,
}

impl Display for InstructionDestination {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            InstructionDestination::PointedByHL(_) => write!(f, "[hl]"),
            InstructionDestination::PointedByN16(addr) => write!(f, "[${:04X}]", addr),
            InstructionDestination::PointedByA8(addr) => {
                write!(f, "[${:04X}]", IO_REGISTERS_START + *addr as u16)
            }
            InstructionDestination::PointedByCPlusFF00 => {
                write!(f, "[c]")
            }
            InstructionDestination::Reg8(reg) => write!(f, "{}", reg),
            InstructionDestination::Reg16(reg) => write!(f, "{}", reg),
            InstructionDestination::PointedByReg16(_, reg) => write!(f, "[{}]", reg),
            InstructionDestination::PointedByHLI(_) => write!(f, "[hli]"),
            InstructionDestination::PointedByHLD(_) => write!(f, "[hld]"),
            InstructionDestination::StackPointer => write!(f, "sp"),
        }
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

pub type InstructionResult = Result<InstructionEffect, InstructionError>;
