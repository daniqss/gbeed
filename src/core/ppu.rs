use std::{cell::RefCell, rc::Rc};

use crate::core::memory::MemoryBus;

pub struct Ppu {
    memory_bus: MemoryBus,
}

impl Ppu {
    pub fn new(memory_bus: MemoryBus) -> Self { Self { memory_bus } }
}
