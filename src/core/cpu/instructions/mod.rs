mod adc;
mod ld;
mod ldh;

use std::fmt::{Display, Write};

pub use adc::*;
pub use ld::*;
pub use ldh::*;

use crate::core::cpu::{R8, R16};

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

#[derive(Debug, PartialEq)]
pub enum InstructionDestination<'a> {
    PointedByHL(&'a mut u8),
    PointedByN16(&'a mut u8, u16),
    PointedByN16AndNext((&'a mut u8, &'a mut u8), u16),
    PointedByCPlusFF00(&'a mut u8, u16),
    Register8(&'a mut u8, R8),
    Register16((&'a mut u8, &'a mut u8), R16),
    PointedByRegister16(&'a mut u8, R16),
    PointedByHLI(&'a mut u8, (&'a mut u8, &'a mut u8)),
    PointedByHLD(&'a mut u8, (&'a mut u8, &'a mut u8)),
    StackPointer(&'a mut u16),
}

impl Display for InstructionDestination<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            InstructionDestination::PointedByHL(_) => write!(f, "[hl]"),
            InstructionDestination::PointedByN16(_, address) => write!(f, "[${:04X}]", address),
            InstructionDestination::PointedByN16AndNext(_, address)
            | InstructionDestination::PointedByCPlusFF00(_, address) => {
                write!(f, "[${:04X}]", address)
            }
            InstructionDestination::Register8(_, reg) => write!(f, "{}", reg),
            InstructionDestination::Register16(_, reg) => write!(f, "{}", reg),
            InstructionDestination::PointedByRegister16(_, reg) => write!(f, "[{}]", reg),
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
    pub flags: Option<u8>,
}

impl InstructionEffect {
    pub fn new(cycles: u8, len: u8, flags: Option<u8>) -> Self { Self { cycles, len, flags } }
}

/// Errors that can occur during instruction execution
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
