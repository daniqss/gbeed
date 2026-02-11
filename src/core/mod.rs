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
        interrupts::{
            IE, JOYPAD_INTERRUPT, LCD_STAT_INTERRUPT, SERIAL_INTERRUPT, TIMER_INTERRUPT, VBLANK_INTERRUPT,
        },
        ppu::{DMA_REGISTER, PPU_REGISTER_END, PPU_REGISTER_START},
        serial::{SERIAL_REGISTER_END, SERIAL_REGISTER_START},
        timer::{TIMER_REGISTER_END, TIMER_REGISTER_START},
    },
    utils::{high, low, to_u16},
};

pub use apu::Apu;
pub use cartrigde::Cartridge;
pub use cpu::{AFTER_BOOT_CPU, Cpu};
pub use interrupts::Interrupt;
pub use joypad::Joypad;
pub use joypad::JoypadButton;
pub use memory::*;
pub use ppu::Ppu;
pub use serial::Serial;
pub use timer::Timer;

use self::{interrupts::IF, joypad::JOYP};

const BANK_REGISTER: u16 = 0xFF50;

#[derive(Debug, Default)]
pub struct Dmg {
    pub bus: Memory,
    pub cpu: Cpu,
    pub ppu: Ppu,
    pub joypad: Joypad,
    pub serial: Serial,
    pub timer: Timer,
    pub apu: Apu,
    pub interrupt_flag: Interrupt,
    pub interrupt_enable: Interrupt,
    pub bank: u8,
}

impl Dmg {
    pub fn new(game: Option<Cartridge>, boot_rom: Option<Vec<u8>>) -> Dmg {
        let joypad = Joypad::default();
        let serial = Serial::new();
        let timer = Timer::new();
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
        while self.cpu.cycles < 17556 {
            let _instr = self.step()?;

            // println!(
            //     "Executing instruction at {:04X} and {}: {}",
            //     self.cpu.pc,
            //     self.cpu.cycles,
            //     _instr.disassembly()
            // );
        }

        self.cpu.cycles = 0;
        self.ppu.last_cycles = 0;

        Ok(())
    }

    pub fn step(&mut self) -> Result<Box<dyn Instruction>> {
        let start_cycles = self.cpu.cycles;

        // check if is neccessatry to handle interrupts before executing the instruction
        if self.cpu.ime || self.cpu.halted {
            if self.handle_interrupts() {
                self.cpu.ime = false;
                self.cpu.halted = false;

                self.cpu.cycles = self.cpu.cycles.wrapping_add(20);
            }
        }

        // cpu
        let opcode = self.read(self.cpu.pc);

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
        Ppu::step(self, cycles * 4);

        // timer
        let delta_cycles = cycles - start_cycles;
        self.timer.step(delta_cycles * 4);

        // serial
        self.serial.step(&mut self.interrupt_flag);

        Ok(instruction)
    }

    fn handle_interrupts(&mut self) -> bool {
        let enabled_interrupts = self.interrupt_enable.0 & self.interrupt_flag.0;

        if enabled_interrupts & 0b00011111 == 0 {
            if self.cpu.halted && !self.cpu.ime {
                self.cpu.halted = false;
            }
            return false;
        }

        if enabled_interrupts & VBLANK_INTERRUPT != 0 {
            Cpu::service_interrupt(self, 0x40, VBLANK_INTERRUPT);
        } else if enabled_interrupts & LCD_STAT_INTERRUPT != 0 {
            Cpu::service_interrupt(self, 0x48, LCD_STAT_INTERRUPT);
        } else if enabled_interrupts & TIMER_INTERRUPT != 0 {
            Cpu::service_interrupt(self, 0x50, TIMER_INTERRUPT);
        } else if enabled_interrupts & SERIAL_INTERRUPT != 0 {
            Cpu::service_interrupt(self, 0x58, SERIAL_INTERRUPT);
        } else if enabled_interrupts & JOYPAD_INTERRUPT != 0 {
            Cpu::service_interrupt(self, 0x60, JOYPAD_INTERRUPT);
        }

        true
    }
}

impl Accessible<u16> for Dmg {
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

            NOT_USABLE_START..=NOT_USABLE_END => {
                eprintln!(
                    "Reads to prohibited memory region [{}, {}] with address {:04X} return 0xFF",
                    NOT_USABLE_START, NOT_USABLE_END, address
                );
                0xFF
            }

            IO_REGISTERS_START..=IO_REGISTERS_END => match address {
                JOYP => self.joypad.read(address),
                SERIAL_REGISTER_START..=SERIAL_REGISTER_END => self.serial.read(address),
                TIMER_REGISTER_START..=TIMER_REGISTER_END => self.timer.read(address),

                IF => self.interrupt_flag.0,

                APU_REGISTER_START..=APU_REGISTER_END => self.apu.read(address),
                PPU_REGISTER_START..=PPU_REGISTER_END => self.ppu.read(address),

                BANK_REGISTER => self.bank,

                _ => {
                    eprintln!("Reads to unimplemented IO register {:04X} return 0xFF", address);
                    0xFF
                }
            },
            HRAM_START..=HRAM_END => self.bus.hram[(address - HRAM_START) as usize],
            IE => self.interrupt_enable.0,
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

            NOT_USABLE_START..=NOT_USABLE_END => eprintln!(
                "Writes to prohibited memory region [{}, {}] with address {:04X} are ignored",
                NOT_USABLE_START, NOT_USABLE_END, address
            ),

            IO_REGISTERS_START..=IO_REGISTERS_END => match address {
                JOYP => self.joypad.write(address, value),
                SERIAL_REGISTER_START..=SERIAL_REGISTER_END => self.serial.write(address, value),
                TIMER_REGISTER_START..=TIMER_REGISTER_END => self.timer.write(address, value),

                IF => self.interrupt_flag.0 = value,

                APU_REGISTER_START..=APU_REGISTER_END => self.apu.write(address, value),

                DMA_REGISTER => Ppu::dma_transfer(self, value),
                PPU_REGISTER_START..=PPU_REGISTER_END => self.ppu.write(address, value),

                BANK_REGISTER => {
                    if self.bank == 0 {
                        self.bus.unmap_boot_rom();
                    }

                    self.bank = value;
                }

                _ => eprintln!("Writes to unimplemented IO register {:04X} are ignored", address),
            },
            HRAM_START..=HRAM_END => self.bus.hram[(address - HRAM_START) as usize] = value,
            IE => self.interrupt_enable.0 = value,
        }
    }
}

impl Accessible16<u16, u16> for Dmg {
    fn load(&self, address: u16) -> u16 { to_u16(self.read(address), self.read(address.wrapping_add(1))) }

    fn store(&mut self, address: u16, value: u16) {
        self.write(address, low(value));
        self.write(address.wrapping_add(1), high(value));
    }
}

impl Accessible<R8> for Dmg {
    fn read(&self, address: R8) -> u8 {
        match address {
            R8::A => self.cpu.a,
            R8::F => self.cpu.f,
            R8::B => self.cpu.b,
            R8::C => self.cpu.c,
            R8::D => self.cpu.d,
            R8::E => self.cpu.e,
            R8::H => self.cpu.h,
            R8::L => self.cpu.l,
        }
    }

    fn write(&mut self, address: R8, value: u8) {
        match address {
            R8::A => self.cpu.a = value,
            R8::F => self.cpu.f = value & 0xF0,
            R8::B => self.cpu.b = value,
            R8::C => self.cpu.c = value,
            R8::D => self.cpu.d = value,
            R8::E => self.cpu.e = value,
            R8::H => self.cpu.h = value,
            R8::L => self.cpu.l = value,
        }
    }
}

impl Accessible16<R16, R8> for Dmg {
    fn load(&self, address: R16) -> u16 {
        match address {
            R16::AF => self.cpu.af(),
            R16::BC => self.cpu.bc(),
            R16::DE => self.cpu.de(),
            R16::HL => self.cpu.hl(),
        }
    }

    fn store(&mut self, address: R16, value: u16) {
        match address {
            R16::AF => self.cpu.set_af(value),
            R16::BC => self.cpu.set_bc(value),
            R16::DE => self.cpu.set_de(value),
            R16::HL => self.cpu.set_hl(value),
        };
    }
}
