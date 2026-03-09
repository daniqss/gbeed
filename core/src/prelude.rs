pub use crate::cartrigde::Cartridge;
pub use crate::dmg::Dmg;
pub use crate::joypad::{Joypad, JoypadButton};
pub use crate::memory::{Accessible, Accessible16};
pub use crate::ppu::{DMG_SCREEN_HEIGHT, DMG_SCREEN_WIDTH};
pub use crate::utils;
pub use crate::{bit_accessors, field_bit_accessors, flag_methods, mem_range, reg16};
pub use std::{
    cell::RefCell,
    io,
    ops::{Index, IndexMut},
    rc::Rc,
};

pub type Error = Box<dyn std::error::Error>;
pub type Result<T> = std::result::Result<T, Error>;
