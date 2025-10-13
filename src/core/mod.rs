mod cartrigde;
mod cpu;
mod instructions;
mod license;
mod memory;
mod ppu;

pub use cartrigde::Cartridge;
use cpu::Cpu;
use memory::MemoryBus;
use ppu::Ppu;

use crate::core::memory::Memory;

pub struct Dmg {
    pub cartridge: Cartridge,
    pub memory_bus: MemoryBus,
    pub cpu: Cpu,
    pub ppu: Ppu,
}

impl Dmg {
    pub fn new(cartridge: Cartridge, game_rom: Vec<u8>, boot_rom: Vec<u8>) -> Dmg {
        let memory_bus = Memory::new(Some(game_rom), Some(boot_rom));

        Dmg {
            cpu: Cpu::new(memory_bus.clone()),
            ppu: Ppu::new(memory_bus.clone()),
            memory_bus,
            cartridge,
        }
    }

    pub fn reset(&mut self) { self.cpu.reset(); }

    pub fn run(&mut self) {
        loop {
            let opcode = self.memory_bus.borrow()[self.cpu.pc];

            match self.cpu.exec(opcode) {
                Ok(effect) => {
                    self.cpu.cycles += effect.cycles as usize;
                    self.cpu.pc += effect.len;
                    match effect.flags {
                        Some(f) => self.cpu.f |= f,
                        None => {}
                    };
                }
                Err(e) => {
                    println!("Error: {}", e);

                    self.cpu.pc += 1;
                }
            };

            std::thread::sleep(std::time::Duration::from_secs(1));
        }
    }
}
