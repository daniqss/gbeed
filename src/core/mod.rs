mod cartrigde;
mod cpu;
mod license;
pub mod memory;
mod ppu;
mod registers;

use std::{cell::RefCell, rc::Rc};

pub use cartrigde::Cartridge;
use cpu::Cpu;
use memory::MemoryBus;
use ppu::Ppu;
pub use registers::*;

pub use crate::core::{joypad::Joypad, memory::Memory, serial::Serial, timer::TimerController};

pub struct Dmg {
    pub cartridge: Option<Cartridge>,
    pub bus: MemoryBus,
    pub cpu: Cpu,
    pub ppu: Rc<RefCell<Ppu>>,
    pub joypad: Rc<RefCell<Joypad>>,
    pub serial: Rc<RefCell<Serial>>,
}

impl Dmg {
    pub fn new(cartridge: Option<Cartridge>, game_rom: Option<Vec<u8>>, boot_rom: Option<Vec<u8>>) -> Dmg {
        let ppu = Ppu::new();
        let timer = TimerController::new();
        let joypad = Rc::new(RefCell::new(Joypad::default()));
        let serial = Rc::new(RefCell::new(Serial::new()));

        let registers = HardwareRegisters {
            joypad: joypad.clone(),
            serial: serial.clone(),
            timer: timer.clone(),
            interrupt_flag: 0,
            // sound: Rc::new(RefCell::new(registers::sound::SoundController {})),
            // timer: Rc::new(RefCell::new(registers::timer::TimerController {})),
            ppu: ppu.clone(),
            boot: 0,
            interrupt_enable: 0,
        };

        let start_at_boot = boot_rom.is_some();
        let bus = Memory::new(game_rom, boot_rom, Some(registers));

        Dmg {
            cpu: Cpu::new(start_at_boot),
            ppu,
            joypad,
            serial,
            bus,
            cartridge,
        }
    }

    pub fn reset(&mut self) { self.cpu.reset(); }

    pub fn run(&mut self) {
        let opcode = self.bus.borrow()[self.cpu.pc];

        let mut instruction = match self.cpu.fetch(self.bus.clone(), opcode) {
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

        let (cycles, len, flags) = match instruction.exec() {
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
