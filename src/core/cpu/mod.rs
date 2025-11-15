mod flags;
mod instructions;
mod registers;

use crate::{
    core::{
        cpu::flags::ZERO_FLAG_MASK,
        memory::{IO_REGISTERS_START, MemoryBus},
    },
    utils::to_u16,
};
use instructions::{InstructionDestination as ID, InstructionTarget as IT, JumpCondition as JC, *};
use registers::{Reg8 as R8, Reg16 as R16};
use std::fmt::{self, Display, Formatter};

/// # CPU
/// Gameboy CPU, with a mix of Intel 8080 and Zilog Z80 features and instruction set.
/// Most of its register are 8-bits ones, that are commonly used as pairs to perform 16-bits operations.
/// The only 16-bits registers are the stack pointer (SP) and the program counter (PC).
pub struct Cpu {
    pub a: u8,
    pub f: u8,
    pub b: u8,
    pub c: u8,
    pub d: u8,
    pub e: u8,
    pub h: u8,
    pub l: u8,

    pub pc: u16,
    pub sp: u16,

    pub cycles: usize,
    pub ime: bool,
    pub bus: MemoryBus,
}

impl Cpu {
    pub fn new(bus: MemoryBus) -> Cpu {
        Cpu {
            a: 0x00,
            f: 0x00,
            b: 0x00,
            c: 0x00,
            d: 0x00,
            e: 0x00,
            h: 0x00,
            l: 0x00,
            pc: 0x0100,
            sp: 0x0000,

            cycles: 0,
            ime: false,
            bus,
        }
    }

    pub fn af(&self) -> u16 { to_u16(self.f, self.a) }
    pub fn bc(&self) -> u16 { to_u16(self.c, self.b) }
    pub fn de(&self) -> u16 { to_u16(self.e, self.d) }
    pub fn hl(&self) -> u16 { to_u16(self.l, self.h) }
    pub fn mut_af(&mut self) -> (&mut u8, &mut u8) { (&mut self.f, &mut self.a) }
    pub fn mut_bc(&mut self) -> (&mut u8, &mut u8) { (&mut self.c, &mut self.b) }
    pub fn mut_de(&mut self) -> (&mut u8, &mut u8) { (&mut self.e, &mut self.d) }
    pub fn mut_hl(&mut self) -> (&mut u8, &mut u8) { (&mut self.l, &mut self.h) }

    pub fn reset(&mut self) {
        self.a = 0x00;
        self.f = 0x00;
        self.b = 0x00;
        self.c = 0x00;
        self.d = 0x00;
        self.e = 0x00;
        self.h = 0x00;
        self.l = 0x00;
        self.pc = 0x0100;
        self.sp = 0x0000;
        self.cycles = 0;
    }

    /// Execute instruction based on the opcode.
    /// Return a result with the effect of the instruction or an instruction error (e.g unused opcode)
    pub fn fetch(&'_ mut self, opcode: u8) -> Result<Box<dyn Instruction<'_> + '_>, InstructionError> {
        let instruction: Box<dyn Instruction> = match opcode {
            0x00 => Nop::new(),
            0x01 => Ld::new(
                ID::Reg16((&mut self.c, &mut self.b), R16::BC),
                IT::Imm16(to_u16(
                    self.bus.clone().borrow()[self.pc + 1],
                    self.bus.clone().borrow()[self.pc + 2],
                )),
            ),
            0x02 => Ld::new(
                ID::PointedByReg16(self.bus.clone(), self.bc(), R16::BC),
                IT::Reg8(self.a, R8::A),
            ),
            0x03 => Inc::new(ID::Reg16(self.mut_bc(), R16::BC)),
            0x04 => Inc::new(ID::Reg8(&mut self.b, R8::B)),
            0x05 => Dec::new(ID::Reg8(&mut self.b, R8::B)),
            0x06 => Ld::new(
                ID::Reg8(&mut self.b, R8::B),
                IT::Imm8(self.bus.clone().borrow()[self.pc + 1]),
            ),
            0x07 => Rlca::new(&mut self.a),
            0x08 => Ld::new(
                ID::PointedByN16(
                    self.bus.clone(),
                    to_u16(
                        self.bus.clone().borrow()[self.pc + 1],
                        self.bus.clone().borrow()[self.pc + 2],
                    ),
                ),
                IT::StackPointer(self.sp),
            ),
            0x09 => {
                let hl = self.hl();
                Add::new(ID::Reg16(self.mut_bc(), R16::BC), IT::Reg16(hl, R16::HL))
            }
            0x40 => return Err(InstructionError::NoOp(opcode, self.pc)),
            0x41 => Ld::new(ID::Reg8(&mut self.b, R8::B), IT::Reg8(self.c, R8::C)),
            0x42 => Ld::new(ID::Reg8(&mut self.b, R8::B), IT::Reg8(self.d, R8::D)),
            0x43 => Ld::new(ID::Reg8(&mut self.b, R8::B), IT::Reg8(self.e, R8::E)),
            0x44 => Ld::new(ID::Reg8(&mut self.b, R8::B), IT::Reg8(self.h, R8::H)),
            0x45 => Ld::new(ID::Reg8(&mut self.b, R8::B), IT::Reg8(self.l, R8::L)),
            0x46 => {
                let addr = self.hl();
                Ld::new(
                    ID::Reg8(&mut self.b, R8::B),
                    IT::PointedByHL(self.bus.clone().borrow()[addr]),
                )
            }
            0x47 => Ld::new(ID::Reg8(&mut self.b, R8::B), IT::Reg8(self.a, R8::A)),
            0x48 => Ld::new(ID::Reg8(&mut self.c, R8::C), IT::Reg8(self.b, R8::B)),
            0x49 => return Err(InstructionError::NoOp(opcode, self.pc)),
            0x4A => Ld::new(ID::Reg8(&mut self.c, R8::C), IT::Reg8(self.d, R8::D)),
            0x4B => Ld::new(ID::Reg8(&mut self.c, R8::C), IT::Reg8(self.e, R8::E)),
            0x4C => Ld::new(ID::Reg8(&mut self.c, R8::C), IT::Reg8(self.h, R8::H)),
            0x4D => Ld::new(ID::Reg8(&mut self.c, R8::C), IT::Reg8(self.l, R8::L)),
            0x4E => {
                let addr = self.hl();
                Ld::new(
                    ID::Reg8(&mut self.c, R8::C),
                    IT::PointedByHL(self.bus.clone().borrow()[addr]),
                )
            }
            0x4F => Ld::new(ID::Reg8(&mut self.c, R8::C), IT::Reg8(self.a, R8::A)),
            0x50 => Ld::new(ID::Reg8(&mut self.d, R8::D), IT::Reg8(self.b, R8::B)),
            0x51 => Ld::new(ID::Reg8(&mut self.d, R8::D), IT::Reg8(self.c, R8::C)),
            0x52 => return Err(InstructionError::NoOp(opcode, self.pc)),
            0x53 => Ld::new(ID::Reg8(&mut self.d, R8::D), IT::Reg8(self.e, R8::E)),
            0x54 => Ld::new(ID::Reg8(&mut self.d, R8::D), IT::Reg8(self.h, R8::H)),
            0x55 => Ld::new(ID::Reg8(&mut self.d, R8::D), IT::Reg8(self.l, R8::L)),
            0x56 => {
                let addr = self.hl();
                Ld::new(
                    ID::Reg8(&mut self.d, R8::D),
                    IT::PointedByHL(self.bus.clone().borrow()[addr]),
                )
            }
            0x57 => Ld::new(ID::Reg8(&mut self.d, R8::D), IT::Reg8(self.a, R8::A)),
            0x58 => Ld::new(ID::Reg8(&mut self.e, R8::E), IT::Reg8(self.b, R8::B)),
            0x59 => Ld::new(ID::Reg8(&mut self.e, R8::E), IT::Reg8(self.c, R8::C)),
            0x5A => Ld::new(ID::Reg8(&mut self.e, R8::E), IT::Reg8(self.d, R8::D)),
            0x5B => return Err(InstructionError::NoOp(opcode, self.pc)),
            0x5C => Ld::new(ID::Reg8(&mut self.e, R8::E), IT::Reg8(self.h, R8::H)),
            0x5D => Ld::new(ID::Reg8(&mut self.e, R8::E), IT::Reg8(self.l, R8::L)),
            0x5E => {
                let addr = self.hl();
                Ld::new(
                    ID::Reg8(&mut self.e, R8::E),
                    IT::PointedByHL(self.bus.clone().borrow()[addr]),
                )
            }
            0x5F => Ld::new(ID::Reg8(&mut self.e, R8::E), IT::Reg8(self.a, R8::A)),
            0x60 => Ld::new(ID::Reg8(&mut self.h, R8::H), IT::Reg8(self.b, R8::B)),
            0x61 => Ld::new(ID::Reg8(&mut self.h, R8::H), IT::Reg8(self.c, R8::C)),
            0x62 => Ld::new(ID::Reg8(&mut self.h, R8::H), IT::Reg8(self.d, R8::D)),
            0x63 => Ld::new(ID::Reg8(&mut self.h, R8::H), IT::Reg8(self.e, R8::E)),
            0x64 => return Err(InstructionError::NoOp(opcode, self.pc)),
            0x65 => Ld::new(ID::Reg8(&mut self.h, R8::H), IT::Reg8(self.l, R8::L)),
            0x66 => {
                let addr = self.hl();
                Ld::new(
                    ID::Reg8(&mut self.h, R8::H),
                    IT::PointedByHL(self.bus.clone().borrow()[addr]),
                )
            }
            0x67 => Ld::new(ID::Reg8(&mut self.h, R8::H), IT::Reg8(self.a, R8::A)),
            0x68 => Ld::new(ID::Reg8(&mut self.l, R8::L), IT::Reg8(self.b, R8::B)),
            0x69 => Ld::new(ID::Reg8(&mut self.l, R8::L), IT::Reg8(self.c, R8::C)),
            0x6A => Ld::new(ID::Reg8(&mut self.l, R8::L), IT::Reg8(self.d, R8::D)),
            0x6B => Ld::new(ID::Reg8(&mut self.l, R8::L), IT::Reg8(self.e, R8::E)),
            0x6C => Ld::new(ID::Reg8(&mut self.l, R8::L), IT::Reg8(self.h, R8::H)),
            0x6D => return Err(InstructionError::NoOp(opcode, self.pc)),
            0x6E => {
                let addr = self.hl();
                Ld::new(
                    ID::Reg8(&mut self.l, R8::L),
                    IT::PointedByHL(self.bus.clone().borrow()[addr]),
                )
            }
            0x6F => Ld::new(ID::Reg8(&mut self.l, R8::L), IT::Reg8(self.a, R8::A)),
            0x70 => Ld::new(
                ID::PointedByHL(self.bus.clone(), self.hl()),
                IT::Reg8(self.b, R8::B),
            ),
            0x71 => Ld::new(
                ID::PointedByHL(self.bus.clone(), self.hl()),
                IT::Reg8(self.c, R8::C),
            ),
            0x72 => Ld::new(
                ID::PointedByHL(self.bus.clone(), self.hl()),
                IT::Reg8(self.d, R8::D),
            ),
            0x73 => Ld::new(
                ID::PointedByHL(self.bus.clone(), self.hl()),
                IT::Reg8(self.e, R8::E),
            ),
            0x74 => Ld::new(
                ID::PointedByHL(self.bus.clone(), self.hl()),
                IT::Reg8(self.h, R8::H),
            ),
            0x75 => Ld::new(
                ID::PointedByHL(self.bus.clone(), self.hl()),
                IT::Reg8(self.l, R8::L),
            ),
            0x76 => return Err(InstructionError::NotImplemented(opcode, self.pc)),
            0x77 => Ld::new(
                ID::PointedByHL(self.bus.clone(), self.hl()),
                IT::Reg8(self.a, R8::A),
            ),
            0x78 => Ld::new(ID::Reg8(&mut self.a, R8::A), IT::Reg8(self.b, R8::B)),
            0x79 => Ld::new(ID::Reg8(&mut self.a, R8::A), IT::Reg8(self.c, R8::C)),
            0x7A => Ld::new(ID::Reg8(&mut self.a, R8::A), IT::Reg8(self.d, R8::D)),
            0x7B => Ld::new(ID::Reg8(&mut self.a, R8::A), IT::Reg8(self.e, R8::E)),
            0x7C => Ld::new(ID::Reg8(&mut self.a, R8::A), IT::Reg8(self.h, R8::H)),
            0x7D => Ld::new(ID::Reg8(&mut self.a, R8::A), IT::Reg8(self.l, R8::L)),
            0x7E => {
                let addr = self.hl();
                Ld::new(
                    ID::Reg8(&mut self.a, R8::A),
                    IT::PointedByHL(self.bus.clone().borrow()[addr]),
                )
            }
            0x7F => return Err(InstructionError::NoOp(opcode, self.pc)),
            0xC2 => {
                let ppcc = self.pc;
                Jp::new(
                    &mut self.pc,
                    IT::JumpToImm16(
                        JC::NotZero(self.f & ZERO_FLAG_MASK == 0),
                        to_u16(
                            self.bus.clone().borrow()[ppcc + 1],
                            self.bus.clone().borrow()[ppcc + 2],
                        ),
                    ),
                )
            }
            0xE0 => Ldh::new(
                ID::PointedByN16(self.bus.clone(), self.pc + 1),
                IT::Reg8(self.a, R8::A),
            ),
            0xE2 => Ldh::new(
                ID::PointedByCPlusFF00(self.bus.clone(), IO_REGISTERS_START + self.c as u16),
                IT::Reg8(self.a, R8::A),
            ),
            0xF0 => Ldh::new(
                ID::Reg8(&mut self.a, R8::A),
                IT::PointedByN16(self.bus.clone().borrow()[self.pc + 1], self.pc + 1),
            ),
            0xF2 => Ldh::new(
                ID::Reg8(&mut self.a, R8::A),
                IT::PointedByCPlusFF00(
                    self.bus.clone().borrow()[IO_REGISTERS_START + self.c as u16],
                    self.c as u16,
                ),
            ),
            _ => return Err(InstructionError::NotImplemented(opcode, self.pc)),
        };

        Ok(instruction)
    }
}

impl Display for Cpu {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        write!(
            f,
            "a: {:02X} f: {:02X} b: {:02X} c: {:02X} d: {:02X} e: {:02X} h: {:02X} l: {:02X} pc: {:04X} sp: {:04X}",
            self.a, self.f, self.b, self.c, self.d, self.e, self.h, self.l, self.pc, self.sp
        )
    }
}
