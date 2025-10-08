mod cartrigde;
mod cpu;
mod license;
mod memory;
mod ppu;

use std::rc::Rc;

pub use cartrigde::Cartridge;
use cpu::Cpu;
use memory::MemoryBus;
use ppu::Ppu;

pub struct Dmg {
    pub cpu: Cpu,
    pub memory_bus: Rc<MemoryBus>,
    pub ppu: Ppu,
    pub cartridge: Cartridge,
}

impl Dmg {
    pub fn new(cartridge: Cartridge) -> Dmg {
        let memory_bus = Rc::new(MemoryBus::new());

        Dmg {
            cpu: Cpu::new(memory_bus.clone()),
            ppu: Ppu::new(memory_bus.clone()),
            memory_bus,
            cartridge,
        }
    }

    pub fn reset(&mut self) {
        self.cpu.reset();
    }

    pub fn run(&mut self) {
        loop {
            let instruction = self.memory_bus.read_word(self.cpu.pc);

            self.cpu.exec_next(instruction);
        }
    }
}
