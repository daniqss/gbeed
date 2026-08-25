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
    /// Executes the instruction, changing the gb state and returning the effect of the instruction
    fn exec(&mut self, gb: &mut Dmg) -> InstructionResult;
    /// Tuple of cycles and length of the instruction in bytes
    fn info(&self) -> (u8, u8);
    /// Returns the disassembly representation of the instruction
    fn disassembly(&self) -> String;
}

instruction_dispatch! {
    pub enum Instructions {
        AdcImm8, AdcPointedByHL, AdcR8, AddAPointedByHL, AddAR8, AddHLSP, AddImm8, AddR16,
        AddSPImm8, AndImm8, AndPointedByHL, AndR8, BitPointedByHL, BitR8, Call, Ccf, CpImm8,
        Cpl, CpPointedByHL, CpR8, Daa, DecPointedByHL, DecR16, DecR8, DecStackPointer, Di, Ei,
        Halt, IncPointedByHL, IncR16, IncR8, IncStackPointer, JpToHL, JpToImm16, Jr,
        LdAPointedByHLDec, LdAPointedByHLInc, LdAPointedByImm16, LdAPointedByR16, LdhAC,
        LdhAImm8, LdhCA, LdhImm8A, LdHLSPPlusImm8, LdImm16SP, LdPointedByHLDecA,
        LdPointedByHLImm8, LdPointedByHLIncA, LdPointedByHLR8, LdPointedByImm16A,
        LdPointedByR16A, LdR16Imm16, LdR8Imm8, LdR8PointedByHL, LdR8R8, LdSPHL, LdSPImm16, Nop,
        OrImm8, OrPointedByHL, OrR8, Pop, Push, ResPointedByHL, ResR8, Ret, Reti, Rla, Rlca,
        RlcPointedByHL, RlcR8, RlPointedByHL, RlR8, Rra, Rrca, RrcPointedByHL, RrcR8,
        RrPointedByHL, RrR8, Rst, SbcImm8, SbcPointedByHL, SbcR8, Scf, SetPointedByHL, SetR8,
        SlaPointedByHL, SlaR8, SraPointedByHL, SraR8, SrlPointedByHL, SrlR8, Stop, SubImm8,
        SubPointedByHL, SubR8, SwapPointedByHL, SwapR8, XorImm8, XorPointedByHL, XorR8,
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
