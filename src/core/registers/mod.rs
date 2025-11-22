pub(crate) mod joypad;
pub(crate) mod serial;
pub(crate) mod sound;
pub(crate) mod timer;

use std::{
    cell::RefCell,
    fmt::Debug,
    ops::{Index, IndexMut},
    rc::Rc,
};

use joypad::Joypad;

use crate::core::{
    ppu::Ppu,
    registers::{serial::Serial, sound::SoundController, timer::TimerController},
};

type HardwareRegister = Rc<RefCell<dyn MemoryMappedRegister<Output = u8>>>;

#[derive(Debug)]
pub struct HardwareRegisters {
    pub joypad: HardwareRegister,
    // pub serial: HardwareRegister,
    pub interrupt_flag: u8,
    // pub sound: HardwareRegister,
    // pub timer: HardwareRegister,
    pub ppu: HardwareRegister,
    pub boot: u8,
    pub interrupt_enable: u8,
}

impl HardwareRegisters {
    pub fn new(
        joypad: Rc<RefCell<Joypad>>,
        // serial: Rc<RefCell<Serial>>,
        // sound: Rc<RefCell<SoundController>>,
        // timer: Rc<RefCell<TimerController>>,
        ppu: Rc<RefCell<Ppu>>,
    ) -> Self {
        HardwareRegisters {
            joypad,
            // serial,
            interrupt_flag: 0,
            // sound,
            // timer,
            ppu,
            boot: 0,
            interrupt_enable: 0,
        }
    }
}

pub trait MemoryMappedRegister: Index<u16> + IndexMut<u16> + Debug {}
