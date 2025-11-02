mod flags;
mod instructions;
mod registers;

use crate::{
    core::memory::{IO_REGISTERS_START, MemoryBus},
    utils::to_u16,
};
use instructions::{InstructionDestination as ID, InstructionTarget as IT, *};
use registers::{Register8 as R8, Register16 as R16};
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
    bus: MemoryBus,
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
            bus,
        }
    }

    pub fn get_hl(&self) -> u16 { to_u16(self.h, self.l) }

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
    pub fn exec(&mut self, opcode: u8) -> InstructionResult {
        let mut bus = self.bus.borrow_mut();

        let mut instruction: Box<dyn Instruction> = match opcode {
            0x00 => todo!("NOP"),
            0x40 => return Err(InstructionError::NoOp(opcode, self.pc)),
            0x41 => LD::new(ID::Register8(&mut self.b, R8::B), IT::Register8(self.c, R8::C)),
            0x42 => LD::new(ID::Register8(&mut self.b, R8::B), IT::Register8(self.d, R8::D)),
            0x43 => LD::new(ID::Register8(&mut self.b, R8::B), IT::Register8(self.e, R8::E)),
            0x44 => LD::new(ID::Register8(&mut self.b, R8::B), IT::Register8(self.h, R8::H)),
            0x45 => LD::new(ID::Register8(&mut self.b, R8::B), IT::Register8(self.l, R8::L)),
            0x46 => {
                let addr = self.get_hl();
                LD::new(ID::Register8(&mut self.b, R8::B), IT::PointedByHL(bus[addr]))
            }
            0x47 => LD::new(ID::Register8(&mut self.b, R8::B), IT::Register8(self.a, R8::A)),
            0x48 => LD::new(ID::Register8(&mut self.c, R8::C), IT::Register8(self.b, R8::B)),
            0x49 => return Err(InstructionError::NoOp(opcode, self.pc)),
            0x4A => LD::new(ID::Register8(&mut self.c, R8::C), IT::Register8(self.d, R8::D)),
            0x4B => LD::new(ID::Register8(&mut self.c, R8::C), IT::Register8(self.e, R8::E)),
            0x4C => LD::new(ID::Register8(&mut self.c, R8::C), IT::Register8(self.h, R8::H)),
            0x4D => LD::new(ID::Register8(&mut self.c, R8::C), IT::Register8(self.l, R8::L)),
            0x4E => {
                let addr = self.get_hl();
                LD::new(ID::Register8(&mut self.c, R8::C), IT::PointedByHL(bus[addr]))
            }
            0x4F => LD::new(ID::Register8(&mut self.c, R8::C), IT::Register8(self.a, R8::A)),
            0x50 => LD::new(ID::Register8(&mut self.d, R8::D), IT::Register8(self.b, R8::B)),
            0x51 => LD::new(ID::Register8(&mut self.d, R8::D), IT::Register8(self.c, R8::C)),
            0x52 => return Err(InstructionError::NoOp(opcode, self.pc)),
            0x53 => LD::new(ID::Register8(&mut self.d, R8::D), IT::Register8(self.e, R8::E)),
            0x54 => LD::new(ID::Register8(&mut self.d, R8::D), IT::Register8(self.h, R8::H)),
            0x55 => LD::new(ID::Register8(&mut self.d, R8::D), IT::Register8(self.l, R8::L)),
            0x56 => {
                let addr = self.get_hl();
                LD::new(ID::Register8(&mut self.d, R8::D), IT::PointedByHL(bus[addr]))
            }
            0x57 => LD::new(ID::Register8(&mut self.d, R8::D), IT::Register8(self.a, R8::A)),
            0x58 => LD::new(ID::Register8(&mut self.e, R8::E), IT::Register8(self.b, R8::B)),
            0x59 => LD::new(ID::Register8(&mut self.e, R8::E), IT::Register8(self.c, R8::C)),
            0x5A => LD::new(ID::Register8(&mut self.e, R8::E), IT::Register8(self.d, R8::D)),
            0x5B => return Err(InstructionError::NoOp(opcode, self.pc)),
            0x5C => LD::new(ID::Register8(&mut self.e, R8::E), IT::Register8(self.h, R8::H)),
            0x5D => LD::new(ID::Register8(&mut self.e, R8::E), IT::Register8(self.l, R8::L)),
            0x5E => {
                let addr = self.get_hl();
                LD::new(ID::Register8(&mut self.e, R8::E), IT::PointedByHL(bus[addr]))
            }
            0x5F => LD::new(ID::Register8(&mut self.e, R8::E), IT::Register8(self.a, R8::A)),
            0x60 => LD::new(ID::Register8(&mut self.h, R8::H), IT::Register8(self.b, R8::B)),
            0x61 => LD::new(ID::Register8(&mut self.h, R8::H), IT::Register8(self.c, R8::C)),
            0x62 => LD::new(ID::Register8(&mut self.h, R8::H), IT::Register8(self.d, R8::D)),
            0x63 => LD::new(ID::Register8(&mut self.h, R8::H), IT::Register8(self.e, R8::E)),
            0x64 => return Err(InstructionError::NoOp(opcode, self.pc)),
            0x65 => LD::new(ID::Register8(&mut self.h, R8::H), IT::Register8(self.l, R8::L)),
            0x66 => {
                let addr = self.get_hl();
                let value = self.bus.borrow()[addr];
                LD::new(ID::Register8(&mut self.h, R8::H), IT::PointedByHL(value))
            }
            0x67 => LD::new(ID::Register8(&mut self.h, R8::H), IT::Register8(self.a, R8::A)),
            0x68 => LD::new(ID::Register8(&mut self.l, R8::L), IT::Register8(self.b, R8::B)),
            0x69 => LD::new(ID::Register8(&mut self.l, R8::L), IT::Register8(self.c, R8::C)),
            0x6A => LD::new(ID::Register8(&mut self.l, R8::L), IT::Register8(self.d, R8::D)),
            0x6B => LD::new(ID::Register8(&mut self.l, R8::L), IT::Register8(self.e, R8::E)),
            0x6C => LD::new(ID::Register8(&mut self.l, R8::L), IT::Register8(self.h, R8::H)),
            0x6D => return Err(InstructionError::NoOp(opcode, self.pc)),
            0x6E => {
                let addr = self.get_hl();
                let value = self.bus.borrow()[addr];
                LD::new(ID::Register8(&mut self.l, R8::L), IT::PointedByHL(value))
            }
            0x6F => LD::new(ID::Register8(&mut self.l, R8::L), IT::Register8(self.a, R8::A)),
            0x70 => LD::new(ID::PointedByHL(&mut bus[self.get_hl()]), IT::Register8(self.b, R8::B)),
            0x71 => LD::new(ID::PointedByHL(&mut bus[self.get_hl()]), IT::Register8(self.c, R8::C)),
            0x72 => LD::new(ID::PointedByHL(&mut bus[self.get_hl()]), IT::Register8(self.d, R8::D)),
            0x73 => LD::new(ID::PointedByHL(&mut bus[self.get_hl()]), IT::Register8(self.e, R8::E)),
            0x74 => LD::new(ID::PointedByHL(&mut bus[self.get_hl()]), IT::Register8(self.h, R8::H)),
            0x75 => LD::new(ID::PointedByHL(&mut bus[self.get_hl()]), IT::Register8(self.l, R8::L)),
            0x76 => return Err(InstructionError::NotImplemented(opcode, self.pc)),
            0x77 => LD::new(ID::PointedByHL(&mut bus[self.get_hl()]), IT::Register8(self.a, R8::A)),
            0x78 => LD::new(ID::Register8(&mut self.a, R8::A), IT::Register8(self.b, R8::B)),
            0x79 => LD::new(ID::Register8(&mut self.a, R8::A), IT::Register8(self.c, R8::C)),
            0x7A => LD::new(ID::Register8(&mut self.a, R8::A), IT::Register8(self.d, R8::D)),
            0x7B => LD::new(ID::Register8(&mut self.a, R8::A), IT::Register8(self.e, R8::E)),
            0x7C => LD::new(ID::Register8(&mut self.a, R8::A), IT::Register8(self.h, R8::H)),
            0x7D => LD::new(ID::Register8(&mut self.a, R8::A), IT::Register8(self.l, R8::L)),
            0x7E => {
                let addr = self.get_hl();
                let value = self.bus.borrow()[addr];
                LD::new(ID::Register8(&mut self.a, R8::A), IT::PointedByHL(value))
            }
            0x7F => return Err(InstructionError::NoOp(opcode, self.pc)),
            0xE0 => LDH::new(
                ID::PointedByN16(&mut bus[self.pc + 1], self.pc + 1),
                IT::Register8(self.a, R8::A),
            ),
            0xE2 => LDH::new(
                ID::PointedByCPlusFF00(&mut bus[IO_REGISTERS_START + self.c as u16], self.c as u16),
                IT::Register8(self.a, R8::A),
            ),
            0xF0 => LDH::new(
                ID::Register8(&mut self.a, R8::A),
                IT::PointedByN16(bus[self.pc + 1], self.pc + 1),
            ),
            0xF2 => LDH::new(
                ID::Register8(&mut self.a, R8::A),
                IT::PointedByCPlusFF00(bus[IO_REGISTERS_START + self.c as u16], self.c as u16),
            ),
            _ => return Err(InstructionError::NotImplemented(opcode, self.pc)),
        };

        instruction.exec()
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
