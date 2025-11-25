mod flags;
mod instructions;
mod registers;

use crate::{
    Memory,
    core::{
        cpu::flags::{CARRY_FLAG_MASK, ZERO_FLAG_MASK},
        memory::IO_REGISTERS_START,
    },
    utils::to_u16,
};
use instructions::{InstructionDestination as ID, InstructionTarget as IT, JumpCondition as JC, *};
use registers::{Reg8 as R8, Reg16 as R16};
use std::fmt::{self, Display, Formatter};

/// # CPU
/// Gameboy CPU, with a mix of Intel 8080 and Zilog Z80 features and instruction set, the Sharp LR35902.
/// Most of its register are 8-bits ones, that are commonly used as pairs to perform 16-bits operations.
/// The only 16-bits registers are the stack pointer (SP) and the program counter (PC).
#[derive(Debug, Default)]
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
}

impl Cpu {
    pub fn new(start_at_boot: bool) -> Cpu {
        Cpu {
            a: if start_at_boot { 0x01 } else { 0x00 },
            f: 0x00,
            b: 0x00,
            c: 0x00,
            d: 0x00,
            e: 0x00,
            h: 0x00,
            l: 0x00,
            pc: if start_at_boot { 0x0000 } else { 0x0100 },
            sp: 0x0000,

            cycles: 0,
            ime: false,
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
        self.ime = false;
        self.cycles = 0;
    }

    /// Execute instruction based on the opcode.
    /// Return a result with the effect of the instruction or an instruction error (e.g unused opcode)
    pub fn fetch(&mut self, bus: &Memory, opcode: u8) -> Result<Box<dyn Instruction>, InstructionError> {
        let instruction: Box<dyn Instruction> = match opcode {
            0x00 => Nop::new(),
            0x01 => Ld::new(ID::Reg16(R16::BC), IT::Imm16(bus.read_word(self.pc + 1))),
            0x02 => Ld::new(ID::PointedByReg16(self.bc(), R16::BC), IT::Reg8(self.a, R8::A)),
            0x03 => Inc::new(ID::Reg16(R16::BC)),
            0x04 => Inc::new(ID::Reg8(R8::B)),
            0x05 => Dec::new(ID::Reg8(R8::B)),
            0x06 => Ld::new(ID::Reg8(R8::B), IT::Imm8(bus[self.pc + 1])),
            0x07 => Rlca::new(),
            0x08 => Ld::new(
                ID::PointedByN16(bus.clone(), bus_ref.read_word(self.pc + 1)),
                IT::StackPointer(self.sp),
            ),
            0x09 => {
                let bc = self.bc();
                Add::new(ID::Reg16(self.mut_hl(), R16::HL), IT::Reg16(bc, R16::BC))
            }
            0x0A => {
                let pointed = bus_ref[self.bc()];
                Ld::new(ID::Reg8(&mut self.a, R8::A), IT::PointedByReg16(pointed, R16::BC))
            }
            0x0B => Dec::new(ID::Reg16(self.mut_bc(), R16::BC)),
            0x0C => Inc::new(ID::Reg8(&mut self.c, R8::C)),
            0x0D => Dec::new(ID::Reg8(&mut self.c, R8::C)),
            0x0E => Ld::new(ID::Reg8(&mut self.c, R8::C), IT::Imm8(bus_ref[self.pc + 1])),
            0x0F => Rrca::new(&mut self.a),
            0x10 => Stop::new(),
            0x11 => Ld::new(
                ID::Reg16((&mut self.e, &mut self.d), R16::DE),
                IT::Imm16(bus_ref.read_word(self.pc + 1)),
            ),
            0x12 => Ld::new(
                ID::PointedByReg16(bus.clone(), self.de(), R16::DE),
                IT::Reg8(self.a, R8::A),
            ),
            0x13 => Inc::new(ID::Reg16(self.mut_de(), R16::DE)),
            0x14 => Inc::new(ID::Reg8(&mut self.d, R8::D)),
            0x15 => Dec::new(ID::Reg8(&mut self.d, R8::D)),
            0x16 => Ld::new(ID::Reg8(&mut self.d, R8::D), IT::Imm8(bus_ref[self.pc + 1])),
            0x17 => Rla::new(self.f & CARRY_FLAG_MASK != 0, &mut self.a),
            0x18 => {
                let ppcc = self.pc;
                Jr::new(&mut self.pc, IT::JumpToImm8(JC::None, bus_ref[ppcc + 1] as i8))
            }
            0x19 => {
                let de = self.de();
                Add::new(ID::Reg16(self.mut_hl(), R16::HL), IT::Reg16(de, R16::DE))
            }
            0x1A => {
                let pointed = bus_ref[self.de()];
                Ld::new(ID::Reg8(&mut self.a, R8::A), IT::PointedByReg16(pointed, R16::DE))
            }
            0x1B => Dec::new(ID::Reg16(self.mut_de(), R16::DE)),
            0x1C => Inc::new(ID::Reg8(&mut self.e, R8::E)),
            0x1D => Dec::new(ID::Reg8(&mut self.e, R8::E)),
            0x1E => Ld::new(ID::Reg8(&mut self.e, R8::E), IT::Imm8(bus_ref[self.pc + 1])),
            0x1F => Rra::new(self.f & CARRY_FLAG_MASK != 0, &mut self.a),
            0x20 => {
                let ppcc = self.pc;
                Jr::new(
                    &mut self.pc,
                    IT::JumpToImm8(JC::NotZero(self.f & ZERO_FLAG_MASK == 0), bus_ref[ppcc + 1] as i8),
                )
            }
            0x21 => Ld::new(
                ID::Reg16((&mut self.l, &mut self.h), R16::HL),
                IT::Imm16(bus_ref.read_word(self.pc + 1)),
            ),
            0x22 => {
                let aa = self.a;
                Ld::new(ID::PointedByHLI(bus.clone(), self.mut_hl()), IT::Reg8(aa, R8::A))
            }
            0x23 => Inc::new(ID::Reg16(self.mut_hl(), R16::HL)),
            0x24 => Inc::new(ID::Reg8(&mut self.h, R8::H)),
            0x25 => Dec::new(ID::Reg8(&mut self.h, R8::H)),
            0x26 => Ld::new(ID::Reg8(&mut self.h, R8::H), IT::Imm8(bus_ref[self.pc + 1])),
            0x27 => Daa::new(&mut self.a, &mut self.f),
            0x28 => {
                let pc = self.pc;
                Jr::new(
                    &mut self.pc,
                    IT::JumpToImm8(JC::Zero(self.f & ZERO_FLAG_MASK != 0), bus_ref[pc + 1] as i8),
                )
            }
            0x29 => {
                let hl = self.hl();
                Add::new(ID::Reg16(self.mut_hl(), R16::HL), IT::Reg16(hl, R16::HL))
            }
            0x2A => {
                let pointed = bus_ref[self.hl()];
                Ld::new(
                    ID::Reg8(&mut self.a, R8::A),
                    IT::PointedByHLI(pointed, (&mut self.l, &mut self.h)),
                )
            }
            0x2b => Dec::new(ID::Reg16(self.mut_hl(), R16::HL)),
            0x2C => Inc::new(ID::Reg8(&mut self.l, R8::L)),
            0x2D => Dec::new(ID::Reg8(&mut self.l, R8::L)),
            0x2E => Ld::new(ID::Reg8(&mut self.l, R8::L), IT::Imm8(bus_ref[self.pc + 1])),
            0x2F => Cpl::new(&mut self.a),
            0x30 => {
                let ppcc = self.pc;
                Jr::new(
                    &mut self.pc,
                    IT::JumpToImm8(
                        JC::NotCarry(self.f & CARRY_FLAG_MASK == 0),
                        bus_ref[ppcc + 1] as i8,
                    ),
                )
            }
            0x31 => Ld::new(
                ID::StackPointer(&mut self.sp),
                IT::Imm16(bus_ref.read_word(self.pc + 1)),
            ),
            0x32 => {
                let aa = self.a;
                Ld::new(ID::PointedByHLD(bus.clone(), self.mut_hl()), IT::Reg8(aa, R8::A))
            }
            0x33 => Inc::new(ID::StackPointer(&mut self.sp)),
            0x34 => Inc::new(ID::PointedByHL(bus.clone(), self.hl())),
            0x35 => Dec::new(ID::PointedByHL(bus.clone(), self.hl())),
            0x36 => Ld::new(
                ID::PointedByHL(bus.clone(), self.hl()),
                IT::Imm8(bus_ref[self.pc + 1]),
            ),
            0x37 => Scf::new(),
            0x38 => {
                let ppcc = self.pc;
                Jr::new(
                    &mut self.pc,
                    IT::JumpToImm8(JC::Carry(self.f & CARRY_FLAG_MASK != 0), bus_ref[ppcc + 1] as i8),
                )
            }
            0x39 => {
                let sp = self.sp;
                Add::new(ID::Reg16(self.mut_hl(), R16::HL), IT::StackPointer(sp))
            }
            0x3A => {
                let pointed = bus_ref[self.hl()];
                Ld::new(
                    ID::Reg8(&mut self.a, R8::A),
                    IT::PointedByHLD(pointed, (&mut self.l, &mut self.h)),
                )
            }
            0x3B => Dec::new(ID::StackPointer(&mut self.sp)),
            0x3C => Inc::new(ID::Reg8(&mut self.a, R8::A)),
            0x3D => Dec::new(ID::Reg8(&mut self.a, R8::A)),
            0x3E => Ld::new(ID::Reg8(&mut self.a, R8::A), IT::Imm8(bus_ref[self.pc + 1])),
            0x3F => Ccf::new(self.f & CARRY_FLAG_MASK != 0),
            0x40 => {
                let b = self.b;
                Ld::new(ID::Reg8(&mut self.b, R8::B), IT::Reg8(b, R8::B))
            }
            0x41 => Ld::new(ID::Reg8(&mut self.b, R8::B), IT::Reg8(self.c, R8::C)),
            0x42 => Ld::new(ID::Reg8(&mut self.b, R8::B), IT::Reg8(self.d, R8::D)),
            0x43 => Ld::new(ID::Reg8(&mut self.b, R8::B), IT::Reg8(self.e, R8::E)),
            0x44 => Ld::new(ID::Reg8(&mut self.b, R8::B), IT::Reg8(self.h, R8::H)),
            0x45 => Ld::new(ID::Reg8(&mut self.b, R8::B), IT::Reg8(self.l, R8::L)),
            0x46 => {
                let addr = self.hl();
                Ld::new(ID::Reg8(&mut self.b, R8::B), IT::PointedByHL(bus_ref[addr]))
            }
            0x47 => Ld::new(ID::Reg8(&mut self.b, R8::B), IT::Reg8(self.a, R8::A)),
            0x48 => Ld::new(ID::Reg8(&mut self.c, R8::C), IT::Reg8(self.b, R8::B)),
            0x49 => {
                let c = self.c;
                Ld::new(ID::Reg8(&mut self.c, R8::C), IT::Reg8(c, R8::C))
            }
            0x4A => Ld::new(ID::Reg8(&mut self.c, R8::C), IT::Reg8(self.d, R8::D)),
            0x4B => Ld::new(ID::Reg8(&mut self.c, R8::C), IT::Reg8(self.e, R8::E)),
            0x4C => Ld::new(ID::Reg8(&mut self.c, R8::C), IT::Reg8(self.h, R8::H)),
            0x4D => Ld::new(ID::Reg8(&mut self.c, R8::C), IT::Reg8(self.l, R8::L)),
            0x4E => {
                let addr = self.hl();
                Ld::new(ID::Reg8(&mut self.c, R8::C), IT::PointedByHL(bus_ref[addr]))
            }
            0x4F => Ld::new(ID::Reg8(&mut self.c, R8::C), IT::Reg8(self.a, R8::A)),
            0x50 => Ld::new(ID::Reg8(&mut self.d, R8::D), IT::Reg8(self.b, R8::B)),
            0x51 => Ld::new(ID::Reg8(&mut self.d, R8::D), IT::Reg8(self.c, R8::C)),
            0x52 => {
                let d = self.d;
                Ld::new(ID::Reg8(&mut self.d, R8::D), IT::Reg8(d, R8::D))
            }
            0x53 => Ld::new(ID::Reg8(&mut self.d, R8::D), IT::Reg8(self.e, R8::E)),
            0x54 => Ld::new(ID::Reg8(&mut self.d, R8::D), IT::Reg8(self.h, R8::H)),
            0x55 => Ld::new(ID::Reg8(&mut self.d, R8::D), IT::Reg8(self.l, R8::L)),
            0x56 => {
                let addr = self.hl();
                Ld::new(ID::Reg8(&mut self.d, R8::D), IT::PointedByHL(bus_ref[addr]))
            }
            0x57 => Ld::new(ID::Reg8(&mut self.d, R8::D), IT::Reg8(self.a, R8::A)),
            0x58 => Ld::new(ID::Reg8(&mut self.e, R8::E), IT::Reg8(self.b, R8::B)),
            0x59 => Ld::new(ID::Reg8(&mut self.e, R8::E), IT::Reg8(self.c, R8::C)),
            0x5A => Ld::new(ID::Reg8(&mut self.e, R8::E), IT::Reg8(self.d, R8::D)),
            0x5B => {
                let e = self.e;
                Ld::new(ID::Reg8(&mut self.e, R8::E), IT::Reg8(e, R8::E))
            }
            0x5C => Ld::new(ID::Reg8(&mut self.e, R8::E), IT::Reg8(self.h, R8::H)),
            0x5D => Ld::new(ID::Reg8(&mut self.e, R8::E), IT::Reg8(self.l, R8::L)),
            0x5E => {
                let addr = self.hl();
                Ld::new(ID::Reg8(&mut self.e, R8::E), IT::PointedByHL(bus_ref[addr]))
            }
            0x5F => Ld::new(ID::Reg8(&mut self.e, R8::E), IT::Reg8(self.a, R8::A)),
            0x60 => Ld::new(ID::Reg8(&mut self.h, R8::H), IT::Reg8(self.b, R8::B)),
            0x61 => Ld::new(ID::Reg8(&mut self.h, R8::H), IT::Reg8(self.c, R8::C)),
            0x62 => Ld::new(ID::Reg8(&mut self.h, R8::H), IT::Reg8(self.d, R8::D)),
            0x63 => Ld::new(ID::Reg8(&mut self.h, R8::H), IT::Reg8(self.e, R8::E)),
            0x64 => {
                let h = self.h;
                Ld::new(ID::Reg8(&mut self.h, R8::H), IT::Reg8(h, R8::H))
            }
            0x65 => Ld::new(ID::Reg8(&mut self.h, R8::H), IT::Reg8(self.l, R8::L)),
            0x66 => {
                let addr = self.hl();
                Ld::new(ID::Reg8(&mut self.h, R8::H), IT::PointedByHL(bus_ref[addr]))
            }
            0x67 => Ld::new(ID::Reg8(&mut self.h, R8::H), IT::Reg8(self.a, R8::A)),
            0x68 => Ld::new(ID::Reg8(&mut self.l, R8::L), IT::Reg8(self.b, R8::B)),
            0x69 => Ld::new(ID::Reg8(&mut self.l, R8::L), IT::Reg8(self.c, R8::C)),
            0x6A => Ld::new(ID::Reg8(&mut self.l, R8::L), IT::Reg8(self.d, R8::D)),
            0x6B => Ld::new(ID::Reg8(&mut self.l, R8::L), IT::Reg8(self.e, R8::E)),
            0x6C => Ld::new(ID::Reg8(&mut self.l, R8::L), IT::Reg8(self.h, R8::H)),
            0x6D => {
                let l = self.l;
                Ld::new(ID::Reg8(&mut self.l, R8::L), IT::Reg8(l, R8::L))
            }
            0x6E => {
                let addr = self.hl();
                Ld::new(ID::Reg8(&mut self.l, R8::L), IT::PointedByHL(bus_ref[addr]))
            }
            0x6F => Ld::new(ID::Reg8(&mut self.l, R8::L), IT::Reg8(self.a, R8::A)),
            0x70 => Ld::new(ID::PointedByHL(bus.clone(), self.hl()), IT::Reg8(self.b, R8::B)),
            0x71 => Ld::new(ID::PointedByHL(bus.clone(), self.hl()), IT::Reg8(self.c, R8::C)),
            0x72 => Ld::new(ID::PointedByHL(bus.clone(), self.hl()), IT::Reg8(self.d, R8::D)),
            0x73 => Ld::new(ID::PointedByHL(bus.clone(), self.hl()), IT::Reg8(self.e, R8::E)),
            0x74 => Ld::new(ID::PointedByHL(bus.clone(), self.hl()), IT::Reg8(self.h, R8::H)),
            0x75 => Ld::new(ID::PointedByHL(bus.clone(), self.hl()), IT::Reg8(self.l, R8::L)),
            0x76 => Halt::new(),
            0x77 => Ld::new(ID::PointedByHL(bus.clone(), self.hl()), IT::Reg8(self.a, R8::A)),
            0x78 => Ld::new(ID::Reg8(&mut self.a, R8::A), IT::Reg8(self.b, R8::B)),
            0x79 => Ld::new(ID::Reg8(&mut self.a, R8::A), IT::Reg8(self.c, R8::C)),
            0x7A => Ld::new(ID::Reg8(&mut self.a, R8::A), IT::Reg8(self.d, R8::D)),
            0x7B => Ld::new(ID::Reg8(&mut self.a, R8::A), IT::Reg8(self.e, R8::E)),
            0x7C => Ld::new(ID::Reg8(&mut self.a, R8::A), IT::Reg8(self.h, R8::H)),
            0x7D => Ld::new(ID::Reg8(&mut self.a, R8::A), IT::Reg8(self.l, R8::L)),
            0x7E => {
                let addr = self.hl();
                Ld::new(ID::Reg8(&mut self.a, R8::A), IT::PointedByHL(bus_ref[addr]))
            }
            0x7F => {
                let a = self.a;
                Ld::new(ID::Reg8(&mut self.a, R8::A), IT::Reg8(a, R8::A))
            }
            0x80 => Add::new(ID::Reg8(&mut self.a, R8::A), IT::Reg8(self.b, R8::B)),
            0x81 => Add::new(ID::Reg8(&mut self.a, R8::A), IT::Reg8(self.c, R8::C)),
            0x82 => Add::new(ID::Reg8(&mut self.a, R8::A), IT::Reg8(self.d, R8::D)),
            0x83 => Add::new(ID::Reg8(&mut self.a, R8::A), IT::Reg8(self.e, R8::E)),
            0x84 => Add::new(ID::Reg8(&mut self.a, R8::A), IT::Reg8(self.h, R8::H)),
            0x85 => Add::new(ID::Reg8(&mut self.a, R8::A), IT::Reg8(self.l, R8::L)),
            0x86 => {
                let pointed = bus_ref[self.hl()];
                Add::new(ID::Reg8(&mut self.a, R8::A), IT::PointedByHL(pointed))
            }
            0x87 => {
                let a = self.a;
                Add::new(ID::Reg8(&mut self.a, R8::A), IT::Reg8(a, R8::A))
            }
            0x88 => Adc::new(&mut self.a, self.f, IT::Reg8(self.b, R8::B)),
            0x89 => Adc::new(&mut self.a, self.f, IT::Reg8(self.c, R8::C)),
            0x8A => Adc::new(&mut self.a, self.f, IT::Reg8(self.d, R8::D)),
            0x8B => Adc::new(&mut self.a, self.f, IT::Reg8(self.e, R8::E)),
            0x8C => Adc::new(&mut self.a, self.f, IT::Reg8(self.h, R8::H)),
            0x8D => Adc::new(&mut self.a, self.f, IT::Reg8(self.l, R8::L)),
            0x8E => {
                let pointed = bus_ref[self.hl()];
                Adc::new(&mut self.a, self.f, IT::PointedByHL(pointed))
            }
            0x8F => {
                let a = self.a;
                Adc::new(&mut self.a, self.f, IT::Reg8(a, R8::A))
            }
            0x90 => Sub::new(&mut self.a, IT::Reg8(self.b, R8::B)),
            0x91 => Sub::new(&mut self.a, IT::Reg8(self.c, R8::C)),
            0x92 => Sub::new(&mut self.a, IT::Reg8(self.d, R8::D)),
            0x93 => Sub::new(&mut self.a, IT::Reg8(self.e, R8::E)),
            0x94 => Sub::new(&mut self.a, IT::Reg8(self.h, R8::H)),
            0x95 => Sub::new(&mut self.a, IT::Reg8(self.l, R8::L)),
            0x96 => {
                let pointed = bus_ref[self.hl()];
                Sub::new(&mut self.a, IT::PointedByHL(pointed))
            }
            0x97 => {
                let a = self.a;
                Sub::new(&mut self.a, IT::Reg8(a, R8::A))
            }
            0x98 => Sbc::new(&mut self.a, self.f, IT::Reg8(self.b, R8::B)),
            0x99 => Sbc::new(&mut self.a, self.f, IT::Reg8(self.c, R8::C)),
            0x9A => Sbc::new(&mut self.a, self.f, IT::Reg8(self.d, R8::D)),
            0x9B => Sbc::new(&mut self.a, self.f, IT::Reg8(self.e, R8::E)),
            0x9C => Sbc::new(&mut self.a, self.f, IT::Reg8(self.h, R8::H)),
            0x9D => Sbc::new(&mut self.a, self.f, IT::Reg8(self.l, R8::L)),
            0x9E => {
                let pointed = bus_ref[self.hl()];
                Sbc::new(&mut self.a, self.f, IT::PointedByHL(pointed))
            }
            0x9F => {
                let a = self.a;
                Sbc::new(&mut self.a, self.f, IT::Reg8(a, R8::A))
            }
            0xA0 => And::new(&mut self.a, IT::Reg8(self.b, R8::B)),
            0xA1 => And::new(&mut self.a, IT::Reg8(self.c, R8::C)),
            0xA2 => And::new(&mut self.a, IT::Reg8(self.d, R8::D)),
            0xA3 => And::new(&mut self.a, IT::Reg8(self.e, R8::E)),
            0xA4 => And::new(&mut self.a, IT::Reg8(self.h, R8::H)),
            0xA5 => And::new(&mut self.a, IT::Reg8(self.l, R8::L)),
            0xA6 => {
                let pointed = bus_ref[self.hl()];
                And::new(&mut self.a, IT::PointedByHL(pointed))
            }
            0xA7 => {
                let a = self.a;
                And::new(&mut self.a, IT::Reg8(a, R8::A))
            }
            0xA8 => Xor::new(&mut self.a, IT::Reg8(self.b, R8::B)),
            0xA9 => Xor::new(&mut self.a, IT::Reg8(self.c, R8::C)),
            0xAA => Xor::new(&mut self.a, IT::Reg8(self.d, R8::D)),
            0xAB => Xor::new(&mut self.a, IT::Reg8(self.e, R8::E)),
            0xAC => Xor::new(&mut self.a, IT::Reg8(self.h, R8::H)),
            0xAD => Xor::new(&mut self.a, IT::Reg8(self.l, R8::L)),
            0xAE => {
                let pointed = bus_ref[self.hl()];
                Xor::new(&mut self.a, IT::PointedByHL(pointed))
            }
            0xAF => {
                let a = self.a;
                Xor::new(&mut self.a, IT::Reg8(a, R8::A))
            }
            0xB0 => Or::new(&mut self.a, IT::Reg8(self.b, R8::B)),
            0xB1 => Or::new(&mut self.a, IT::Reg8(self.c, R8::C)),
            0xB2 => Or::new(&mut self.a, IT::Reg8(self.d, R8::D)),
            0xB3 => Or::new(&mut self.a, IT::Reg8(self.e, R8::E)),
            0xB4 => Or::new(&mut self.a, IT::Reg8(self.h, R8::H)),
            0xB5 => Or::new(&mut self.a, IT::Reg8(self.l, R8::L)),
            0xB6 => {
                let pointed = bus_ref[self.hl()];
                Or::new(&mut self.a, IT::PointedByHL(pointed))
            }
            0xB7 => {
                let a = self.a;
                Or::new(&mut self.a, IT::Reg8(a, R8::A))
            }
            0xB8 => Cp::new(self.a, IT::Reg8(self.b, R8::B)),
            0xB9 => Cp::new(self.a, IT::Reg8(self.c, R8::C)),
            0xBA => Cp::new(self.a, IT::Reg8(self.d, R8::D)),
            0xBB => Cp::new(self.a, IT::Reg8(self.e, R8::E)),
            0xBC => Cp::new(self.a, IT::Reg8(self.h, R8::H)),
            0xBD => Cp::new(self.a, IT::Reg8(self.l, R8::L)),
            0xBE => Cp::new(self.a, IT::PointedByHL(bus_ref[self.hl()])),
            0xBF => Cp::new(self.a, IT::Reg8(self.a, R8::A)),
            0xC0 => Ret::new(
                &mut self.pc,
                &mut self.sp,
                bus.clone(),
                JC::NotZero(self.f & ZERO_FLAG_MASK == 0),
            ),
            0xC1 => Pop::new(
                ID::Reg16((&mut self.c, &mut self.b), R16::BC),
                bus_ref.read_word(self.pc),
                &mut self.sp,
            ),
            0xC2 => {
                let ppcc = self.pc;
                Jp::new(
                    &mut self.pc,
                    IT::JumpToImm16(
                        JC::NotZero(self.f & ZERO_FLAG_MASK == 0),
                        bus_ref.read_word(ppcc + 1),
                    ),
                )
            }
            0xC3 => {
                let ppcc = self.pc;
                Jp::new(
                    &mut self.pc,
                    IT::JumpToImm16(JC::None, bus_ref.read_word(ppcc + 1)),
                )
            }
            0xC4 => {
                let pc = self.pc;
                Call::new(
                    &mut self.pc,
                    &mut self.sp,
                    bus.clone(),
                    IT::JumpToImm16(
                        JC::NotZero(self.f & ZERO_FLAG_MASK == 0),
                        bus_ref.read_word(pc + 1),
                    ),
                )
            }
            0xC5 => {
                let bc = self.bc();
                Push::new(&mut self.sp, bus.clone(), IT::Reg16(bc, R16::BC))
            }
            0xC6 => Add::new(ID::Reg8(&mut self.a, R8::A), IT::Imm8(bus_ref[self.pc + 1])),
            0xC7 => Rst::new(&mut self.pc, &mut self.sp, bus.clone(), 0x00),
            0xC8 => Ret::new(
                &mut self.pc,
                &mut self.sp,
                bus.clone(),
                JC::Zero(self.f & ZERO_FLAG_MASK != 0),
            ),
            0xC9 => Ret::new(&mut self.pc, &mut self.sp, bus.clone(), JC::None),
            0xCA => {
                let ppcc = self.pc;
                Jp::new(
                    &mut self.pc,
                    IT::JumpToImm16(
                        JC::Zero(self.f & ZERO_FLAG_MASK != 0),
                        bus_ref.read_word(ppcc + 1),
                    ),
                )
            }
            0xCB => {
                let cb_opcode = bus_ref[self.pc + 1];
                match self.fetch_cb(bus.clone(), cb_opcode) {
                    Ok(instruction) => instruction,
                    Err(e) => return Err(e),
                }
            }
            0xCC => {
                let pc = self.pc;
                Call::new(
                    &mut self.pc,
                    &mut self.sp,
                    bus.clone(),
                    IT::JumpToImm16(JC::Zero(self.f & ZERO_FLAG_MASK != 0), bus_ref.read_word(pc + 1)),
                )
            }
            0xCD => {
                let pc = self.pc;
                Call::new(
                    &mut self.pc,
                    &mut self.sp,
                    bus.clone(),
                    IT::JumpToImm16(JC::None, bus_ref.read_word(pc + 1)),
                )
            }
            0xCE => Adc::new(&mut self.a, self.f, IT::Imm8(bus_ref[self.pc + 1])),
            0xCF => Rst::new(&mut self.pc, &mut self.sp, bus.clone(), 0x08),
            0xD0 => Ret::new(
                &mut self.pc,
                &mut self.sp,
                bus.clone(),
                JC::NotCarry(self.f & CARRY_FLAG_MASK == 0),
            ),
            0xD1 => Pop::new(
                ID::Reg16((&mut self.e, &mut self.d), R16::DE),
                bus_ref.read_word(self.pc),
                &mut self.sp,
            ),
            0xD2 => {
                let ppcc = self.pc;
                Jp::new(
                    &mut self.pc,
                    IT::JumpToImm16(
                        JC::NotCarry(self.f & CARRY_FLAG_MASK == 0),
                        bus_ref.read_word(ppcc + 1),
                    ),
                )
            }
            0xD3 => return Err(InstructionError::UnusedOpcode(opcode, self.pc)),
            0xD4 => {
                let pc = self.pc;
                Call::new(
                    &mut self.pc,
                    &mut self.sp,
                    bus.clone(),
                    IT::JumpToImm16(
                        JC::NotCarry(self.f & CARRY_FLAG_MASK == 0),
                        bus_ref.read_word(pc + 1),
                    ),
                )
            }
            0xD5 => {
                let de = self.de();
                Push::new(&mut self.sp, bus.clone(), IT::Reg16(de, R16::DE))
            }
            0xD6 => Sub::new(&mut self.a, IT::Imm8(bus_ref[self.pc + 1])),
            0xD7 => Rst::new(&mut self.pc, &mut self.sp, bus.clone(), 0x10),
            0xD8 => Ret::new(
                &mut self.pc,
                &mut self.sp,
                bus.clone(),
                JC::Carry(self.f & CARRY_FLAG_MASK != 0),
            ),
            0xD9 => Reti::new(&mut self.pc, &mut self.sp, &mut self.ime, bus.clone()),
            0xDA => {
                let ppcc = self.pc;
                Jp::new(
                    &mut self.pc,
                    IT::JumpToImm16(
                        JC::Carry(self.f & CARRY_FLAG_MASK != 0),
                        bus_ref.read_word(ppcc + 1),
                    ),
                )
            }
            0xDB => return Err(InstructionError::UnusedOpcode(opcode, self.pc)),
            0xDC => {
                let pc = self.pc;
                Call::new(
                    &mut self.pc,
                    &mut self.sp,
                    bus.clone(),
                    IT::JumpToImm16(
                        JC::Carry(self.f & CARRY_FLAG_MASK != 0),
                        bus_ref.read_word(pc + 1),
                    ),
                )
            }
            0xDD => return Err(InstructionError::UnusedOpcode(opcode, self.pc)),
            0xDE => Sbc::new(&mut self.a, self.f, IT::Imm8(bus_ref[self.pc + 1])),
            0xDF => Rst::new(&mut self.pc, &mut self.sp, bus.clone(), 0x18),
            0xE0 => Ldh::new(
                ID::PointedByN16(bus.clone(), self.pc + 1),
                IT::Reg8(self.a, R8::A),
            ),
            0xE1 => Pop::new(
                ID::Reg16((&mut self.l, &mut self.h), R16::HL),
                bus_ref.read_word(self.pc),
                &mut self.sp,
            ),
            0xE2 => Ldh::new(
                ID::PointedByCPlusFF00(bus.clone(), IO_REGISTERS_START + self.c as u16),
                IT::Reg8(self.a, R8::A),
            ),
            0xE3 => return Err(InstructionError::UnusedOpcode(opcode, self.pc)),
            0xE4 => return Err(InstructionError::UnusedOpcode(opcode, self.pc)),
            0xE5 => {
                let hl = self.hl();
                Push::new(&mut self.sp, bus.clone(), IT::Reg16(hl, R16::HL))
            }
            0xE6 => And::new(&mut self.a, IT::Imm8(bus_ref[self.pc + 1])),
            0xE7 => Rst::new(&mut self.pc, &mut self.sp, bus.clone(), 0x20),
            0xE8 => Add::new(
                ID::StackPointer(&mut self.sp),
                IT::SignedImm(bus_ref[self.pc + 1] as i8),
            ),
            0xE9 => {
                let hl = self.hl();
                Jp::new(&mut self.pc, IT::JumpToHL(hl))
            }
            0xEA => Ld::new(
                ID::PointedByN16(bus.clone(), bus_ref.read_word(self.pc + 1)),
                IT::Reg8(self.a, R8::A),
            ),
            0xEB => return Err(InstructionError::UnusedOpcode(opcode, self.pc)),
            0xEC => return Err(InstructionError::UnusedOpcode(opcode, self.pc)),
            0xED => return Err(InstructionError::UnusedOpcode(opcode, self.pc)),
            0xEE => Xor::new(&mut self.a, IT::Imm8(bus_ref[self.pc + 1])),
            0xEF => Rst::new(&mut self.pc, &mut self.sp, bus.clone(), 0x28),
            0xF0 => Ldh::new(
                ID::Reg8(&mut self.a, R8::A),
                IT::PointedByN16(bus_ref[self.pc + 1], self.pc + 1),
            ),
            0xF1 => Pop::new(
                ID::Reg16((&mut self.f, &mut self.a), R16::AF),
                bus_ref.read_word(self.pc),
                &mut self.sp,
            ),
            0xF2 => Ldh::new(
                ID::Reg8(&mut self.a, R8::A),
                IT::PointedByCPlusFF00(bus_ref[IO_REGISTERS_START + self.c as u16], self.c as u16),
            ),
            0xF3 => Di::new(&mut self.ime),
            0xF4 => return Err(InstructionError::UnusedOpcode(opcode, self.pc)),
            0xF5 => {
                let af = self.af();
                Push::new(&mut self.sp, bus.clone(), IT::Reg16(af, R16::AF))
            }
            0xF6 => Or::new(&mut self.a, IT::Imm8(bus_ref[self.pc + 1])),
            0xF7 => Rst::new(&mut self.pc, &mut self.sp, bus.clone(), 0x30),
            0xF8 => Ldh::new(
                ID::Reg16((&mut self.h, &mut self.l), R16::HL),
                IT::StackPointerPlusE8(self.sp, bus_ref[self.pc + 1] as i8),
            ),
            0xF9 => {
                let hl = self.hl();
                Ld::new(ID::StackPointer(&mut self.sp), IT::Reg16(hl, R16::HL))
            }
            0xFA => Ld::new(
                ID::Reg8(&mut self.a, R8::A),
                IT::PointedByN16(bus_ref[self.pc + 1], self.sp),
            ),
            0xFB => Ei::new(&mut self.ime),
            0xFC => return Err(InstructionError::UnusedOpcode(opcode, self.pc)),
            0xFD => return Err(InstructionError::UnusedOpcode(opcode, self.pc)),
            0xFE => Cp::new(self.a, IT::Imm8(bus_ref[self.pc + 1])),
            0xFF => Rst::new(&mut self.pc, &mut self.sp, bus.clone(), 0x38),
        };

        Ok(instruction)
    }

    fn fetch_cb(
        &'_ mut self,
        bus: MemoryBus,
        cb_opcode: u8,
    ) -> Result<Box<dyn Instruction<'_> + '_>, InstructionError> {
        // used bit in res, set and bit instructions
        let bit = (cb_opcode & 0x38) >> 3;

        let instruction: Box<dyn Instruction> = match cb_opcode {
            0x00 => Rlc::new(ID::Reg8(&mut self.b, R8::B)),
            0x01 => Rlc::new(ID::Reg8(&mut self.c, R8::C)),
            0x02 => Rlc::new(ID::Reg8(&mut self.d, R8::D)),
            0x03 => Rlc::new(ID::Reg8(&mut self.e, R8::E)),
            0x04 => Rlc::new(ID::Reg8(&mut self.h, R8::H)),
            0x05 => Rlc::new(ID::Reg8(&mut self.l, R8::L)),
            0x06 => Rlc::new(ID::PointedByHL(bus.clone(), self.hl())),
            0x07 => Rlc::new(ID::Reg8(&mut self.a, R8::A)),
            0x08 => Rrc::new(ID::Reg8(&mut self.b, R8::B)),
            0x09 => Rrc::new(ID::Reg8(&mut self.c, R8::C)),
            0x0A => Rrc::new(ID::Reg8(&mut self.d, R8::D)),
            0x0B => Rrc::new(ID::Reg8(&mut self.e, R8::E)),
            0x0C => Rrc::new(ID::Reg8(&mut self.h, R8::H)),
            0x0D => Rrc::new(ID::Reg8(&mut self.l, R8::L)),
            0x0E => Rrc::new(ID::PointedByHL(bus.clone(), self.hl())),
            0x0F => Rrc::new(ID::Reg8(&mut self.a, R8::A)),
            0x10 => Rl::new(self.f, ID::Reg8(&mut self.b, R8::B)),
            0x11 => Rl::new(self.f, ID::Reg8(&mut self.c, R8::C)),
            0x12 => Rl::new(self.f, ID::Reg8(&mut self.d, R8::D)),
            0x13 => Rl::new(self.f, ID::Reg8(&mut self.e, R8::E)),
            0x14 => Rl::new(self.f, ID::Reg8(&mut self.h, R8::H)),
            0x15 => Rl::new(self.f, ID::Reg8(&mut self.l, R8::L)),
            0x16 => Rl::new(self.f, ID::PointedByHL(bus.clone(), self.hl())),
            0x17 => Rl::new(self.f, ID::Reg8(&mut self.a, R8::A)),
            0x18 => Rr::new(self.f, ID::Reg8(&mut self.b, R8::B)),
            0x19 => Rr::new(self.f, ID::Reg8(&mut self.c, R8::C)),
            0x1A => Rr::new(self.f, ID::Reg8(&mut self.d, R8::D)),
            0x1B => Rr::new(self.f, ID::Reg8(&mut self.e, R8::E)),
            0x1C => Rr::new(self.f, ID::Reg8(&mut self.h, R8::H)),
            0x1D => Rr::new(self.f, ID::Reg8(&mut self.l, R8::L)),
            0x1E => Rr::new(self.f, ID::PointedByHL(bus.clone(), self.hl())),
            0x1F => Rr::new(self.f, ID::Reg8(&mut self.a, R8::A)),
            0x20 => Sla::new(ID::Reg8(&mut self.b, R8::B)),
            0x21 => Sla::new(ID::Reg8(&mut self.c, R8::C)),
            0x22 => Sla::new(ID::Reg8(&mut self.d, R8::D)),
            0x23 => Sla::new(ID::Reg8(&mut self.e, R8::E)),
            0x24 => Sla::new(ID::Reg8(&mut self.h, R8::H)),
            0x25 => Sla::new(ID::Reg8(&mut self.l, R8::L)),
            0x26 => Sla::new(ID::PointedByHL(bus.clone(), self.hl())),
            0x27 => Sla::new(ID::Reg8(&mut self.a, R8::A)),
            0x28 => Sra::new(ID::Reg8(&mut self.b, R8::B)),
            0x29 => Sra::new(ID::Reg8(&mut self.c, R8::C)),
            0x2A => Sra::new(ID::Reg8(&mut self.d, R8::D)),
            0x2B => Sra::new(ID::Reg8(&mut self.e, R8::E)),
            0x2C => Sra::new(ID::Reg8(&mut self.h, R8::H)),
            0x2D => Sra::new(ID::Reg8(&mut self.l, R8::L)),
            0x2E => Sra::new(ID::PointedByHL(bus.clone(), self.hl())),
            0x2F => Sra::new(ID::Reg8(&mut self.a, R8::A)),
            0x30 => Swap::new(ID::Reg8(&mut self.b, R8::B)),
            0x31 => Swap::new(ID::Reg8(&mut self.c, R8::C)),
            0x32 => Swap::new(ID::Reg8(&mut self.d, R8::D)),
            0x33 => Swap::new(ID::Reg8(&mut self.e, R8::E)),
            0x34 => Swap::new(ID::Reg8(&mut self.h, R8::H)),
            0x35 => Swap::new(ID::Reg8(&mut self.l, R8::L)),
            0x36 => Swap::new(ID::PointedByHL(bus.clone(), self.hl())),
            0x37 => Swap::new(ID::Reg8(&mut self.a, R8::A)),
            0x38 => Srl::new(ID::Reg8(&mut self.b, R8::B)),
            0x39 => Srl::new(ID::Reg8(&mut self.c, R8::C)),
            0x3A => Srl::new(ID::Reg8(&mut self.d, R8::D)),
            0x3B => Srl::new(ID::Reg8(&mut self.e, R8::E)),
            0x3C => Srl::new(ID::Reg8(&mut self.h, R8::H)),
            0x3D => Srl::new(ID::Reg8(&mut self.l, R8::L)),
            0x3E => Srl::new(ID::PointedByHL(bus.clone(), self.hl())),
            0x3F => Srl::new(ID::Reg8(&mut self.a, R8::A)),
            0x40..=0x7F => Bit::new(
                bit,
                match cb_opcode & 0x07 {
                    0 => IT::Reg8(self.b, R8::B),
                    1 => IT::Reg8(self.c, R8::C),
                    2 => IT::Reg8(self.d, R8::D),
                    3 => IT::Reg8(self.e, R8::E),
                    4 => IT::Reg8(self.h, R8::H),
                    5 => IT::Reg8(self.l, R8::L),
                    6 => IT::PointedByHL(bus.borrow()[self.hl()]),
                    7 => IT::Reg8(self.a, R8::A),
                    _ => return Err(InstructionError::OutOfRangeCBOpcode(cb_opcode, self.pc)),
                },
            ),
            0x80..=0xBF => Res::new(
                bit,
                match cb_opcode & 0x07 {
                    0 => ID::Reg8(&mut self.b, R8::B),
                    1 => ID::Reg8(&mut self.c, R8::C),
                    2 => ID::Reg8(&mut self.d, R8::D),
                    3 => ID::Reg8(&mut self.e, R8::E),
                    4 => ID::Reg8(&mut self.h, R8::H),
                    5 => ID::Reg8(&mut self.l, R8::L),
                    6 => ID::PointedByHL(bus.clone(), self.hl()),
                    7 => ID::Reg8(&mut self.a, R8::A),
                    _ => return Err(InstructionError::OutOfRangeCBOpcode(cb_opcode, self.pc)),
                },
            ),
            0xC0..=0xFF => Set::new(
                bit,
                match cb_opcode & 0x0F {
                    0 => ID::Reg8(&mut self.b, R8::B),
                    1 => ID::Reg8(&mut self.c, R8::C),
                    2 => ID::Reg8(&mut self.d, R8::D),
                    3 => ID::Reg8(&mut self.e, R8::E),
                    4 => ID::Reg8(&mut self.h, R8::H),
                    5 => ID::Reg8(&mut self.l, R8::L),
                    6 => ID::PointedByHL(bus.clone(), self.hl()),
                    7 => ID::Reg8(&mut self.a, R8::A),
                    _ => return Err(InstructionError::OutOfRangeCBOpcode(cb_opcode, self.pc)),
                },
            ),
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
