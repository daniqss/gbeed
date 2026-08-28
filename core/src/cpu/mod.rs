mod flags;
mod instructions;
mod registers;

use crate::{
    cpu::flags::{CARRY_FLAG_MASK, HALF_CARRY_FLAG_MASK, SUBTRACTION_FLAG_MASK, ZERO_FLAG_MASK},
    dmg::Dmg,
    interrupts::{JOYPAD_INTERRUPT, LCD_STAT_INTERRUPT, SERIAL_INTERRUPT, TIMER_INTERRUPT, VBLANK_INTERRUPT},
    prelude::*,
};

// TODO: not expose individual instructions
pub use instructions::{Instruction, InstructionError, Instructions, Len, Nop};
use instructions::{JumpCondition as JC, *};
pub use registers::{Register8 as R8, Register16 as R16};

use core::fmt::{self, Display, Formatter};

pub type FetchResult = core::result::Result<Instructions, InstructionError>;

pub const FREQUENCY: u32 = 4_194_304;

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
    cycles: 44441,
    ime: false,
    halted: false,
};

/// # CPU
/// Gameboy CPU, with a mix of Intel 8080 and Zilog Z80 features and instruction set, the Sharp SM83.
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
    pub(crate) fn new(start_at_boot: bool) -> Cpu {
        if start_at_boot {
            Cpu::default()
        } else {
            AFTER_BOOT_CPU
        }
    }

    reg16!(pub af, set_af, a, f);
    reg16!(pub bc, set_bc, b, c);
    reg16!(pub de, set_de, d, e);
    reg16!(pub hl, set_hl, h, l);

    flag_methods! {
        pub(crate)
        carry => CARRY_FLAG_MASK,
        zero => ZERO_FLAG_MASK,
        subtraction => SUBTRACTION_FLAG_MASK,
        half_carry => HALF_CARRY_FLAG_MASK,
    }

    pub(crate) fn reset(&mut self) {
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

    #[inline(never)]
    pub(crate) fn step(gb: &mut Dmg) -> Result<Option<Instructions>, InstructionError> {
        // check if is neccessatry to handle interrupts before executing the instruction
        if Cpu::handle_interrupts(gb) {
            // 5 Mcycles = 2 NOP + 3 ...
            gb.cpu.cycles = gb.cpu.cycles.wrapping_add(5);
            return Ok(None);
        }

        if gb.cpu.halted {
            gb.cpu.cycles = gb.cpu.cycles.wrapping_add(4);
            return Ok(None);
        }

        let opcode = gb.read(gb.cpu.pc);

        let mut instruction = Cpu::fetch(gb, opcode)?;
        let effect = instruction.exec(gb)?;

        gb.cpu.cycles = gb.cpu.cycles.wrapping_add(effect.cycles as usize);
        gb.cpu.pc = match effect.len {
            Len::Jump(_) => gb.cpu.pc,
            Len::AddLen(len) => gb.cpu.pc.wrapping_add(len as u16),
        };
        effect.flags.apply(&mut gb.cpu.f);

        Ok(Some(instruction))
    }

    fn handle_interrupts(gb: &mut Dmg) -> bool {
        let enabled_interrupts = gb.interrupt_enable.0 & gb.interrupt_flag.0;

        if enabled_interrupts & 0b0001_1111 == 0 {
            return false;
        }

        if gb.cpu.halted {
            gb.cpu.halted = false;
        }

        if !gb.cpu.ime {
            return false;
        }

        gb.cpu.ime = false;
        if enabled_interrupts & VBLANK_INTERRUPT != 0 {
            Cpu::service_interrupt(gb, 0x40, VBLANK_INTERRUPT);
        } else if enabled_interrupts & LCD_STAT_INTERRUPT != 0 {
            Cpu::service_interrupt(gb, 0x48, LCD_STAT_INTERRUPT);
        } else if enabled_interrupts & TIMER_INTERRUPT != 0 {
            Cpu::service_interrupt(gb, 0x50, TIMER_INTERRUPT);
        } else if enabled_interrupts & SERIAL_INTERRUPT != 0 {
            Cpu::service_interrupt(gb, 0x58, SERIAL_INTERRUPT);
        } else if enabled_interrupts & JOYPAD_INTERRUPT != 0 {
            Cpu::service_interrupt(gb, 0x60, JOYPAD_INTERRUPT);
        }

        true
    }

    fn service_interrupt(gb: &mut Dmg, service_routine_addr: u16, interrupt_mask: u8) {
        let pc = gb.cpu.pc;
        push(gb, pc);

        gb.interrupt_flag.0 &= !interrupt_mask;
        gb.cpu.pc = service_routine_addr;
    }

    /// Execute instruction based on the opcode.
    /// Return a result with the effect of the instruction or an instruction error (e.g unused opcode)
    pub(crate) fn fetch(gb: &mut Dmg, opcode: u8) -> FetchResult {
        let cpu = &gb.cpu;

        let instruction: Instructions = match opcode {
            0x00 => Nop::new().into(),
            0x01 => Ld16::new(R16::BC, gb.load(cpu.pc.wrapping_add(1))).into(),
            0x02 => Ld::new(PointedByR16(R16::BC), R8::A).into(),
            0x03 => Inc16::new(R16::BC).into(),
            0x04 => Inc::new(R8::B).into(),
            0x05 => Dec::new(R8::B).into(),
            0x06 => Ld::new(R8::B, Imm8(gb.read(cpu.pc.wrapping_add(1)))).into(),
            0x07 => Rlca::new().into(),
            0x08 => LdImm16SP::new(gb.load(cpu.pc.wrapping_add(1))).into(),
            0x09 => AddHL::new(R16::BC).into(),
            0x0A => Ld::new(R8::A, PointedByR16(R16::BC)).into(),
            0x0B => Dec16::new(R16::BC).into(),
            0x0C => Inc::new(R8::C).into(),
            0x0D => Dec::new(R8::C).into(),
            0x0E => Ld::new(R8::C, Imm8(gb.read(cpu.pc.wrapping_add(1)))).into(),
            0x0F => Rrca::new().into(),
            0x10 => Stop::new().into(),
            0x11 => Ld16::new(R16::DE, gb.load(cpu.pc.wrapping_add(1))).into(),
            0x12 => Ld::new(PointedByR16(R16::DE), R8::A).into(),
            0x13 => Inc16::new(R16::DE).into(),
            0x14 => Inc::new(R8::D).into(),
            0x15 => Dec::new(R8::D).into(),
            0x16 => Ld::new(R8::D, Imm8(gb.read(cpu.pc.wrapping_add(1)))).into(),
            0x17 => Rla::new(cpu.carry()).into(),
            0x18 => Jr::new(JC::None, gb.read(cpu.pc.wrapping_add(1))).into(),
            0x19 => AddHL::new(R16::DE).into(),
            0x1A => Ld::new(R8::A, PointedByR16(R16::DE)).into(),
            0x1B => Dec16::new(R16::DE).into(),
            0x1C => Inc::new(R8::E).into(),
            0x1D => Dec::new(R8::E).into(),
            0x1E => Ld::new(R8::E, Imm8(gb.read(cpu.pc.wrapping_add(1)))).into(),
            0x1F => Rra::new(cpu.carry()).into(),
            0x20 => Jr::new(JC::NotZero(cpu.not_zero()), gb.read(cpu.pc.wrapping_add(1))).into(),
            0x21 => Ld16::new(R16::HL, gb.load(cpu.pc.wrapping_add(1))).into(),
            0x22 => LdPointedByHLIncA::new().into(),
            0x23 => Inc16::new(R16::HL).into(),
            0x24 => Inc::new(R8::H).into(),
            0x25 => Dec::new(R8::H).into(),
            0x26 => Ld::new(R8::H, Imm8(gb.read(cpu.pc.wrapping_add(1)))).into(),
            0x27 => Daa::new().into(),
            0x28 => Jr::new(JC::Zero(cpu.zero()), gb.read(cpu.pc.wrapping_add(1))).into(),
            0x29 => AddHL::new(R16::HL).into(),
            0x2A => LdAPointedByHLInc::new().into(),
            0x2b => Dec16::new(R16::HL).into(),
            0x2C => Inc::new(R8::L).into(),
            0x2D => Dec::new(R8::L).into(),
            0x2E => Ld::new(R8::L, Imm8(gb.read(cpu.pc.wrapping_add(1)))).into(),
            0x2F => Cpl::new().into(),
            0x30 => Jr::new(JC::NotCarry(cpu.not_carry()), gb.read(cpu.pc.wrapping_add(1))).into(),
            0x31 => Ld16::new(StackPointer, gb.load(cpu.pc.wrapping_add(1))).into(),
            0x32 => LdPointedByHLDecA::new().into(),
            0x33 => Inc16::new(StackPointer).into(),
            0x34 => Inc::new(PointedByHL).into(),
            0x35 => Dec::new(PointedByHL).into(),
            0x36 => Ld::new(PointedByHL, Imm8(gb.read(cpu.pc.wrapping_add(1)))).into(),
            0x37 => Scf::new().into(),
            0x38 => Jr::new(JC::Carry(cpu.carry()), gb.read(cpu.pc.wrapping_add(1))).into(),
            0x39 => AddHL::new(StackPointer).into(),
            0x3A => LdAPointedByHLDec::new().into(),
            0x3B => Dec16::new(StackPointer).into(),
            0x3C => Inc::new(R8::A).into(),
            0x3D => Dec::new(R8::A).into(),
            0x3E => Ld::new(R8::A, Imm8(gb.read(cpu.pc.wrapping_add(1)))).into(),
            0x3F => Ccf::new(cpu.carry()).into(),
            0x40 => Ld::new(R8::B, R8::B).into(),
            0x41 => Ld::new(R8::B, R8::C).into(),
            0x42 => Ld::new(R8::B, R8::D).into(),
            0x43 => Ld::new(R8::B, R8::E).into(),
            0x44 => Ld::new(R8::B, R8::H).into(),
            0x45 => Ld::new(R8::B, R8::L).into(),
            0x46 => Ld::new(R8::B, PointedByHL).into(),
            0x47 => Ld::new(R8::B, R8::A).into(),
            0x48 => Ld::new(R8::C, R8::B).into(),
            0x49 => Ld::new(R8::C, R8::C).into(),
            0x4A => Ld::new(R8::C, R8::D).into(),
            0x4B => Ld::new(R8::C, R8::E).into(),
            0x4C => Ld::new(R8::C, R8::H).into(),
            0x4D => Ld::new(R8::C, R8::L).into(),
            0x4E => Ld::new(R8::C, PointedByHL).into(),
            0x4F => Ld::new(R8::C, R8::A).into(),
            0x50 => Ld::new(R8::D, R8::B).into(),
            0x51 => Ld::new(R8::D, R8::C).into(),
            0x52 => Ld::new(R8::D, R8::D).into(),
            0x53 => Ld::new(R8::D, R8::E).into(),
            0x54 => Ld::new(R8::D, R8::H).into(),
            0x55 => Ld::new(R8::D, R8::L).into(),
            0x56 => Ld::new(R8::D, PointedByHL).into(),
            0x57 => Ld::new(R8::D, R8::A).into(),
            0x58 => Ld::new(R8::E, R8::B).into(),
            0x59 => Ld::new(R8::E, R8::C).into(),
            0x5A => Ld::new(R8::E, R8::D).into(),
            0x5B => Ld::new(R8::E, R8::E).into(),
            0x5C => Ld::new(R8::E, R8::H).into(),
            0x5D => Ld::new(R8::E, R8::L).into(),
            0x5E => Ld::new(R8::E, PointedByHL).into(),
            0x5F => Ld::new(R8::E, R8::A).into(),
            0x60 => Ld::new(R8::H, R8::B).into(),
            0x61 => Ld::new(R8::H, R8::C).into(),
            0x62 => Ld::new(R8::H, R8::D).into(),
            0x63 => Ld::new(R8::H, R8::E).into(),
            0x64 => Ld::new(R8::H, R8::H).into(),
            0x65 => Ld::new(R8::H, R8::L).into(),
            0x66 => Ld::new(R8::H, PointedByHL).into(),
            0x67 => Ld::new(R8::H, R8::A).into(),
            0x68 => Ld::new(R8::L, R8::B).into(),
            0x69 => Ld::new(R8::L, R8::C).into(),
            0x6A => Ld::new(R8::L, R8::D).into(),
            0x6B => Ld::new(R8::L, R8::E).into(),
            0x6C => Ld::new(R8::L, R8::H).into(),
            0x6D => Ld::new(R8::L, R8::L).into(),
            0x6E => Ld::new(R8::L, PointedByHL).into(),
            0x6F => Ld::new(R8::L, R8::A).into(),
            0x70 => Ld::new(PointedByHL, R8::B).into(),
            0x71 => Ld::new(PointedByHL, R8::C).into(),
            0x72 => Ld::new(PointedByHL, R8::D).into(),
            0x73 => Ld::new(PointedByHL, R8::E).into(),
            0x74 => Ld::new(PointedByHL, R8::H).into(),
            0x75 => Ld::new(PointedByHL, R8::L).into(),
            0x76 => Halt::new().into(),
            0x77 => Ld::new(PointedByHL, R8::A).into(),
            0x78 => Ld::new(R8::A, R8::B).into(),
            0x79 => Ld::new(R8::A, R8::C).into(),
            0x7A => Ld::new(R8::A, R8::D).into(),
            0x7B => Ld::new(R8::A, R8::E).into(),
            0x7C => Ld::new(R8::A, R8::H).into(),
            0x7D => Ld::new(R8::A, R8::L).into(),
            0x7E => Ld::new(R8::A, PointedByHL).into(),
            0x7F => Ld::new(R8::A, R8::A).into(),
            0x80 => AddA::new(R8::B).into(),
            0x81 => AddA::new(R8::C).into(),
            0x82 => AddA::new(R8::D).into(),
            0x83 => AddA::new(R8::E).into(),
            0x84 => AddA::new(R8::H).into(),
            0x85 => AddA::new(R8::L).into(),
            0x86 => AddA::new(PointedByHL).into(),
            0x87 => AddA::new(R8::A).into(),
            0x88 => Adc::new(R8::B).into(),
            0x89 => Adc::new(R8::C).into(),
            0x8A => Adc::new(R8::D).into(),
            0x8B => Adc::new(R8::E).into(),
            0x8C => Adc::new(R8::H).into(),
            0x8D => Adc::new(R8::L).into(),
            0x8E => Adc::new(PointedByHL).into(),
            0x8F => Adc::new(R8::A).into(),
            0x90 => Sub::new(R8::B).into(),
            0x91 => Sub::new(R8::C).into(),
            0x92 => Sub::new(R8::D).into(),
            0x93 => Sub::new(R8::E).into(),
            0x94 => Sub::new(R8::H).into(),
            0x95 => Sub::new(R8::L).into(),
            0x96 => Sub::new(PointedByHL).into(),
            0x97 => Sub::new(R8::A).into(),
            0x98 => Sbc::new(R8::B).into(),
            0x99 => Sbc::new(R8::C).into(),
            0x9A => Sbc::new(R8::D).into(),
            0x9B => Sbc::new(R8::E).into(),
            0x9C => Sbc::new(R8::H).into(),
            0x9D => Sbc::new(R8::L).into(),
            0x9E => Sbc::new(PointedByHL).into(),
            0x9F => Sbc::new(R8::A).into(),
            0xA0 => And::new(R8::B).into(),
            0xA1 => And::new(R8::C).into(),
            0xA2 => And::new(R8::D).into(),
            0xA3 => And::new(R8::E).into(),
            0xA4 => And::new(R8::H).into(),
            0xA5 => And::new(R8::L).into(),
            0xA6 => And::new(PointedByHL).into(),
            0xA7 => And::new(R8::A).into(),
            0xA8 => Xor::new(R8::B).into(),
            0xA9 => Xor::new(R8::C).into(),
            0xAA => Xor::new(R8::D).into(),
            0xAB => Xor::new(R8::E).into(),
            0xAC => Xor::new(R8::H).into(),
            0xAD => Xor::new(R8::L).into(),
            0xAE => Xor::new(PointedByHL).into(),
            0xAF => Xor::new(R8::A).into(),
            0xB0 => Or::new(R8::B).into(),
            0xB1 => Or::new(R8::C).into(),
            0xB2 => Or::new(R8::D).into(),
            0xB3 => Or::new(R8::E).into(),
            0xB4 => Or::new(R8::H).into(),
            0xB5 => Or::new(R8::L).into(),
            0xB6 => Or::new(PointedByHL).into(),
            0xB7 => Or::new(R8::A).into(),
            0xB8 => Cp::new(R8::B).into(),
            0xB9 => Cp::new(R8::C).into(),
            0xBA => Cp::new(R8::D).into(),
            0xBB => Cp::new(R8::E).into(),
            0xBC => Cp::new(R8::H).into(),
            0xBD => Cp::new(R8::L).into(),
            0xBE => Cp::new(PointedByHL).into(),
            0xBF => Cp::new(R8::A).into(),
            0xC0 => Ret::new(JC::NotZero(cpu.not_zero())).into(),
            0xC1 => Pop::new(R16::BC).into(),
            0xC2 => JpToImm16::new(JC::NotZero(cpu.not_zero()), gb.load(cpu.pc.wrapping_add(1))).into(),
            0xC3 => JpToImm16::new(JC::None, gb.load(cpu.pc.wrapping_add(1))).into(),
            0xC4 => Call::new(JC::NotZero(cpu.not_zero()), gb.load(cpu.pc.wrapping_add(1))).into(),
            0xC5 => Push::new(R16::BC).into(),
            0xC6 => AddA::new(Imm8(gb.read(cpu.pc.wrapping_add(1)))).into(),
            0xC7 => Rst::new(0x00).into(),
            0xC8 => Ret::new(JC::Zero(cpu.zero())).into(),
            0xC9 => Ret::new(JC::None).into(),
            0xCA => JpToImm16::new(JC::Zero(cpu.zero()), gb.load(cpu.pc.wrapping_add(1))).into(),
            0xCB => {
                let cb_opcode = gb.read(cpu.pc.wrapping_add(1));
                Cpu::fetch_cb(gb, cb_opcode)?
            }
            0xCC => Call::new(JC::Zero(cpu.zero()), gb.load(cpu.pc.wrapping_add(1))).into(),
            0xCD => Call::new(JC::None, gb.load(cpu.pc.wrapping_add(1))).into(),
            0xCE => Adc::new(Imm8(gb.read(cpu.pc.wrapping_add(1)))).into(),
            0xCF => Rst::new(0x08).into(),
            0xD0 => Ret::new(JC::NotCarry(cpu.not_carry())).into(),
            0xD1 => Pop::new(R16::DE).into(),
            0xD2 => JpToImm16::new(JC::NotCarry(cpu.not_carry()), gb.load(cpu.pc.wrapping_add(1))).into(),
            0xD3 => return Err(InstructionError::UnusedOpcode(opcode, cpu.pc)),
            0xD4 => Call::new(JC::NotCarry(cpu.not_carry()), gb.load(cpu.pc.wrapping_add(1))).into(),
            0xD5 => Push::new(R16::DE).into(),
            0xD6 => Sub::new(Imm8(gb.read(cpu.pc.wrapping_add(1)))).into(),
            0xD7 => Rst::new(0x10).into(),
            0xD8 => Ret::new(JC::Carry(cpu.carry())).into(),
            0xD9 => Reti::new().into(),
            0xDA => JpToImm16::new(JC::Carry(cpu.carry()), gb.load(cpu.pc.wrapping_add(1))).into(),
            0xDB => return Err(InstructionError::UnusedOpcode(opcode, cpu.pc)),
            0xDC => Call::new(JC::Carry(cpu.carry()), gb.load(cpu.pc.wrapping_add(1))).into(),
            0xDD => return Err(InstructionError::UnusedOpcode(opcode, cpu.pc)),
            0xDE => Sbc::new(Imm8(gb.read(cpu.pc.wrapping_add(1)))).into(),
            0xDF => Rst::new(0x18).into(),
            0xE0 => Ldh::new(PointedByHighImm8(gb.read(cpu.pc.wrapping_add(1))), R8::A).into(),
            0xE1 => Pop::new(R16::HL).into(),
            0xE2 => Ldh::new(PointedByC, R8::A).into(),
            0xE3 => return Err(InstructionError::UnusedOpcode(opcode, cpu.pc)),
            0xE4 => return Err(InstructionError::UnusedOpcode(opcode, cpu.pc)),
            0xE5 => Push::new(R16::HL).into(),
            0xE6 => And::new(Imm8(gb.read(cpu.pc.wrapping_add(1)))).into(),
            0xE7 => Rst::new(0x20).into(),
            0xE8 => AddSPImm8::new(gb.read(cpu.pc.wrapping_add(1)) as i8).into(),
            0xE9 => JpToHL::new(cpu.hl()).into(),
            0xEA => Ld::new(PointedByImm16(gb.load(cpu.pc.wrapping_add(1))), R8::A).into(),
            0xEB => return Err(InstructionError::UnusedOpcode(opcode, cpu.pc)),
            0xEC => return Err(InstructionError::UnusedOpcode(opcode, cpu.pc)),
            0xED => return Err(InstructionError::UnusedOpcode(opcode, cpu.pc)),
            0xEE => Xor::new(Imm8(gb.read(cpu.pc.wrapping_add(1)))).into(),
            0xEF => Rst::new(0x28).into(),
            0xF0 => Ldh::new(R8::A, PointedByHighImm8(gb.read(cpu.pc.wrapping_add(1)))).into(),
            0xF1 => Pop::new(R16::AF).into(),
            0xF2 => Ldh::new(R8::A, PointedByC).into(),
            0xF3 => Di::new().into(),
            0xF4 => return Err(InstructionError::UnusedOpcode(opcode, cpu.pc)),
            0xF5 => Push::new(R16::AF).into(),
            0xF6 => Or::new(Imm8(gb.read(cpu.pc.wrapping_add(1)))).into(),
            0xF7 => Rst::new(0x30).into(),
            0xF8 => LdHLSPPlusImm8::new(gb.read(cpu.pc.wrapping_add(1)) as i8).into(),
            0xF9 => LdSPHL::new().into(),
            0xFA => Ld::new(R8::A, PointedByImm16(gb.load(cpu.pc.wrapping_add(1)))).into(),
            0xFB => Ei::new().into(),
            0xFC => return Err(InstructionError::UnusedOpcode(opcode, cpu.pc)),
            0xFD => return Err(InstructionError::UnusedOpcode(opcode, cpu.pc)),
            0xFE => Cp::new(Imm8(gb.read(cpu.pc.wrapping_add(1)))).into(),
            0xFF => Rst::new(0x38).into(),
        };

        Ok(instruction)
    }

    fn fetch_cb(gb: &mut Dmg, cb_opcode: u8) -> FetchResult {
        // used bit in res, set and bit instructions
        let bit = (cb_opcode & 0x38) >> 3;
        let cpu = &gb.cpu;

        let instruction: Instructions = match cb_opcode {
            0x00 => Rlc::new(R8::B).into(),
            0x01 => Rlc::new(R8::C).into(),
            0x02 => Rlc::new(R8::D).into(),
            0x03 => Rlc::new(R8::E).into(),
            0x04 => Rlc::new(R8::H).into(),
            0x05 => Rlc::new(R8::L).into(),
            0x06 => Rlc::new(PointedByHL).into(),
            0x07 => Rlc::new(R8::A).into(),
            0x08 => Rrc::new(R8::B).into(),
            0x09 => Rrc::new(R8::C).into(),
            0x0A => Rrc::new(R8::D).into(),
            0x0B => Rrc::new(R8::E).into(),
            0x0C => Rrc::new(R8::H).into(),
            0x0D => Rrc::new(R8::L).into(),
            0x0E => Rrc::new(PointedByHL).into(),
            0x0F => Rrc::new(R8::A).into(),
            0x10 => Rl::new(R8::B).into(),
            0x11 => Rl::new(R8::C).into(),
            0x12 => Rl::new(R8::D).into(),
            0x13 => Rl::new(R8::E).into(),
            0x14 => Rl::new(R8::H).into(),
            0x15 => Rl::new(R8::L).into(),
            0x16 => Rl::new(PointedByHL).into(),
            0x17 => Rl::new(R8::A).into(),
            0x18 => Rr::new(R8::B).into(),
            0x19 => Rr::new(R8::C).into(),
            0x1A => Rr::new(R8::D).into(),
            0x1B => Rr::new(R8::E).into(),
            0x1C => Rr::new(R8::H).into(),
            0x1D => Rr::new(R8::L).into(),
            0x1E => Rr::new(PointedByHL).into(),
            0x1F => Rr::new(R8::A).into(),
            0x20 => Sla::new(R8::B).into(),
            0x21 => Sla::new(R8::C).into(),
            0x22 => Sla::new(R8::D).into(),
            0x23 => Sla::new(R8::E).into(),
            0x24 => Sla::new(R8::H).into(),
            0x25 => Sla::new(R8::L).into(),
            0x26 => Sla::new(PointedByHL).into(),
            0x27 => Sla::new(R8::A).into(),
            0x28 => Sra::new(R8::B).into(),
            0x29 => Sra::new(R8::C).into(),
            0x2A => Sra::new(R8::D).into(),
            0x2B => Sra::new(R8::E).into(),
            0x2C => Sra::new(R8::H).into(),
            0x2D => Sra::new(R8::L).into(),
            0x2E => Sra::new(PointedByHL).into(),
            0x2F => Sra::new(R8::A).into(),
            0x30 => Swap::new(R8::B).into(),
            0x31 => Swap::new(R8::C).into(),
            0x32 => Swap::new(R8::D).into(),
            0x33 => Swap::new(R8::E).into(),
            0x34 => Swap::new(R8::H).into(),
            0x35 => Swap::new(R8::L).into(),
            0x36 => Swap::new(PointedByHL).into(),
            0x37 => Swap::new(R8::A).into(),
            0x38 => Srl::new(R8::B).into(),
            0x39 => Srl::new(R8::C).into(),
            0x3A => Srl::new(R8::D).into(),
            0x3B => Srl::new(R8::E).into(),
            0x3C => Srl::new(R8::H).into(),
            0x3D => Srl::new(R8::L).into(),
            0x3E => Srl::new(PointedByHL).into(),
            0x3F => Srl::new(R8::A).into(),
            0x40..=0x7F => match cb_opcode & 0x07 {
                0 => Bit::new(bit, R8::B).into(),
                1 => Bit::new(bit, R8::C).into(),
                2 => Bit::new(bit, R8::D).into(),
                3 => Bit::new(bit, R8::E).into(),
                4 => Bit::new(bit, R8::H).into(),
                5 => Bit::new(bit, R8::L).into(),
                6 => Bit::new(bit, PointedByHL).into(),
                7 => Bit::new(bit, R8::A).into(),
                _ => return Err(InstructionError::OutOfRangeCBOpcode(cb_opcode, cpu.pc)),
            },
            0x80..=0xBF => match cb_opcode & 0x07 {
                0 => Res::new(bit, R8::B).into(),
                1 => Res::new(bit, R8::C).into(),
                2 => Res::new(bit, R8::D).into(),
                3 => Res::new(bit, R8::E).into(),
                4 => Res::new(bit, R8::H).into(),
                5 => Res::new(bit, R8::L).into(),
                6 => Res::new(bit, PointedByHL).into(),
                7 => Res::new(bit, R8::A).into(),
                _ => unreachable!(),
            },
            0xC0..=0xFF => match cb_opcode & 0x07 {
                0 => Set::new(bit, R8::B).into(),
                1 => Set::new(bit, R8::C).into(),
                2 => Set::new(bit, R8::D).into(),
                3 => Set::new(bit, R8::E).into(),
                4 => Set::new(bit, R8::H).into(),
                5 => Set::new(bit, R8::L).into(),
                6 => Set::new(bit, PointedByHL).into(),
                7 => Set::new(bit, R8::A).into(),
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
