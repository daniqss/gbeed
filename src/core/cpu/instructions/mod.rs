mod adc;
mod add;
mod and;
mod bit;
mod cp;
mod cpl;
mod daa;
mod dec;
mod di;
mod ei;
mod halt;
mod inc;
mod ld;
mod ldh;
mod nop;
mod or;
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
pub use cp::Cp;
pub use cpl::Cpl;
pub use daa::Daa;
pub use dec::Dec;
pub use di::Di;
pub use ei::Ei;
pub use halt::Halt;
pub use inc::Inc;
pub use ld::Ld;
pub use ldh::Ldh;
pub use nop::Nop;
pub use or::Or;
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

use crate::core::{
    cpu::{R8, R16, flags::Flags},
    memory::MemoryBus,
};

/// Represents a CPU instruction.
/// The instruction can be executed and can provide its disassembly representation
pub trait Instruction<'a> {
    fn exec(&mut self) -> InstructionResult;
    fn disassembly(&self, w: &mut dyn Write) -> Result<(), std::fmt::Error>;
}

impl Display for dyn Instruction<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result { self.disassembly(f) }
}

/// Instructions possible operands and targets.
/// Only used when various operand types are possible for the same instruction.
/// maybe we should remove non Dst variants, use always references an use it as mutable or not depending on the context
#[derive(Debug, PartialEq)]
pub enum InstructionTarget<'a> {
    Immediate8(u8),
    Immediate16(u16),
    SignedImm(i8),
    Register8(u8, R8),
    Register16((u8, u8), R16),
    PointedByHL(u8),
    PointedByN16(u8, u16),
    PointedByCPlusFF00(u8, u16),
    PointedByRegister16(u8, R16),
    PointedByHLI(u8, (&'a mut u8, &'a mut u8)),
    PointedByHLD(u8, (&'a mut u8, &'a mut u8)),
    StackPointer(u16),
    StackPointerPlusE8(u16, i8),
}

impl Display for InstructionTarget<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            InstructionTarget::Immediate8(n8) => write!(f, "${:02X}", n8),
            InstructionTarget::Immediate16(n16) => write!(f, "${:04X}", n16),
            InstructionTarget::SignedImm(e8) => write!(f, "{:+}", e8),
            InstructionTarget::Register8(_, reg) => write!(f, "{}", reg),
            InstructionTarget::Register16(_, reg) => write!(f, "{}", reg),
            InstructionTarget::PointedByHL(_) => write!(f, "[hl]"),
            InstructionTarget::PointedByN16(_, address) => write!(f, "[${:04X}]", address),
            InstructionTarget::PointedByCPlusFF00(_, address) => write!(f, "[${:04X}]", address),
            InstructionTarget::PointedByRegister16(_, reg) => write!(f, "[{}]", reg),
            InstructionTarget::PointedByHLI(_, _) => write!(f, "[hli]"),
            InstructionTarget::PointedByHLD(_, _) => write!(f, "[hld]"),
            InstructionTarget::StackPointer(_) => write!(f, "sp"),
            InstructionTarget::StackPointerPlusE8(_, e8) => write!(f, "sp+{:+}", e8),
        }
    }
}

#[derive(Debug)]
pub enum InstructionDestination<'a> {
    PointedByHL(MemoryBus, u16),
    PointedByN16(MemoryBus, u16),
    PointedByN16AndNext(MemoryBus, u16),
    PointedByCPlusFF00(MemoryBus, u16),
    Register8(&'a mut u8, R8),
    Register16((&'a mut u8, &'a mut u8), R16),
    PointedByRegister16(MemoryBus, u16, R16),
    PointedByHLI(MemoryBus, (&'a mut u8, &'a mut u8)),
    PointedByHLD(MemoryBus, (&'a mut u8, &'a mut u8)),
    StackPointer(&'a mut u16),
}

impl Display for InstructionDestination<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            InstructionDestination::PointedByHL(_, _) => write!(f, "[hl]"),
            InstructionDestination::PointedByN16(_, address) => write!(f, "[${:04X}]", address),
            InstructionDestination::PointedByN16AndNext(_, address)
            | InstructionDestination::PointedByCPlusFF00(_, address) => {
                write!(f, "[${:04X}]", address)
            }
            InstructionDestination::Register8(_, reg) => write!(f, "{}", reg),
            InstructionDestination::Register16(_, reg) => write!(f, "{}", reg),
            InstructionDestination::PointedByRegister16(_, _, reg) => write!(f, "[{}]", reg),
            InstructionDestination::PointedByHLI(_, _) => write!(f, "[hli]"),
            InstructionDestination::PointedByHLD(_, _) => write!(f, "[hld]"),
            InstructionDestination::StackPointer(_) => write!(f, "sp"),
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
    NoOp(u8, u16),
    UnusedOpcode(u8, u16),
    AddressOutOfRange(u16, Option<u8>, Option<u16>),
    NotImplemented(u8, u16),
    MalformedInstruction,
}

impl std::fmt::Display for InstructionError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            InstructionError::NoOp(opcode, pc) => {
                write!(f, "No operation for opcode {:02X} at PC {:04X}", opcode, pc)
            }
            InstructionError::UnusedOpcode(opcode, pc) => {
                write!(f, "Unused opcode {:02X} at PC {:04X}", opcode, pc)
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
