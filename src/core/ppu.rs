use std::rc::Rc;

use crate::core::memory::MemoryBus;

pub struct Ppu {
    memory_bus: Rc<MemoryBus>,
}

impl Ppu {
    pub fn new(memory_bus: Rc<MemoryBus>) -> Self { Self { memory_bus } }
}
