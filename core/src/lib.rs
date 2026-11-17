#![no_std]

#[cfg(feature = "alloc")]
extern crate alloc;

#[cfg(any(feature = "std", test))]
extern crate std;

mod apu;
/// cbindgen:ignore
mod cartrigde;
mod controller;
/// cbindgen:ignore
mod cpu;
mod dmg;
#[cfg(feature = "ffi")]
pub mod ffi;
/// cbindgen:ignore
mod interrupts;
/// cbindgen:ignore
mod joypad;
/// cbindgen:ignore
mod memory;
mod ppu;
pub mod prelude;
mod serial;
/// cbindgen:ignore
mod timer;
/// cbindgen:ignore
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
pub use ppu::{GB_SCREEN_HEIGHT, GB_SCREEN_WIDTH, Ppu};
pub use serial::Serial;
pub use timer::Timer;

pub use apu::{AudioPlayer, GB_AUDIO_BUFFER_SIZE, GB_SAMPLE_RATE, GB_STEREO_AUDIO_BUFFER_SIZE};
pub use controller::{Controller, DefaultController};
pub use ppu::Renderer;
pub use serial::SerialListener;
