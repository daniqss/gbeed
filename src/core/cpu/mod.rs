mod flags;
mod instructions;
mod registers;

use crate::{
    Dmg,
    core::{
        Accessible, Accessible16,
        cpu::flags::{CARRY_FLAG_MASK, HALF_CARRY_FLAG_MASK, SUBTRACTION_FLAG_MASK, ZERO_FLAG_MASK},
    },
    prelude::*,
    utils::{from_u16, high, low, to_u16},
};
pub use instructions::{Instruction, Len};
use instructions::{JumpCondition as JC, *};
pub use registers::{Register8 as R8, Register16 as R16};

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
    halted: false,
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
    pub halted: bool,
}

impl Cpu {
    pub fn new(start_at_boot: bool) -> Cpu {
        if start_at_boot {
            Cpu::default()
        } else {
            AFTER_BOOT_CPU
        }
    }

    reg16!(af, set_af, a, f);
    reg16!(bc, set_bc, b, c);
    reg16!(de, set_de, d, e);
    reg16!(hl, set_hl, h, l);

    flag_methods! {
        carry => CARRY_FLAG_MASK,
        zero => ZERO_FLAG_MASK,
        subtraction => SUBTRACTION_FLAG_MASK,
        half_carry => HALF_CARRY_FLAG_MASK,
    }

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
        self.halted = AFTER_BOOT_CPU.halted;
    }

    pub fn service_interrupt(gb: &mut Dmg, service_routine_addr: u16, interrupt_mask: u8) {
        // we need a push function
        let mut sp = gb.cpu.sp.wrapping_sub(1);
        gb.write(sp, high(gb.cpu.pc));
        sp = sp.wrapping_sub(1);
        gb.write(sp, low(gb.cpu.pc));
        gb.cpu.sp = sp;

        gb.interrupt_flag.0 &= !interrupt_mask;
        gb.cpu.pc = service_routine_addr;
    }

    /// Execute instruction based on the opcode.
    /// Return a result with the effect of the instruction or an instruction error (e.g unused opcode)
    pub fn fetch(gb: &mut Dmg, opcode: u8) -> FetchResult {
        let cpu = &gb.cpu;

        let instruction: Box<dyn Instruction> = match opcode {
            0x00 => Nop::new(),
            0x01 => LdR16Imm16::new(R16::BC, gb.load(cpu.pc + 1)),
            0x02 => LdPointedByR16A::new(R16::BC),
            0x03 => IncR16::new(R16::BC),
            0x04 => IncR8::new(R8::B),
            0x05 => DecR8::new(R8::B),
            0x06 => LdR8Imm8::new(R8::B, gb.read(cpu.pc + 1)),
            0x07 => Rlca::new(),
            0x08 => LdImm16SP::new(gb.load(cpu.pc + 1)),
            0x09 => AddR16::new(R16::BC),
            0x0A => LdAPointedByR16::new(R16::BC),
            0x0B => DecR16::new(R16::BC),
            0x0C => IncR8::new(R8::C),
            0x0D => DecR8::new(R8::C),
            0x0E => LdR8Imm8::new(R8::C, gb.read(cpu.pc + 1)),
            0x0F => Rrca::new(),
            0x10 => Stop::new(),
            0x11 => LdR16Imm16::new(R16::DE, gb.load(cpu.pc + 1)),
            0x12 => LdPointedByR16A::new(R16::DE),
            0x13 => IncR16::new(R16::DE),
            0x14 => IncR8::new(R8::D),
            0x15 => DecR8::new(R8::D),
            0x16 => LdR8Imm8::new(R8::D, gb.read(cpu.pc + 1)),
            0x17 => Rla::new(cpu.carry()),
            0x18 => Jr::new(JC::None, gb.read(cpu.pc + 1)),
            0x19 => AddR16::new(R16::DE),
            0x1A => LdAPointedByR16::new(R16::DE),
            0x1B => DecR16::new(R16::DE),
            0x1C => IncR8::new(R8::E),
            0x1D => DecR8::new(R8::E),
            0x1E => LdR8Imm8::new(R8::E, gb.read(cpu.pc + 1)),
            0x1F => Rra::new(cpu.carry()),
            0x20 => Jr::new(JC::NotZero(cpu.not_zero()), gb.read(cpu.pc + 1)),
            0x21 => LdR16Imm16::new(R16::HL, gb.load(cpu.pc + 1)),
            0x22 => LdPointedByHLIncA::new(),
            0x23 => IncR16::new(R16::HL),
            0x24 => IncR8::new(R8::H),
            0x25 => DecR8::new(R8::H),
            0x26 => LdR8Imm8::new(R8::H, gb.read(cpu.pc + 1)),
            0x27 => Daa::new(),
            0x28 => Jr::new(JC::Zero(cpu.zero()), gb.read(cpu.pc + 1)),
            0x29 => AddR16::new(R16::HL),
            0x2A => LdAPointedByHLInc::new(),
            0x2b => DecR16::new(R16::HL),
            0x2C => IncR8::new(R8::L),
            0x2D => DecR8::new(R8::L),
            0x2E => LdR8Imm8::new(R8::L, gb.read(cpu.pc + 1)),
            0x2F => Cpl::new(),
            0x30 => Jr::new(JC::NotCarry(cpu.not_carry()), gb.read(cpu.pc + 1)),
            0x31 => LdSPImm16::new(gb.load(cpu.pc + 1)),
            0x32 => LdPointedByHLDecA::new(),
            0x33 => IncStackPointer::new(),
            0x34 => IncPointedByHL::new(),
            0x35 => DecPointedByHL::new(),
            0x36 => LdPointedByHLImm8::new(gb.read(cpu.pc + 1)),
            0x37 => Scf::new(),
            0x38 => Jr::new(JC::Carry(cpu.carry()), gb.read(cpu.pc + 1)),
            0x39 => AddSP::new(),
            0x3A => LdAPointedByHLDec::new(),
            0x3B => DecStackPointer::new(),
            0x3C => IncR8::new(R8::A),
            0x3D => DecR8::new(R8::A),
            0x3E => LdR8Imm8::new(R8::A, gb.read(cpu.pc + 1)),
            0x3F => Ccf::new(cpu.carry()),
            0x40 => LdR8R8::new(R8::B, R8::B),
            0x41 => LdR8R8::new(R8::B, R8::C),
            0x42 => LdR8R8::new(R8::B, R8::D),
            0x43 => LdR8R8::new(R8::B, R8::E),
            0x44 => LdR8R8::new(R8::B, R8::H),
            0x45 => LdR8R8::new(R8::B, R8::L),
            0x46 => LdR8PointedByHL::new(R8::B),
            0x47 => LdR8R8::new(R8::B, R8::A),
            0x48 => LdR8R8::new(R8::C, R8::B),
            0x49 => LdR8R8::new(R8::C, R8::C),
            0x4A => LdR8R8::new(R8::C, R8::D),
            0x4B => LdR8R8::new(R8::C, R8::E),
            0x4C => LdR8R8::new(R8::C, R8::H),
            0x4D => LdR8R8::new(R8::C, R8::L),
            0x4E => LdR8PointedByHL::new(R8::C),
            0x4F => LdR8R8::new(R8::C, R8::A),
            0x50 => LdR8R8::new(R8::D, R8::B),
            0x51 => LdR8R8::new(R8::D, R8::C),
            0x52 => LdR8R8::new(R8::D, R8::D),
            0x53 => LdR8R8::new(R8::D, R8::E),
            0x54 => LdR8R8::new(R8::D, R8::H),
            0x55 => LdR8R8::new(R8::D, R8::L),
            0x56 => LdR8PointedByHL::new(R8::D),
            0x57 => LdR8R8::new(R8::D, R8::A),
            0x58 => LdR8R8::new(R8::E, R8::B),
            0x59 => LdR8R8::new(R8::E, R8::C),
            0x5A => LdR8R8::new(R8::E, R8::D),
            0x5B => LdR8R8::new(R8::E, R8::E),
            0x5C => LdR8R8::new(R8::E, R8::H),
            0x5D => LdR8R8::new(R8::E, R8::L),
            0x5E => LdR8PointedByHL::new(R8::E),
            0x5F => LdR8R8::new(R8::E, R8::A),
            0x60 => LdR8R8::new(R8::H, R8::B),
            0x61 => LdR8R8::new(R8::H, R8::C),
            0x62 => LdR8R8::new(R8::H, R8::D),
            0x63 => LdR8R8::new(R8::H, R8::E),
            0x64 => LdR8R8::new(R8::H, R8::H),
            0x65 => LdR8R8::new(R8::H, R8::L),
            0x66 => LdR8PointedByHL::new(R8::H),
            0x67 => LdR8R8::new(R8::H, R8::A),
            0x68 => LdR8R8::new(R8::L, R8::B),
            0x69 => LdR8R8::new(R8::L, R8::C),
            0x6A => LdR8R8::new(R8::L, R8::D),
            0x6B => LdR8R8::new(R8::L, R8::E),
            0x6C => LdR8R8::new(R8::L, R8::H),
            0x6D => LdR8R8::new(R8::L, R8::L),
            0x6E => LdR8PointedByHL::new(R8::L),
            0x6F => LdR8R8::new(R8::L, R8::A),
            0x70 => LdPointedByHLR8::new(R8::B),
            0x71 => LdPointedByHLR8::new(R8::C),
            0x72 => LdPointedByHLR8::new(R8::D),
            0x73 => LdPointedByHLR8::new(R8::E),
            0x74 => LdPointedByHLR8::new(R8::H),
            0x75 => LdPointedByHLR8::new(R8::L),
            0x76 => Halt::new(),
            0x77 => LdPointedByHLR8::new(R8::A),
            0x78 => LdR8R8::new(R8::A, R8::B),
            0x79 => LdR8R8::new(R8::A, R8::C),
            0x7A => LdR8R8::new(R8::A, R8::D),
            0x7B => LdR8R8::new(R8::A, R8::E),
            0x7C => LdR8R8::new(R8::A, R8::H),
            0x7D => LdR8R8::new(R8::A, R8::L),
            0x7E => LdR8PointedByHL::new(R8::A),
            0x7F => LdR8R8::new(R8::A, R8::A),
            0x80 => AddAR8::new(R8::B),
            0x81 => AddAR8::new(R8::C),
            0x82 => AddAR8::new(R8::D),
            0x83 => AddAR8::new(R8::E),
            0x84 => AddAR8::new(R8::H),
            0x85 => AddAR8::new(R8::L),
            0x86 => AddAPointedByHL::new(),
            0x87 => AddAR8::new(R8::A),
            0x88 => AdcR8::new(R8::B),
            0x89 => AdcR8::new(R8::C),
            0x8A => AdcR8::new(R8::D),
            0x8B => AdcR8::new(R8::E),
            0x8C => AdcR8::new(R8::H),
            0x8D => AdcR8::new(R8::L),
            0x8E => AdcPointedByHL::new(),
            0x8F => AdcR8::new(R8::A),
            0x90 => SubR8::new(R8::B),
            0x91 => SubR8::new(R8::C),
            0x92 => SubR8::new(R8::D),
            0x93 => SubR8::new(R8::E),
            0x94 => SubR8::new(R8::H),
            0x95 => SubR8::new(R8::L),
            0x96 => SubPointedByHL::new(),
            0x97 => SubR8::new(R8::A),
            0x98 => SbcR8::new(R8::B),
            0x99 => SbcR8::new(R8::C),
            0x9A => SbcR8::new(R8::D),
            0x9B => SbcR8::new(R8::E),
            0x9C => SbcR8::new(R8::H),
            0x9D => SbcR8::new(R8::L),
            0x9E => SbcPointedByHL::new(),
            0x9F => SbcR8::new(R8::A),
            0xA0 => AndR8::new(R8::B),
            0xA1 => AndR8::new(R8::C),
            0xA2 => AndR8::new(R8::D),
            0xA3 => AndR8::new(R8::E),
            0xA4 => AndR8::new(R8::H),
            0xA5 => AndR8::new(R8::L),
            0xA6 => AndPointedByHL::new(),
            0xA7 => AndR8::new(R8::A),
            0xA8 => XorR8::new(R8::B),
            0xA9 => XorR8::new(R8::C),
            0xAA => XorR8::new(R8::D),
            0xAB => XorR8::new(R8::E),
            0xAC => XorR8::new(R8::H),
            0xAD => XorR8::new(R8::L),
            0xAE => XorPointedByHL::new(),
            0xAF => XorR8::new(R8::A),
            0xB0 => OrR8::new(R8::B),
            0xB1 => OrR8::new(R8::C),
            0xB2 => OrR8::new(R8::D),
            0xB3 => OrR8::new(R8::E),
            0xB4 => OrR8::new(R8::H),
            0xB5 => OrR8::new(R8::L),
            0xB6 => OrPointedByHL::new(),
            0xB7 => OrR8::new(R8::A),
            0xB8 => CpR8::new(R8::B),
            0xB9 => CpR8::new(R8::C),
            0xBA => CpR8::new(R8::D),
            0xBB => CpR8::new(R8::E),
            0xBC => CpR8::new(R8::H),
            0xBD => CpR8::new(R8::L),
            0xBE => CpPointedByHL::new(),
            0xBF => CpR8::new(R8::A),
            0xC0 => Ret::new(JC::NotZero(cpu.not_zero())),
            0xC1 => Pop::new(R16::BC),
            0xC2 => JpToImm16::new(JC::NotZero(cpu.not_zero()), gb.load(cpu.pc + 1)),
            0xC3 => JpToImm16::new(JC::None, gb.load(cpu.pc + 1)),
            0xC4 => Call::new(JC::NotZero(cpu.not_zero()), gb.load(cpu.pc + 1)),
            0xC5 => Push::new(R16::BC),
            0xC6 => AddImm8::new(gb.read(cpu.pc + 1)),
            0xC7 => Rst::new(0x00),
            0xC8 => Ret::new(JC::Zero(cpu.zero())),
            0xC9 => Ret::new(JC::None),
            0xCA => JpToImm16::new(JC::Zero(cpu.zero()), gb.load(cpu.pc + 1)),
            0xCB => {
                let cb_opcode = gb.read(cpu.pc + 1);
                match Cpu::fetch_cb(gb, cb_opcode) {
                    Ok(instruction) => instruction,
                    Err(e) => return Err(e),
                }
            }
            0xCC => Call::new(JC::Zero(cpu.zero()), gb.load(cpu.pc + 1)),
            0xCD => Call::new(JC::None, gb.load(cpu.pc + 1)),
            0xCE => AdcImm8::new(gb.read(cpu.pc + 1)),
            0xCF => Rst::new(0x08),
            0xD0 => Ret::new(JC::NotCarry(cpu.not_carry())),
            0xD1 => Pop::new(R16::DE),
            0xD2 => JpToImm16::new(JC::NotCarry(cpu.not_carry()), gb.load(cpu.pc + 1)),
            0xD3 => return Err(InstructionError::UnusedOpcode(opcode, cpu.pc)),
            0xD4 => Call::new(JC::NotCarry(cpu.not_carry()), gb.load(cpu.pc + 1)),
            0xD5 => Push::new(R16::DE),
            0xD6 => SubImm8::new(gb.read(cpu.pc + 1)),
            0xD7 => Rst::new(0x10),
            0xD8 => Ret::new(JC::Carry(cpu.carry())),
            0xD9 => Reti::new(),
            0xDA => JpToImm16::new(JC::Carry(cpu.carry()), gb.load(cpu.pc + 1)),
            0xDB => return Err(InstructionError::UnusedOpcode(opcode, cpu.pc)),
            0xDC => Call::new(JC::Carry(cpu.carry()), gb.load(cpu.pc + 1)),
            0xDD => return Err(InstructionError::UnusedOpcode(opcode, cpu.pc)),
            0xDE => SbcImm8::new(gb.read(cpu.pc + 1), cpu.carry()),
            0xDF => Rst::new(0x18),
            0xE0 => LdhImm8A::new(gb.read(cpu.pc + 1)),
            0xE1 => Pop::new(R16::HL),
            0xE2 => LdhCA::new(),
            0xE3 => return Err(InstructionError::UnusedOpcode(opcode, cpu.pc)),
            0xE4 => return Err(InstructionError::UnusedOpcode(opcode, cpu.pc)),
            0xE5 => Push::new(R16::HL),
            0xE6 => AndImm8::new(gb.read(cpu.pc + 1)),
            0xE7 => Rst::new(0x20),
            0xE8 => AddSPImm8::new(gb.read(cpu.pc + 1) as i8),
            0xE9 => JpToHL::new(cpu.hl()),
            0xEA => LdPointedByImm16A::new(gb.load(cpu.pc + 1)),
            0xEB => return Err(InstructionError::UnusedOpcode(opcode, cpu.pc)),
            0xEC => return Err(InstructionError::UnusedOpcode(opcode, cpu.pc)),
            0xED => return Err(InstructionError::UnusedOpcode(opcode, cpu.pc)),
            0xEE => XorImm8::new(gb.read(cpu.pc + 1)),
            0xEF => Rst::new(0x28),
            0xF0 => LdhAImm8::new(gb.read(cpu.pc + 1)),
            0xF1 => Pop::new(R16::AF),
            0xF2 => LdhAC::new(),
            0xF3 => Di::new(),
            0xF4 => return Err(InstructionError::UnusedOpcode(opcode, cpu.pc)),
            0xF5 => Push::new(R16::AF),
            0xF6 => OrImm8::new(gb.read(cpu.pc + 1)),
            0xF7 => Rst::new(0x30),
            0xF8 => LdHLSPPlusImm8::new(gb.read(cpu.pc + 1) as i8),
            0xF9 => LdSPHL::new(),
            0xFA => LdAPointedByImm16::new(gb.load(cpu.pc + 1)),
            0xFB => Ei::new(),
            0xFC => return Err(InstructionError::UnusedOpcode(opcode, cpu.pc)),
            0xFD => return Err(InstructionError::UnusedOpcode(opcode, cpu.pc)),
            0xFE => CpImm8::new(gb.read(cpu.pc + 1)),
            0xFF => Rst::new(0x38),
        };

        Ok(instruction)
    }

    fn fetch_cb(gb: &mut Dmg, cb_opcode: u8) -> FetchResult {
        // used bit in res, set and bit instructions
        let bit = (cb_opcode & 0x38) >> 3;
        let cpu = &gb.cpu;

        let instruction: Box<dyn Instruction> = match cb_opcode {
            0x00 => RlcR8::new(R8::B),
            0x01 => RlcR8::new(R8::C),
            0x02 => RlcR8::new(R8::D),
            0x03 => RlcR8::new(R8::E),
            0x04 => RlcR8::new(R8::H),
            0x05 => RlcR8::new(R8::L),
            0x06 => RlcPointedByHL::new(),
            0x07 => RlcR8::new(R8::A),
            0x08 => RrcR8::new(R8::B),
            0x09 => RrcR8::new(R8::C),
            0x0A => RrcR8::new(R8::D),
            0x0B => RrcR8::new(R8::E),
            0x0C => RrcR8::new(R8::H),
            0x0D => RrcR8::new(R8::L),
            0x0E => RrcPointedByHL::new(),
            0x0F => RrcR8::new(R8::A),
            0x10 => RlR8::new(R8::B),
            0x11 => RlR8::new(R8::C),
            0x12 => RlR8::new(R8::D),
            0x13 => RlR8::new(R8::E),
            0x14 => RlR8::new(R8::H),
            0x15 => RlR8::new(R8::L),
            0x16 => RlPointedByHL::new(),
            0x17 => RlR8::new(R8::A),
            0x18 => RrR8::new(R8::B),
            0x19 => RrR8::new(R8::C),
            0x1A => RrR8::new(R8::D),
            0x1B => RrR8::new(R8::E),
            0x1C => RrR8::new(R8::H),
            0x1D => RrR8::new(R8::L),
            0x1E => RrPointedByHL::new(),
            0x1F => RrR8::new(R8::A),
            0x20 => SlaR8::new(R8::B),
            0x21 => SlaR8::new(R8::C),
            0x22 => SlaR8::new(R8::D),
            0x23 => SlaR8::new(R8::E),
            0x24 => SlaR8::new(R8::H),
            0x25 => SlaR8::new(R8::L),
            0x26 => SlaPointedByHL::new(),
            0x27 => SlaR8::new(R8::A),
            0x28 => SraR8::new(R8::B),
            0x29 => SraR8::new(R8::C),
            0x2A => SraR8::new(R8::D),
            0x2B => SraR8::new(R8::E),
            0x2C => SraR8::new(R8::H),
            0x2D => SraR8::new(R8::L),
            0x2E => SraPointedByHL::new(),
            0x2F => SraR8::new(R8::A),
            0x30 => SwapR8::new(R8::B),
            0x31 => SwapR8::new(R8::C),
            0x32 => SwapR8::new(R8::D),
            0x33 => SwapR8::new(R8::E),
            0x34 => SwapR8::new(R8::H),
            0x35 => SwapR8::new(R8::L),
            0x36 => SwapPointedByHL::new(),
            0x37 => SwapR8::new(R8::A),
            0x38 => SrlR8::new(R8::B),
            0x39 => SrlR8::new(R8::C),
            0x3A => SrlR8::new(R8::D),
            0x3B => SrlR8::new(R8::E),
            0x3C => SrlR8::new(R8::H),
            0x3D => SrlR8::new(R8::L),
            0x3E => SrlPointedByHL::new(),
            0x3F => SrlR8::new(R8::A),
            0x40..=0x7F => match cb_opcode & 0x07 {
                0 => BitR8::new(bit, R8::B),
                1 => BitR8::new(bit, R8::C),
                2 => BitR8::new(bit, R8::D),
                3 => BitR8::new(bit, R8::E),
                4 => BitR8::new(bit, R8::H),
                5 => BitR8::new(bit, R8::L),
                6 => BitPointedByHL::new(bit),
                7 => BitR8::new(bit, R8::A),
                _ => return Err(InstructionError::OutOfRangeCBOpcode(cb_opcode, cpu.pc)),
            },
            0x80..=0xBF => match cb_opcode & 0x07 {
                0 => ResR8::new(bit, R8::B),
                1 => ResR8::new(bit, R8::C),
                2 => ResR8::new(bit, R8::D),
                3 => ResR8::new(bit, R8::E),
                4 => ResR8::new(bit, R8::H),
                5 => ResR8::new(bit, R8::L),
                6 => ResPointedByHL::new(bit),
                7 => ResR8::new(bit, R8::A),
                _ => unreachable!(),
            },
            0xC0..=0xFF => match cb_opcode & 0x07 {
                0 => SetR8::new(bit, R8::B),
                1 => SetR8::new(bit, R8::C),
                2 => SetR8::new(bit, R8::D),
                3 => SetR8::new(bit, R8::E),
                4 => SetR8::new(bit, R8::H),
                5 => SetR8::new(bit, R8::L),
                6 => SetPointedByHL::new(bit),
                7 => SetR8::new(bit, R8::A),
                _ => unreachable!(),
            },
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
