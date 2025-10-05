mod cpu;
mod memory;

use cpu::Cpu;
use memory::Memory;

pub struct Dmg {
    pub cpu: Cpu,
    pub memory: Memory,
}

impl Dmg {
    pub fn new() -> Dmg {
        Dmg {
            cpu: Cpu::new(),
            memory: Memory::new(),
        }
    }
}
