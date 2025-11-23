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
use sdl2::libc::SCHED_BATCH;

use crate::core::{
    joypad::JOYPAD_ADDR,
    ppu::Ppu,
    registers::{serial::Serial, sound::SoundController, timer::TimerController},
    serial::{SB_REGISTER, SC_REGISTER},
};

#[derive(Debug)]
pub struct HardwareRegisters {
    pub joypad: Rc<RefCell<Joypad>>,
    pub serial: Rc<RefCell<Serial>>,
    pub interrupt_flag: u8,
    // pub sound: HardwareRegister,
    // pub timer: HardwareRegister,
    pub ppu: Rc<RefCell<Ppu>>,
    pub boot: u8,
    pub interrupt_enable: u8,
}

impl HardwareRegisters {
    pub fn new(
        joypad: Rc<RefCell<Joypad>>,
        serial: Rc<RefCell<Serial>>,
        // sound: Rc<RefCell<SoundController>>,
        // timer: Rc<RefCell<TimerController>>,
        ppu: Rc<RefCell<Ppu>>,
    ) -> Self {
        HardwareRegisters {
            joypad,
            serial,
            interrupt_flag: 0,
            // sound,
            // timer,
            ppu,
            boot: 0,
            interrupt_enable: 0,
        }
    }

    pub fn read(&self, address: u16) -> u8 {
        match address {
            JOYPAD_ADDR => self.joypad.borrow().0,
            SB_REGISTER => self.serial.borrow().sb,
            SC_REGISTER => self.serial.borrow().sc,
            0xFF0F => self.interrupt_flag,
            0xFF50 => self.boot,
            0xFFFF => self.interrupt_enable,
            _ => panic!("Read from unimplemented hardware register: {:04X}", address),
        }
    }
}
