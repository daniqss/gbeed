mod joypad;

use std::ops::{Index, IndexMut};

use joypad::Joypad;

pub trait MemoryMappedRegister: Index<u16> + IndexMut<u16> {}
