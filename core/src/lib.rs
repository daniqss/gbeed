#![no_std]

extern crate alloc;

#[cfg(any(feature = "std", test))]
extern crate std;

mod apu;
mod cartrigde;
mod controller;
mod cpu;
mod dmg;
mod interrupts;
mod joypad;
mod memory;
mod ppu;
pub mod prelude;
mod serial;
mod timer;
mod utils;

#[doc(hidden)]
pub use pastey::paste as __paste;

// the emulator, its cartridge and every error they can produce
pub use cartrigde::{Cartridge, CartridgeError, CartridgeHeader, CartridgeResult, Destination};

pub use cartrigde::{CARTRIDGE_LOGO_END, CARTRIDGE_LOGO_START};
pub use cpu::InstructionError;
pub use dmg::{Dmg, DmgError};

pub use apu::Apu;
pub use cpu::{Cpu, Instructions};
pub use interrupts::Interrupt;
pub use joypad::{Joypad, JoypadButton};
pub use memory::*;
pub use ppu::{DMG_SCREEN_HEIGHT, DMG_SCREEN_WIDTH, Ppu};
pub use serial::Serial;
pub use timer::Timer;

pub use apu::{AudioPlayer, BUFFER_SIZE, DefaultAudioPlayer, SAMPLE_RATE};
pub use controller::{Controller, DefaultController};
pub use ppu::{DefaultRenderer, Renderer};
pub use serial::{DefaultSerialListener, SerialListener};
