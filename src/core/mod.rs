mod apu;
mod cartrigde;
mod cpu;
mod interrupts;
mod joypad;
mod license;
mod memory;
pub mod ppu;
mod serial;
mod timer;

pub use crate::prelude::*;
use crate::{
    core::{
        apu::{APU_REGISTER_END, APU_REGISTER_START},
        cpu::{Instruction, Len, R8, R16},
        interrupts::IE,
        ppu::{PPU_REGISTER_END, PPU_REGISTER_START},
    },
    utils::with_u16,
};

pub use apu::Apu;
pub use cartrigde::Cartridge;
pub use cpu::{AFTER_BOOT_CPU, Cpu};
pub use interrupts::Interrupt;
pub use joypad::Joypad;
pub use memory::*;
pub use ppu::Ppu;
pub use serial::Serial;
pub use timer::TimerController;

use self::{
    interrupts::IF,
    joypad::JOYP,
    serial::{SB, SC},
    timer::{DIV, TAC, TIMA, TMA},
};

const BANK_REGISTER: u16 = 0xFF50;

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

    /// Modifies the DMG state by executing one CPU instruction, and return the executed instruction
    pub fn run(&mut self) -> Result<()> {
        // one frame (70224 cycles)
        while self.cpu.cycles < 70224 {
            let _instr = self.step()?;
            if self.cpu.pc == 0x0100 {
                println!("just arrive to game rom start");
                println!("Cpu: {}", self.cpu);
                std::thread::sleep(std::time::Duration::from_millis(500));
            }
        }
        self.cpu.cycles = 0;
        self.ppu.last_cycles = 0;

        Ok(())
    }

    pub fn step(&mut self) -> Result<Box<dyn Instruction>> {
        let opcode = self[self.cpu.pc];

        let mut instruction = match Cpu::fetch(self, opcode) {
            Ok(instr) => instr,
            Err(e) => Err(Error::Generic(format!(
                "Error fetching instruction at {:04X}: {}",
                self.cpu.pc, e
            )))?,
        };

        let effect = match instruction.exec(self) {
            Ok(effect) => effect,
            Err(e) => Err(Error::Generic(format!(
                "Error executing instruction at {:04X}: {}",
                self.cpu.pc, e
            )))?,
        };

        let cycles = self.cpu.cycles.wrapping_add(effect.cycles as usize);

        self.cpu.cycles = cycles;
        self.cpu.pc = match effect.len {
            Len::Jump(_) => self.cpu.pc,
            Len::AddLen(len) => self.cpu.pc.wrapping_add(len as u16),
        };
        effect.flags.apply(&mut self.cpu.f);

        // ppu
        Ppu::step(self, cycles);

        // timer
        self.timer.step(cycles);

        Ok(instruction)
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

                APU_REGISTER_START..=APU_REGISTER_END => &self.apu[addr],

                PPU_REGISTER_START..=PPU_REGISTER_END => &self.ppu[addr],

                BANK_REGISTER => &self.bank,

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
            IO_REGISTERS_START..=IO_REGISTERS_END => match addr {
                JOYP => &mut self.joypad.0,

                SB => &mut self.serial.sb,
                SC => &mut self.serial.sc,

                DIV => &mut self.timer.divider,
                TIMA => &mut self.timer.timer_counter,
                TMA => &mut self.timer.timer_modulo,
                TAC => &mut self.timer.timer_control,

                IF => &mut self.interrupt_flag.0,

                0xF100..=0xFF26 => &mut self.apu[addr],

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

impl MemoryMapped for Dmg {
    fn read(&self, address: u16) -> u8 {
        match address {
            ROM_BANK00_START..=ROM_BANKNN_END => self.bus.rom[address as usize],
            VRAM_START..=VRAM_END => self.bus.vram[(address - VRAM_START) as usize],
            EXTERNAL_RAM_START..=EXTERNAL_RAM_END => {
                self.bus.external_ram[(address - EXTERNAL_RAM_START) as usize]
            }
            WRAM_BANK0_START..=WRAM_BANKN_END => self.bus.ram[(address - WRAM_BANK0_START) as usize],
            ECHO_RAM_START..=ECHO_RAM_END => {
                let offset = (address - ECHO_RAM_START) as usize;
                self.bus.ram[offset]
            }
            OAM_START..=OAM_END => self.bus.oam_ram[(address - OAM_START) as usize],

            NOT_USABLE_START..=NOT_USABLE_END => unreachable!(
                "Read to prohibited memory region [{}, {}] with address {:04X}",
                NOT_USABLE_START, NOT_USABLE_END, address
            ),

            IO_REGISTERS_START..=IO_REGISTERS_END => match address {
                JOYP => self.joypad.0,

                SB => self.serial.sb,
                SC => self.serial.sc,

                DIV => self.timer.divider,
                TIMA => self.timer.timer_counter,
                TMA => self.timer.timer_modulo,
                TAC => self.timer.timer_control,

                IF => self.interrupt_flag.0,

                APU_REGISTER_START..=APU_REGISTER_END => self.apu[addr],

                PPU_REGISTER_START..=PPU_REGISTER_END => self.ppu[addr],

                BANK_REGISTER => self.bank,

                _ => unreachable!(
                    "Read of IO register {:04X} should have been handled by other components",
                    address
                ),
            },
            HRAM_START..=HRAM_END => self.bus.hram[(address - HRAM_START) as usize],

            _ => unreachable!("Read of address {address:04X} should have been handled by other components"),
        }
    }

    fn write(&mut self, address: u16, value: u8) {
        match address {
            ROM_BANK00_START..=ROM_BANKNN_END => self.bus.rom[address as usize] = value,
            VRAM_START..=VRAM_END => self.bus.vram[(address - VRAM_START) as usize] = value,
            EXTERNAL_RAM_START..=EXTERNAL_RAM_END => {
                self.bus.external_ram[(address - EXTERNAL_RAM_START) as usize] = value
            }
            WRAM_BANK0_START..=WRAM_BANKN_END => self.bus.ram[(address - WRAM_BANK0_START) as usize] = value,
            ECHO_RAM_START..=ECHO_RAM_END => {
                let offset = (address - ECHO_RAM_START) as usize;
                self.bus.ram[offset] = value;
            }
            OAM_START..=OAM_END => self.bus.oam_ram[(address - OAM_START) as usize] = value,
            HRAM_START..=HRAM_END => self.bus.hram[(address - HRAM_START) as usize] = value,

            _ => unreachable!("Write of address {address:04X} should have been handled by other components"),
        }
    }
}
