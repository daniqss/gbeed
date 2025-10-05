mod cpu;
mod memory;

use cpu::Cpu;
use memory::Memory;

pub struct Dmg {
    pub cpu: Cpu,
    pub memory: Memory,
    pub cycles: u64,
}

impl Dmg {
    pub fn new() -> Dmg {
        Dmg {
            cpu: Cpu::new(),
            memory: Memory::new(),
            cycles: 0,
        }
    }

    pub fn reset(&mut self) {
        self.cpu.reset();
        self.cycles = 0;
    }
}
