pub(crate) mod interrupts;
pub(crate) mod joypad;
pub(crate) mod serial;
pub(crate) mod timer;

use crate::prelude::*;
use std::fmt::Debug;

use joypad::Joypad;

use crate::core::{interrupts::*, joypad::*, ppu::*, serial::*, timer::*};

#[derive(Debug)]
pub struct HardwareRegisters {
    pub joypad: Rc<RefCell<Joypad>>,
    pub serial: Rc<RefCell<Serial>>,
    pub interrupt_flag: Rc<RefCell<Interrupt>>,
    // pub sound: HardwareRegister,
    pub timer: Rc<RefCell<TimerController>>,
    pub ppu: Rc<RefCell<Ppu>>,
    pub boot: u8,
    pub interrupt_enable: Rc<RefCell<Interrupt>>,
}

impl HardwareRegisters {
    pub fn new(
        joypad: Rc<RefCell<Joypad>>,
        serial: Rc<RefCell<Serial>>,
        timer: Rc<RefCell<TimerController>>,
        interrupt_flag: Rc<RefCell<Interrupt>>,
        // sound: Rc<RefCell<SoundController>>,
        ppu: Rc<RefCell<Ppu>>,
        interrupt_enable: Rc<RefCell<Interrupt>>,
    ) -> Self {
        HardwareRegisters {
            joypad,
            serial,
            interrupt_flag,
            // sound,
            timer,
            ppu,
            boot: 0,
            interrupt_enable,
        }
    }

    pub fn read(&self, address: u16) -> u8 {
        match address {
            // joypad
            JOYP => self.joypad.borrow().0,

            // serial
            SB => self.serial.borrow().sb,
            SC => self.serial.borrow().sc,

            // timer
            DIV => self.timer.borrow().divider,
            TIMA => self.timer.borrow().timer_counter,
            TMA => self.timer.borrow().timer_modulo,
            TAC => self.timer.borrow().timer_control,

            // interrupt flag
            IF => self.interrupt_flag.borrow().0,

            0xFF50 => self.boot,

            // interrupt enable
            IE => self.interrupt_enable.borrow().0,
            _ => panic!("Read from unimplemented hardware register: {:04X}", address),
        }
    }
}
