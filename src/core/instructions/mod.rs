pub mod load;

pub struct InstructionEffect {
    pub cycles: u8,
    pub len: u16,
    pub flags: Option<u8>,
}

impl InstructionEffect {
    pub fn new(cycles: u8, len: u16, flags: Option<u8>) -> Self {
        InstructionEffect { cycles, len, flags }
    }
}

pub enum InstructionError {
    NoOp(u8, u16),
    UnusedOpcode(u8, u16),
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
        }
    }
}

pub type InstructionResult = Result<InstructionEffect, InstructionError>;
