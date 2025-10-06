mod cartrigde;
mod cpu;
mod license;
mod memory;
mod ppu;

pub use cartrigde::Cartridge;
use cpu::Cpu;
use memory::Memory;
use ppu::Ppu;

pub struct Dmg {
    pub cpu: Cpu,
    pub memory: Memory,
    pub ppu: Ppu,
    pub cartridge: Cartridge,
}

impl Dmg {
    pub fn new(cartridge: Cartridge) -> Dmg {
        Dmg {
            cpu: Cpu::new(),
            memory: Memory::new(),
            ppu: Ppu::new(),
            cartridge,
        }
    }

    pub fn reset(&mut self) { self.cpu.reset(); }

    pub fn run(&mut self) {
        while true {
            let instruction = self.memory[self.cpu.pc];

            self.cpu.exec_next(instruction);
        }
    }
}
