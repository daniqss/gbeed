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
    pub cartridge: Rc<Cartridge>,
    pub memory_bus: Rc<MemoryBus>,
    pub cpu: Cpu,
    pub ppu: Ppu,
}

impl Dmg {
    pub fn new(cartridge: Cartridge) -> Dmg {
        let cartridge = Rc::new(cartridge);
        let memory_bus = Rc::new(MemoryBus::new(Some(cartridge.clone())));

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
            let instruction = self.memory_bus[self.cpu.pc];

            self.cpu.pc = self.cpu.exec_next(instruction);
        }
    }
}
