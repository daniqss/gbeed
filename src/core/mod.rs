mod cartrigde;
mod cpu;
mod license;
pub mod memory;
mod ppu;

use std::{cell::RefCell, rc::Rc};

pub use cartrigde::Cartridge;
use cpu::Cpu;
use memory::MemoryBus;
use ppu::Ppu;

use crate::core::memory::Memory;

pub struct Dmg {
    pub cartridge: Cartridge,
    pub bus: MemoryBus,
    pub cpu: Cpu,
    pub ppu: Rc<RefCell<Ppu>>,
}

impl Dmg {
    pub fn new(cartridge: Cartridge, game_rom: Vec<u8>, boot_rom: Vec<u8>) -> Dmg {
        let ppu = Ppu::new();
        let bus = Memory::new(Some(game_rom), Some(boot_rom), Some(ppu.clone()));

        Dmg {
            cpu: Cpu::new(),
            ppu: ppu,
            bus,
            cartridge,
        }
    }

    pub fn reset(&mut self) { self.cpu.reset(); }

    pub fn run(&mut self) {
        loop {
            let opcode = self.bus.borrow()[self.cpu.pc];

            let mut instruction = match self.cpu.fetch(self.bus.clone(), opcode) {
                Ok(instr) => instr,
                Err(e) => {
                    eprintln!("Error fetching instruction: {}", e);
                    break;
                }
            };

            let writer = &mut String::new();
            match instruction.disassembly(writer) {
                Ok(_) => println!("{}", writer),
                Err(e) => {
                    eprintln!("Error disassembling instruction: {}", e);
                }
            }

            let (cycles, len, flags) = match instruction.exec() {
                Ok(effect) => (effect.cycles as usize, effect.len as u16, effect.flags),
                Err(e) => {
                    eprintln!("Error executing instruction: {}", e);
                    break;
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

            std::thread::sleep(std::time::Duration::from_secs(1));
        }
    }
}
