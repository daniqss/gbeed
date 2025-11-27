mod apu;
mod cartrigde;
mod cpu;
mod interrupts;
mod joypad;
mod license;
mod memory;
mod ppu;
mod serial;
mod timer;

pub use crate::prelude::*;
use crate::{
    core::{
        cpu::{R8, R16},
        interrupts::IE,
        memory::Accessable,
    },
    utils::with_u16,
};

pub use apu::Apu;
pub use cartrigde::Cartridge;
pub use cpu::Cpu;
pub use interrupts::Interrupt;
pub use joypad::Joypad;
pub use memory::{
    HRAM_END, HRAM_START, IO_REGISTERS_END, IO_REGISTERS_START, Memory, NOT_USABLE_END, ROM_BANK00_START,
    ROM_BANKNN_END,
};
pub use ppu::Ppu;
pub use serial::Serial;
pub use timer::TimerController;

use self::{
    interrupts::IF,
    joypad::JOYP,
    serial::{SB, SC},
    timer::{DIV, TAC, TIMA, TMA},
};

#[derive(Debug, Default)]
pub struct Dmg {
    pub bus: Memory,
    pub cpu: Cpu,
    pub ppu: Ppu,
    pub joypad: Joypad,
    pub serial: Serial,
    pub timer: TimerController,
    pub apu: Apu,
    pub interrupt_flag: Interrupt,
    pub interrupt_enable: Interrupt,
    pub bank: u8,
}

impl Dmg {
    pub fn new(game: Option<Cartridge>, boot_rom: Option<Vec<u8>>) -> Dmg {
        let joypad = Joypad::default();
        let serial = Serial::new();
        let timer = TimerController::new();
        let apu = Apu::new();
        let interrupt_flag = Interrupt::new();
        let ppu = Ppu::new();
        let interrupt_enable = Interrupt::new();

        let start_at_boot = boot_rom.is_some();
        let bus = Memory::new(game, boot_rom);

        Dmg {
            cpu: Cpu::new(start_at_boot),
            ppu,
            joypad,
            serial,
            bus,
            timer,
            apu,
            interrupt_flag,
            interrupt_enable,
            bank: 0,
        }
    }

    pub fn reset(&mut self) { self.cpu.reset(); }

    pub fn run(&mut self) {
        let opcode = self.bus[self.cpu.pc];

        let mut instruction = match Cpu::fetch(self, opcode) {
            Ok(instr) => instr,
            Err(e) => {
                eprintln!("Error fetching instruction: {}", e);
                return;
            }
        };

        #[cfg(debug_assertions)]
        {
            let writer = &mut String::new();
            match instruction.disassembly(writer) {
                Ok(_) => println!("{}", writer),
                Err(e) => {
                    eprintln!("Error disassembling instruction: {}", e);
                }
            }
        }

        let (cycles, len, flags) = match instruction.exec(self) {
            Ok(effect) => (effect.cycles as usize, effect.len as u16, effect.flags),
            Err(e) => {
                eprintln!("Error executing instruction: {}", e);
                return;
            }
        };

        // explicitly drop the instruction to release borrow references
        // after actually making changes to the CPU state and return the effect
        // maybe in the future we must implement a better way to handle this
        // using reference counting for registers I guess
        // which would be ez, because we already have registers enums
        // so changing that to a type/struct that holds Rc<RefCell<u8>> would be easy
        drop(instruction);

        self.cpu.cycles = self.cpu.cycles.wrapping_add(cycles);
        self.cpu.pc = self.cpu.pc.wrapping_add(len);
        flags.apply(&mut self.cpu.f);

        #[cfg(debug_assertions)]
        {
            println!("{}", self.cpu);
            println!("{}", self.cpu.cycles);
        }
    }
}

impl Index<u16> for Dmg {
    type Output = u8;

    fn index(&self, addr: u16) -> &Self::Output {
        match addr {
            ROM_BANK00_START..=NOT_USABLE_END => &self.bus[addr],
            IO_REGISTERS_START..=IO_REGISTERS_END => match addr {
                JOYP => &self.joypad.0,

                SB => &self.serial.sb,
                SC => &self.serial.sc,

                DIV => &self.timer.divider,
                TIMA => &self.timer.timer_counter,
                TMA => &self.timer.timer_modulo,
                TAC => &self.timer.timer_control,

                IF => &self.interrupt_flag.0,

                0xF100..=0xF1FF => &self.apu[addr],

                0xFF40..=0xFF4B => &self.ppu[addr],

                0xFF50 => &self.bank,

                _ => unreachable!(),
            },
            HRAM_START..=HRAM_END => &self.bus[addr],
            IE => &self.interrupt_enable.0,
        }
    }
}

impl IndexMut<u16> for Dmg {
    fn index_mut(&mut self, addr: u16) -> &mut Self::Output {
        match addr {
            ROM_BANK00_START..=NOT_USABLE_END => &mut self.bus[addr],
            IO_REGISTERS_START..=IO_REGISTERS_END => match addr {
                JOYP => &mut self.joypad.0,

                SB => &mut self.serial.sb,
                SC => &mut self.serial.sc,

                DIV => &mut self.timer.divider,
                TIMA => &mut self.timer.timer_counter,
                TMA => &mut self.timer.timer_modulo,
                TAC => &mut self.timer.timer_control,

                IF => &mut self.interrupt_flag.0,

                0xF100..=0xF1FF => &mut self.apu[addr],

                0xFF40..=0xFF4B => &mut self.ppu[addr],

                0xFF50 => {
                    if self.bank == 0 {
                        self.bus.unmap_boot_rom();
                    }

                    &mut self.bank
                }

                _ => unreachable!(),
            },
            HRAM_START..=HRAM_END => &mut self.bus[addr],
            IE => &mut self.interrupt_enable.0,
        }
    }
}

impl Accessable<u16, u16> for Dmg {
    fn read16(&self, addr: u16) -> u16 { utils::to_u16(self[addr], self[addr + 1]) }

    fn write16(&mut self, addr: u16, value: u16) {
        let (low, high) = utils::to_u8(value);
        self[addr] = low;
        self[addr + 1] = high;
    }
}

impl Index<&R8> for Dmg {
    type Output = u8;

    fn index(&self, reg: &R8) -> &Self::Output {
        match reg {
            R8::A => &self.cpu.a,
            R8::F => &self.cpu.f,
            R8::B => &self.cpu.b,
            R8::C => &self.cpu.c,
            R8::D => &self.cpu.d,
            R8::E => &self.cpu.e,
            R8::H => &self.cpu.h,
            R8::L => &self.cpu.l,
        }
    }
}

impl IndexMut<&R8> for Dmg {
    fn index_mut(&mut self, reg: &R8) -> &mut Self::Output {
        match reg {
            R8::A => &mut self.cpu.a,
            R8::F => &mut self.cpu.f,
            R8::B => &mut self.cpu.b,
            R8::C => &mut self.cpu.c,
            R8::D => &mut self.cpu.d,
            R8::E => &mut self.cpu.e,
            R8::H => &mut self.cpu.h,
            R8::L => &mut self.cpu.l,
        }
    }
}

impl Accessable<&R8, &R16> for Dmg {
    fn read16(&self, reg: &R16) -> u16 {
        match reg {
            R16::AF => self.cpu.af(),
            R16::BC => self.cpu.bc(),
            R16::DE => self.cpu.de(),
            R16::HL => self.cpu.hl(),
        }
    }

    fn write16(&mut self, reg: &R16, value: u16) {
        match reg {
            R16::AF => with_u16(&mut self.cpu.f, &mut self.cpu.a, |_| value),
            R16::BC => with_u16(&mut self.cpu.c, &mut self.cpu.b, |_| value),
            R16::DE => with_u16(&mut self.cpu.e, &mut self.cpu.d, |_| value),
            R16::HL => with_u16(&mut self.cpu.l, &mut self.cpu.h, |_| value),
        };
    }
}
