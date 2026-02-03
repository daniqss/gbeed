mod flags;
mod instructions;
mod registers;

use crate::{
    Dmg,
    core::{
        cpu::flags::{CARRY_FLAG_MASK, ZERO_FLAG_MASK},
        memory::Accessable,
    },
    utils::to_u16,
};
pub use instructions::{Instruction, Len};
use instructions::{InstructionDestination as ID, InstructionTarget as IT, JumpCondition as JC, *};
pub use registers::{Reg8 as R8, Reg16 as R16};
use std::fmt::{self, Display, Formatter};

pub type FetchResult = std::result::Result<Box<dyn Instruction>, InstructionError>;

pub const AFTER_BOOT_CPU: Cpu = Cpu {
    a: 0x01,
    f: 0xB0,
    b: 0x00,
    c: 0x13,
    d: 0x00,
    e: 0xD8,
    h: 0x01,
    l: 0x4D,
    pc: 0x0100,
    sp: 0xFFFE,
    cycles: 60814,
    ime: false,
};

/// # CPU
/// Gameboy CPU, with a mix of Intel 8080 and Zilog Z80 features and instruction set, the Sharp LR35902.
/// Most of its register are 8-bits ones, that are commonly used as pairs to perform 16-bits operations.
/// The only 16-bits registers are the stack pointer (SP) and the program counter (PC).
#[derive(Debug, Default, PartialEq)]
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
        if start_at_boot {
            Cpu::default()
        } else {
            AFTER_BOOT_CPU
        }
    }

    pub fn af(&self) -> u16 { to_u16(self.f, self.a) }
    pub fn bc(&self) -> u16 { to_u16(self.c, self.b) }
    pub fn de(&self) -> u16 { to_u16(self.e, self.d) }
    pub fn hl(&self) -> u16 { to_u16(self.l, self.h) }

    #[inline]
    pub fn carry(&self) -> bool { self.f & CARRY_FLAG_MASK != 0 }
    #[inline]
    pub fn not_carry(&self) -> bool { self.f & CARRY_FLAG_MASK == 0 }
    #[inline]
    pub fn zero(&self) -> bool { self.f & ZERO_FLAG_MASK != 0 }
    #[inline]
    pub fn not_zero(&self) -> bool { self.f & ZERO_FLAG_MASK == 0 }

    pub fn reset(&mut self) {
        self.a = AFTER_BOOT_CPU.a;
        self.f = AFTER_BOOT_CPU.f;
        self.b = AFTER_BOOT_CPU.b;
        self.c = AFTER_BOOT_CPU.c;
        self.d = AFTER_BOOT_CPU.d;
        self.e = AFTER_BOOT_CPU.e;
        self.h = AFTER_BOOT_CPU.h;
        self.l = AFTER_BOOT_CPU.l;
        self.pc = AFTER_BOOT_CPU.pc;
        self.sp = AFTER_BOOT_CPU.sp;
        self.ime = AFTER_BOOT_CPU.ime;
        self.cycles = AFTER_BOOT_CPU.cycles;
    }

    /// Execute instruction based on the opcode.
    /// Return a result with the effect of the instruction or an instruction error (e.g unused opcode)
    pub fn fetch(gb: &mut Dmg, opcode: u8) -> FetchResult {
        let cpu = &gb.cpu;

        let instruction: Box<dyn Instruction> = match opcode {
            0x00 => Nop::new(),
            0x01 => Ld::new(ID::Reg16(R16::BC), IT::Imm16(gb.read16(cpu.pc + 1))),
            0x02 => Ld::new(ID::PointedByReg16(cpu.bc(), R16::BC), IT::Reg8(cpu.a, R8::A)),
            0x03 => Inc::new(ID::Reg16(R16::BC)),
            0x04 => Inc::new(ID::Reg8(R8::B)),
            0x05 => Dec::new(ID::Reg8(R8::B)),
            0x06 => Ld::new(ID::Reg8(R8::B), IT::Imm8(gb[cpu.pc + 1])),
            0x07 => Rlca::new(),
            0x08 => Ld::new(ID::PointedByN16(gb.read16(cpu.pc + 1)), IT::StackPointer(cpu.sp)),
            0x09 => Add::new(ID::Reg16(R16::HL), IT::Reg16(cpu.bc(), R16::BC)),
            0x0A => Ld::new(ID::Reg8(R8::A), IT::PointedByReg16(gb[cpu.bc()], R16::BC)),
            0x0B => Dec::new(ID::Reg16(R16::BC)),
            0x0C => Inc::new(ID::Reg8(R8::C)),
            0x0D => Dec::new(ID::Reg8(R8::C)),
            0x0E => Ld::new(ID::Reg8(R8::C), IT::Imm8(gb[cpu.pc + 1])),
            0x0F => Rrca::new(),
            0x10 => Stop::new(),
            0x11 => Ld::new(ID::Reg16(R16::DE), IT::Imm16(gb.read16(cpu.pc + 1))),
            0x12 => Ld::new(ID::PointedByReg16(cpu.de(), R16::DE), IT::Reg8(cpu.a, R8::A)),
            0x13 => Inc::new(ID::Reg16(R16::DE)),
            0x14 => Inc::new(ID::Reg8(R8::D)),
            0x15 => Dec::new(ID::Reg8(R8::D)),
            0x16 => Ld::new(ID::Reg8(R8::D), IT::Imm8(gb[cpu.pc + 1])),
            0x17 => Rla::new(cpu.carry()),
            0x18 => Jr::new(IT::JumpToImm8(JC::None, gb[cpu.pc + 1])),
            0x19 => Add::new(ID::Reg16(R16::HL), IT::Reg16(cpu.de(), R16::DE)),
            0x1A => Ld::new(ID::Reg8(R8::A), IT::PointedByReg16(gb[cpu.de()], R16::DE)),
            0x1B => Dec::new(ID::Reg16(R16::DE)),
            0x1C => Inc::new(ID::Reg8(R8::E)),
            0x1D => Dec::new(ID::Reg8(R8::E)),
            0x1E => Ld::new(ID::Reg8(R8::E), IT::Imm8(gb[cpu.pc + 1])),
            0x1F => Rra::new(cpu.carry()),
            0x20 => Jr::new(IT::JumpToImm8(JC::NotZero(cpu.not_zero()), gb[cpu.pc + 1])),
            0x21 => Ld::new(ID::Reg16(R16::HL), IT::Imm16(gb.read16(cpu.pc + 1))),
            0x22 => Ld::new(ID::PointedByHLI(cpu.hl()), IT::Reg8(cpu.a, R8::A)),
            0x23 => Inc::new(ID::Reg16(R16::HL)),
            0x24 => Inc::new(ID::Reg8(R8::H)),
            0x25 => Dec::new(ID::Reg8(R8::H)),
            0x26 => Ld::new(ID::Reg8(R8::H), IT::Imm8(gb[cpu.pc + 1])),
            0x27 => Daa::new(),
            0x28 => Jr::new(IT::JumpToImm8(JC::Zero(cpu.zero()), gb[cpu.pc + 1])),
            0x29 => Add::new(ID::Reg16(R16::HL), IT::Reg16(cpu.hl(), R16::HL)),
            0x2A => Ld::new(ID::Reg8(R8::A), IT::PointedByHLI(gb[cpu.hl()])),
            0x2b => Dec::new(ID::Reg16(R16::HL)),
            0x2C => Inc::new(ID::Reg8(R8::L)),
            0x2D => Dec::new(ID::Reg8(R8::L)),
            0x2E => Ld::new(ID::Reg8(R8::L), IT::Imm8(gb[cpu.pc + 1])),
            0x2F => Cpl::new(),
            0x30 => Jr::new(IT::JumpToImm8(JC::NotCarry(cpu.not_carry()), gb[cpu.pc + 1])),
            0x31 => Ld::new(ID::StackPointer, IT::Imm16(gb.read16(cpu.pc + 1))),
            0x32 => Ld::new(ID::PointedByHLD(cpu.hl()), IT::Reg8(cpu.a, R8::A)),
            0x33 => Inc::new(ID::StackPointer),
            0x34 => Inc::new(ID::PointedByHL(cpu.hl())),
            0x35 => Dec::new(ID::PointedByHL(cpu.hl())),
            0x36 => Ld::new(ID::PointedByHL(cpu.hl()), IT::Imm8(gb[cpu.pc + 1])),
            0x37 => Scf::new(),
            0x38 => Jr::new(IT::JumpToImm8(JC::Carry(cpu.carry()), gb[cpu.pc + 1])),
            0x39 => Add::new(ID::Reg16(R16::HL), IT::StackPointer(cpu.sp)),
            0x3A => Ld::new(ID::Reg8(R8::A), IT::PointedByHLD(gb[cpu.hl()])),
            0x3B => Dec::new(ID::StackPointer),
            0x3C => Inc::new(ID::Reg8(R8::A)),
            0x3D => Dec::new(ID::Reg8(R8::A)),
            0x3E => Ld::new(ID::Reg8(R8::A), IT::Imm8(gb[cpu.pc + 1])),
            0x3F => Ccf::new(cpu.carry()),
            0x40 => Ld::new(ID::Reg8(R8::B), IT::Reg8(cpu.b, R8::B)),
            0x41 => Ld::new(ID::Reg8(R8::B), IT::Reg8(cpu.c, R8::C)),
            0x42 => Ld::new(ID::Reg8(R8::B), IT::Reg8(cpu.d, R8::D)),
            0x43 => Ld::new(ID::Reg8(R8::B), IT::Reg8(cpu.e, R8::E)),
            0x44 => Ld::new(ID::Reg8(R8::B), IT::Reg8(cpu.h, R8::H)),
            0x45 => Ld::new(ID::Reg8(R8::B), IT::Reg8(cpu.l, R8::L)),
            0x46 => Ld::new(ID::Reg8(R8::B), IT::PointedByHL(gb[cpu.hl()])),
            0x47 => Ld::new(ID::Reg8(R8::B), IT::Reg8(cpu.a, R8::A)),
            0x48 => Ld::new(ID::Reg8(R8::C), IT::Reg8(cpu.b, R8::B)),
            0x49 => Ld::new(ID::Reg8(R8::C), IT::Reg8(cpu.c, R8::C)),
            0x4A => Ld::new(ID::Reg8(R8::C), IT::Reg8(cpu.d, R8::D)),
            0x4B => Ld::new(ID::Reg8(R8::C), IT::Reg8(cpu.e, R8::E)),
            0x4C => Ld::new(ID::Reg8(R8::C), IT::Reg8(cpu.h, R8::H)),
            0x4D => Ld::new(ID::Reg8(R8::C), IT::Reg8(cpu.l, R8::L)),
            0x4E => Ld::new(ID::Reg8(R8::C), IT::PointedByHL(gb[cpu.hl()])),
            0x4F => Ld::new(ID::Reg8(R8::C), IT::Reg8(cpu.a, R8::A)),
            0x50 => Ld::new(ID::Reg8(R8::D), IT::Reg8(cpu.b, R8::B)),
            0x51 => Ld::new(ID::Reg8(R8::D), IT::Reg8(cpu.c, R8::C)),
            0x52 => Ld::new(ID::Reg8(R8::D), IT::Reg8(cpu.d, R8::D)),
            0x53 => Ld::new(ID::Reg8(R8::D), IT::Reg8(cpu.e, R8::E)),
            0x54 => Ld::new(ID::Reg8(R8::D), IT::Reg8(cpu.h, R8::H)),
            0x55 => Ld::new(ID::Reg8(R8::D), IT::Reg8(cpu.l, R8::L)),
            0x56 => Ld::new(ID::Reg8(R8::D), IT::PointedByHL(gb[cpu.hl()])),
            0x57 => Ld::new(ID::Reg8(R8::D), IT::Reg8(cpu.a, R8::A)),
            0x58 => Ld::new(ID::Reg8(R8::E), IT::Reg8(cpu.b, R8::B)),
            0x59 => Ld::new(ID::Reg8(R8::E), IT::Reg8(cpu.c, R8::C)),
            0x5A => Ld::new(ID::Reg8(R8::E), IT::Reg8(cpu.d, R8::D)),
            0x5B => Ld::new(ID::Reg8(R8::E), IT::Reg8(cpu.e, R8::E)),
            0x5C => Ld::new(ID::Reg8(R8::E), IT::Reg8(cpu.h, R8::H)),
            0x5D => Ld::new(ID::Reg8(R8::E), IT::Reg8(cpu.l, R8::L)),
            0x5E => Ld::new(ID::Reg8(R8::E), IT::PointedByHL(gb[cpu.hl()])),
            0x5F => Ld::new(ID::Reg8(R8::E), IT::Reg8(cpu.a, R8::A)),
            0x60 => Ld::new(ID::Reg8(R8::H), IT::Reg8(cpu.b, R8::B)),
            0x61 => Ld::new(ID::Reg8(R8::H), IT::Reg8(cpu.c, R8::C)),
            0x62 => Ld::new(ID::Reg8(R8::H), IT::Reg8(cpu.d, R8::D)),
            0x63 => Ld::new(ID::Reg8(R8::H), IT::Reg8(cpu.e, R8::E)),
            0x64 => Ld::new(ID::Reg8(R8::H), IT::Reg8(cpu.h, R8::H)),
            0x65 => Ld::new(ID::Reg8(R8::H), IT::Reg8(cpu.l, R8::L)),
            0x66 => Ld::new(ID::Reg8(R8::H), IT::PointedByHL(gb[cpu.hl()])),
            0x67 => Ld::new(ID::Reg8(R8::H), IT::Reg8(cpu.a, R8::A)),
            0x68 => Ld::new(ID::Reg8(R8::L), IT::Reg8(cpu.b, R8::B)),
            0x69 => Ld::new(ID::Reg8(R8::L), IT::Reg8(cpu.c, R8::C)),
            0x6A => Ld::new(ID::Reg8(R8::L), IT::Reg8(cpu.d, R8::D)),
            0x6B => Ld::new(ID::Reg8(R8::L), IT::Reg8(cpu.e, R8::E)),
            0x6C => Ld::new(ID::Reg8(R8::L), IT::Reg8(cpu.h, R8::H)),
            0x6D => Ld::new(ID::Reg8(R8::L), IT::Reg8(cpu.l, R8::L)),
            0x6E => Ld::new(ID::Reg8(R8::L), IT::PointedByHL(gb[cpu.hl()])),
            0x6F => Ld::new(ID::Reg8(R8::L), IT::Reg8(cpu.a, R8::A)),
            0x70 => Ld::new(ID::PointedByHL(cpu.hl()), IT::Reg8(cpu.b, R8::B)),
            0x71 => Ld::new(ID::PointedByHL(cpu.hl()), IT::Reg8(cpu.c, R8::C)),
            0x72 => Ld::new(ID::PointedByHL(cpu.hl()), IT::Reg8(cpu.d, R8::D)),
            0x73 => Ld::new(ID::PointedByHL(cpu.hl()), IT::Reg8(cpu.e, R8::E)),
            0x74 => Ld::new(ID::PointedByHL(cpu.hl()), IT::Reg8(cpu.h, R8::H)),
            0x75 => Ld::new(ID::PointedByHL(cpu.hl()), IT::Reg8(cpu.l, R8::L)),
            0x76 => Halt::new(),
            0x77 => Ld::new(ID::PointedByHL(cpu.hl()), IT::Reg8(cpu.a, R8::A)),
            0x78 => Ld::new(ID::Reg8(R8::A), IT::Reg8(cpu.b, R8::B)),
            0x79 => Ld::new(ID::Reg8(R8::A), IT::Reg8(cpu.c, R8::C)),
            0x7A => Ld::new(ID::Reg8(R8::A), IT::Reg8(cpu.d, R8::D)),
            0x7B => Ld::new(ID::Reg8(R8::A), IT::Reg8(cpu.e, R8::E)),
            0x7C => Ld::new(ID::Reg8(R8::A), IT::Reg8(cpu.h, R8::H)),
            0x7D => Ld::new(ID::Reg8(R8::A), IT::Reg8(cpu.l, R8::L)),
            0x7E => Ld::new(ID::Reg8(R8::A), IT::PointedByHL(gb[cpu.hl()])),
            0x7F => Ld::new(ID::Reg8(R8::A), IT::Reg8(cpu.a, R8::A)),
            0x80 => Add::new(ID::Reg8(R8::A), IT::Reg8(cpu.b, R8::B)),
            0x81 => Add::new(ID::Reg8(R8::A), IT::Reg8(cpu.c, R8::C)),
            0x82 => Add::new(ID::Reg8(R8::A), IT::Reg8(cpu.d, R8::D)),
            0x83 => Add::new(ID::Reg8(R8::A), IT::Reg8(cpu.e, R8::E)),
            0x84 => Add::new(ID::Reg8(R8::A), IT::Reg8(cpu.h, R8::H)),
            0x85 => Add::new(ID::Reg8(R8::A), IT::Reg8(cpu.l, R8::L)),
            0x86 => Add::new(ID::Reg8(R8::A), IT::PointedByHL(gb[cpu.hl()])),
            0x87 => Add::new(ID::Reg8(R8::A), IT::Reg8(cpu.a, R8::A)),
            0x88 => Adc::new(cpu.carry(), IT::Reg8(cpu.b, R8::B)),
            0x89 => Adc::new(cpu.carry(), IT::Reg8(cpu.c, R8::C)),
            0x8A => Adc::new(cpu.carry(), IT::Reg8(cpu.d, R8::D)),
            0x8B => Adc::new(cpu.carry(), IT::Reg8(cpu.e, R8::E)),
            0x8C => Adc::new(cpu.carry(), IT::Reg8(cpu.h, R8::H)),
            0x8D => Adc::new(cpu.carry(), IT::Reg8(cpu.l, R8::L)),
            0x8E => Adc::new(cpu.carry(), IT::PointedByHL(gb[cpu.hl()])),
            0x8F => Adc::new(cpu.carry(), IT::Reg8(cpu.a, R8::A)),
            0x90 => Sub::new(IT::Reg8(cpu.b, R8::B)),
            0x91 => Sub::new(IT::Reg8(cpu.c, R8::C)),
            0x92 => Sub::new(IT::Reg8(cpu.d, R8::D)),
            0x93 => Sub::new(IT::Reg8(cpu.e, R8::E)),
            0x94 => Sub::new(IT::Reg8(cpu.h, R8::H)),
            0x95 => Sub::new(IT::Reg8(cpu.l, R8::L)),
            0x96 => Sub::new(IT::PointedByHL(gb[cpu.hl()])),
            0x97 => Sub::new(IT::Reg8(cpu.a, R8::A)),
            0x98 => Sbc::new(cpu.carry(), IT::Reg8(cpu.b, R8::B)),
            0x99 => Sbc::new(cpu.carry(), IT::Reg8(cpu.c, R8::C)),
            0x9A => Sbc::new(cpu.carry(), IT::Reg8(cpu.d, R8::D)),
            0x9B => Sbc::new(cpu.carry(), IT::Reg8(cpu.e, R8::E)),
            0x9C => Sbc::new(cpu.carry(), IT::Reg8(cpu.h, R8::H)),
            0x9D => Sbc::new(cpu.carry(), IT::Reg8(cpu.l, R8::L)),
            0x9E => Sbc::new(cpu.carry(), IT::PointedByHL(gb[cpu.hl()])),
            0x9F => Sbc::new(cpu.carry(), IT::Reg8(cpu.a, R8::A)),
            0xA0 => And::new(IT::Reg8(cpu.b, R8::B)),
            0xA1 => And::new(IT::Reg8(cpu.c, R8::C)),
            0xA2 => And::new(IT::Reg8(cpu.d, R8::D)),
            0xA3 => And::new(IT::Reg8(cpu.e, R8::E)),
            0xA4 => And::new(IT::Reg8(cpu.h, R8::H)),
            0xA5 => And::new(IT::Reg8(cpu.l, R8::L)),
            0xA6 => And::new(IT::PointedByHL(gb[cpu.hl()])),
            0xA7 => And::new(IT::Reg8(cpu.a, R8::A)),
            0xA8 => Xor::new(IT::Reg8(cpu.b, R8::B)),
            0xA9 => Xor::new(IT::Reg8(cpu.c, R8::C)),
            0xAA => Xor::new(IT::Reg8(cpu.d, R8::D)),
            0xAB => Xor::new(IT::Reg8(cpu.e, R8::E)),
            0xAC => Xor::new(IT::Reg8(cpu.h, R8::H)),
            0xAD => Xor::new(IT::Reg8(cpu.l, R8::L)),
            0xAE => Xor::new(IT::PointedByHL(gb[cpu.hl()])),
            0xAF => Xor::new(IT::Reg8(cpu.a, R8::A)),
            0xB0 => Or::new(IT::Reg8(cpu.b, R8::B)),
            0xB1 => Or::new(IT::Reg8(cpu.c, R8::C)),
            0xB2 => Or::new(IT::Reg8(cpu.d, R8::D)),
            0xB3 => Or::new(IT::Reg8(cpu.e, R8::E)),
            0xB4 => Or::new(IT::Reg8(cpu.h, R8::H)),
            0xB5 => Or::new(IT::Reg8(cpu.l, R8::L)),
            0xB6 => Or::new(IT::PointedByHL(gb[cpu.hl()])),
            0xB7 => Or::new(IT::Reg8(cpu.a, R8::A)),
            0xB8 => Cp::new(IT::Reg8(cpu.b, R8::B)),
            0xB9 => Cp::new(IT::Reg8(cpu.c, R8::C)),
            0xBA => Cp::new(IT::Reg8(cpu.d, R8::D)),
            0xBB => Cp::new(IT::Reg8(cpu.e, R8::E)),
            0xBC => Cp::new(IT::Reg8(cpu.h, R8::H)),
            0xBD => Cp::new(IT::Reg8(cpu.l, R8::L)),
            0xBE => Cp::new(IT::PointedByHL(gb[cpu.hl()])),
            0xBF => Cp::new(IT::Reg8(cpu.a, R8::A)),
            0xC0 => Ret::new(JC::NotZero(cpu.not_zero())),
            0xC1 => Pop::new(ID::Reg16(R16::BC)),
            0xC2 => Jp::new(IT::JumpToImm16(
                JC::NotZero(cpu.not_zero()),
                gb.read16(cpu.pc + 1),
            )),
            0xC3 => Jp::new(IT::JumpToImm16(JC::None, gb.read16(cpu.pc + 1))),
            0xC4 => Call::new(IT::JumpToImm16(
                JC::NotZero(cpu.not_zero()),
                gb.read16(cpu.sp + 1),
            )),
            0xC5 => Push::new(IT::Reg16(cpu.bc(), R16::BC)),
            0xC6 => Add::new(ID::Reg8(R8::A), IT::Imm8(gb[cpu.pc + 1])),
            0xC7 => Rst::new(0x00),
            0xC8 => Ret::new(JC::Zero(cpu.zero())),
            0xC9 => Ret::new(JC::None),
            0xCA => Jp::new(IT::JumpToImm16(JC::Zero(cpu.zero()), gb.read16(cpu.pc + 1))),
            0xCB => {
                let cb_opcode = gb[cpu.pc + 1];
                match Cpu::fetch_cb(gb, cb_opcode) {
                    Ok(instruction) => instruction,
                    Err(e) => return Err(e),
                }
            }
            0xCC => Call::new(IT::JumpToImm16(JC::Zero(cpu.zero()), gb.read16(cpu.pc + 1))),
            0xCD => Call::new(IT::JumpToImm16(JC::None, gb.read16(cpu.pc + 1))),
            0xCE => Adc::new(cpu.carry(), IT::Imm8(gb[cpu.pc + 1])),
            0xCF => Rst::new(0x08),
            0xD0 => Ret::new(JC::NotCarry(cpu.not_carry())),
            0xD1 => Pop::new(ID::Reg16(R16::DE)),
            0xD2 => Jp::new(IT::JumpToImm16(
                JC::NotCarry(cpu.not_carry()),
                gb.read16(cpu.pc + 1),
            )),
            0xD3 => return Err(InstructionError::UnusedOpcode(opcode, cpu.pc)),
            0xD4 => Call::new(IT::JumpToImm16(
                JC::NotCarry(cpu.not_carry()),
                gb.read16(cpu.pc + 1),
            )),
            0xD5 => Push::new(IT::Reg16(cpu.de(), R16::DE)),
            0xD6 => Sub::new(IT::Imm8(gb[cpu.pc + 1])),
            0xD7 => Rst::new(0x10),
            0xD8 => Ret::new(JC::Carry(cpu.carry())),
            0xD9 => Reti::new(),
            0xDA => Jp::new(IT::JumpToImm16(JC::Carry(cpu.carry()), gb.read16(cpu.pc + 1))),
            0xDB => return Err(InstructionError::UnusedOpcode(opcode, cpu.pc)),
            0xDC => Call::new(IT::JumpToImm16(JC::Carry(cpu.carry()), gb.read16(cpu.pc + 1))),
            0xDD => return Err(InstructionError::UnusedOpcode(opcode, cpu.pc)),
            0xDE => Sbc::new(cpu.carry(), IT::Imm8(gb[cpu.pc + 1])),
            0xDF => Rst::new(0x18),
            0xE0 => Ldh::new(ID::PointedByA8(gb[cpu.pc + 1]), IT::Reg8(cpu.a, R8::A)),
            0xE1 => Pop::new(ID::Reg16(R16::HL)),
            0xE2 => Ldh::new(ID::PointedByCPlusFF00, IT::Reg8(cpu.a, R8::A)),
            0xE3 => return Err(InstructionError::UnusedOpcode(opcode, cpu.pc)),
            0xE4 => return Err(InstructionError::UnusedOpcode(opcode, cpu.pc)),
            0xE5 => Push::new(IT::Reg16(cpu.hl(), R16::HL)),
            0xE6 => And::new(IT::Imm8(gb[cpu.pc + 1])),
            0xE7 => Rst::new(0x20),
            0xE8 => Add::new(ID::StackPointer, IT::SignedImm(gb[cpu.pc + 1] as i8)),
            0xE9 => Jp::new(IT::JumpToHL(cpu.hl())),
            0xEA => Ld::new(ID::PointedByN16(gb.read16(cpu.pc + 1)), IT::Reg8(cpu.a, R8::A)),
            0xEB => return Err(InstructionError::UnusedOpcode(opcode, cpu.pc)),
            0xEC => return Err(InstructionError::UnusedOpcode(opcode, cpu.pc)),
            0xED => return Err(InstructionError::UnusedOpcode(opcode, cpu.pc)),
            0xEE => Xor::new(IT::Imm8(gb[cpu.pc + 1])),
            0xEF => Rst::new(0x28),
            0xF0 => Ldh::new(ID::Reg8(R8::A), IT::PointedByA8(gb[cpu.pc + 1])),
            0xF1 => Pop::new(ID::Reg16(R16::AF)),
            0xF2 => Ldh::new(ID::Reg8(R8::A), IT::PointedByCPlusFF00),
            0xF3 => Di::new(),
            0xF4 => return Err(InstructionError::UnusedOpcode(opcode, cpu.pc)),
            0xF5 => Push::new(IT::Reg16(cpu.af(), R16::AF)),
            0xF6 => Or::new(IT::Imm8(gb[cpu.pc + 1])),
            0xF7 => Rst::new(0x30),
            0xF8 => Ld::new(
                ID::Reg16(R16::HL),
                IT::StackPointerPlusE8(cpu.sp, gb[cpu.pc + 1] as i8),
            ),
            0xF9 => Ld::new(ID::StackPointer, IT::Reg16(cpu.hl(), R16::HL)),
            0xFA => Ld::new(ID::Reg8(R8::A), IT::PointedByN16(gb.read16(cpu.pc + 1))),
            0xFB => Ei::new(),
            0xFC => return Err(InstructionError::UnusedOpcode(opcode, cpu.pc)),
            0xFD => return Err(InstructionError::UnusedOpcode(opcode, cpu.pc)),
            0xFE => Cp::new(IT::Imm8(gb[cpu.pc + 1])),
            0xFF => Rst::new(0x38),
        };

        Ok(instruction)
    }

    fn fetch_cb(gb: &mut Dmg, cb_opcode: u8) -> FetchResult {
        // used bit in res, set and bit instructions
        let bit = (cb_opcode & 0x38) >> 3;
        let cpu = &gb.cpu;

        let instruction: Box<dyn Instruction> = match cb_opcode {
            0x00 => Rlc::new(ID::Reg8(R8::B)),
            0x01 => Rlc::new(ID::Reg8(R8::C)),
            0x02 => Rlc::new(ID::Reg8(R8::D)),
            0x03 => Rlc::new(ID::Reg8(R8::E)),
            0x04 => Rlc::new(ID::Reg8(R8::H)),
            0x05 => Rlc::new(ID::Reg8(R8::L)),
            0x06 => Rlc::new(ID::PointedByHL(cpu.hl())),
            0x07 => Rlc::new(ID::Reg8(R8::A)),
            0x08 => Rrc::new(ID::Reg8(R8::B)),
            0x09 => Rrc::new(ID::Reg8(R8::C)),
            0x0A => Rrc::new(ID::Reg8(R8::D)),
            0x0B => Rrc::new(ID::Reg8(R8::E)),
            0x0C => Rrc::new(ID::Reg8(R8::H)),
            0x0D => Rrc::new(ID::Reg8(R8::L)),
            0x0E => Rrc::new(ID::PointedByHL(cpu.hl())),
            0x0F => Rrc::new(ID::Reg8(R8::A)),
            0x10 => Rl::new(cpu.carry(), ID::Reg8(R8::B)),
            0x11 => Rl::new(cpu.carry(), ID::Reg8(R8::C)),
            0x12 => Rl::new(cpu.carry(), ID::Reg8(R8::D)),
            0x13 => Rl::new(cpu.carry(), ID::Reg8(R8::E)),
            0x14 => Rl::new(cpu.carry(), ID::Reg8(R8::H)),
            0x15 => Rl::new(cpu.carry(), ID::Reg8(R8::L)),
            0x16 => Rl::new(cpu.carry(), ID::PointedByHL(cpu.hl())),
            0x17 => Rl::new(cpu.carry(), ID::Reg8(R8::A)),
            0x18 => Rr::new(cpu.carry(), ID::Reg8(R8::B)),
            0x19 => Rr::new(cpu.carry(), ID::Reg8(R8::C)),
            0x1A => Rr::new(cpu.carry(), ID::Reg8(R8::D)),
            0x1B => Rr::new(cpu.carry(), ID::Reg8(R8::E)),
            0x1C => Rr::new(cpu.carry(), ID::Reg8(R8::H)),
            0x1D => Rr::new(cpu.carry(), ID::Reg8(R8::L)),
            0x1E => Rr::new(cpu.carry(), ID::PointedByHL(cpu.hl())),
            0x1F => Rr::new(cpu.carry(), ID::Reg8(R8::A)),
            0x20 => Sla::new(ID::Reg8(R8::B)),
            0x21 => Sla::new(ID::Reg8(R8::C)),
            0x22 => Sla::new(ID::Reg8(R8::D)),
            0x23 => Sla::new(ID::Reg8(R8::E)),
            0x24 => Sla::new(ID::Reg8(R8::H)),
            0x25 => Sla::new(ID::Reg8(R8::L)),
            0x26 => Sla::new(ID::PointedByHL(cpu.hl())),
            0x27 => Sla::new(ID::Reg8(R8::A)),
            0x28 => Sra::new(ID::Reg8(R8::B)),
            0x29 => Sra::new(ID::Reg8(R8::C)),
            0x2A => Sra::new(ID::Reg8(R8::D)),
            0x2B => Sra::new(ID::Reg8(R8::E)),
            0x2C => Sra::new(ID::Reg8(R8::H)),
            0x2D => Sra::new(ID::Reg8(R8::L)),
            0x2E => Sra::new(ID::PointedByHL(cpu.hl())),
            0x2F => Sra::new(ID::Reg8(R8::A)),
            0x30 => Swap::new(ID::Reg8(R8::B)),
            0x31 => Swap::new(ID::Reg8(R8::C)),
            0x32 => Swap::new(ID::Reg8(R8::D)),
            0x33 => Swap::new(ID::Reg8(R8::E)),
            0x34 => Swap::new(ID::Reg8(R8::H)),
            0x35 => Swap::new(ID::Reg8(R8::L)),
            0x36 => Swap::new(ID::PointedByHL(cpu.hl())),
            0x37 => Swap::new(ID::Reg8(R8::A)),
            0x38 => Srl::new(ID::Reg8(R8::B)),
            0x39 => Srl::new(ID::Reg8(R8::C)),
            0x3A => Srl::new(ID::Reg8(R8::D)),
            0x3B => Srl::new(ID::Reg8(R8::E)),
            0x3C => Srl::new(ID::Reg8(R8::H)),
            0x3D => Srl::new(ID::Reg8(R8::L)),
            0x3E => Srl::new(ID::PointedByHL(cpu.hl())),
            0x3F => Srl::new(ID::Reg8(R8::A)),
            0x40..=0x7F => Bit::new(
                bit,
                match cb_opcode & 0x07 {
                    0 => IT::Reg8(cpu.b, R8::B),
                    1 => IT::Reg8(cpu.c, R8::C),
                    2 => IT::Reg8(cpu.d, R8::D),
                    3 => IT::Reg8(cpu.e, R8::E),
                    4 => IT::Reg8(cpu.h, R8::H),
                    5 => IT::Reg8(cpu.l, R8::L),
                    6 => IT::PointedByHL(gb[cpu.hl()]),
                    7 => IT::Reg8(cpu.a, R8::A),
                    _ => return Err(InstructionError::OutOfRangeCBOpcode(cb_opcode, cpu.pc)),
                },
            ),
            0x80..=0xBF => Res::new(
                bit,
                match cb_opcode & 0x07 {
                    0 => ID::Reg8(R8::B),
                    1 => ID::Reg8(R8::C),
                    2 => ID::Reg8(R8::D),
                    3 => ID::Reg8(R8::E),
                    4 => ID::Reg8(R8::H),
                    5 => ID::Reg8(R8::L),
                    6 => ID::PointedByHL(cpu.hl()),
                    7 => ID::Reg8(R8::A),
                    _ => return Err(InstructionError::OutOfRangeCBOpcode(cb_opcode, cpu.pc)),
                },
            ),
            0xC0..=0xFF => Set::new(
                bit,
                match cb_opcode & 0x0F {
                    0 => ID::Reg8(R8::B),
                    1 => ID::Reg8(R8::C),
                    2 => ID::Reg8(R8::D),
                    3 => ID::Reg8(R8::E),
                    4 => ID::Reg8(R8::H),
                    5 => ID::Reg8(R8::L),
                    6 => ID::PointedByHL(cpu.hl()),
                    7 => ID::Reg8(R8::A),
                    _ => return Err(InstructionError::OutOfRangeCBOpcode(cb_opcode, cpu.pc)),
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
            "a: {:02X} f: {:02X} b: {:02X} c: {:02X} d: {:02X} e: {:02X} h: {:02X} l: {:02X} pc: {:04X} sp: {:04X}, cycles: {}",
            self.a, self.f, self.b, self.c, self.d, self.e, self.h, self.l, self.pc, self.sp, self.cycles
        )
    }
}
