use crate::prelude::*;

mem_range!(SERIAL_REGISTER, SB, SC);

pub const SB: u16 = 0xFF01;
pub const SC: u16 = 0xFF02;

pub const SC_TRANSFER_START: u8 = 0x81;
pub const SC_CLOCK_SPEED: u8 = 0x02;
pub const SC_CLOCK_SELECT: u8 = 0x01;

pub trait SerialListener {
    fn on_transfer(&mut self, data: u8);
}

/// # Serial Data Transfer
/// Used for serial communication using a Link Cable between two Game Boys
/// ## Serial transfer data (SB) - 0xFF01
/// This register holds the data to be transferred/received via the serial link.
/// Each cycle the most significant bit is shifted out to the link cable, while the least significant bit is filled with data received from.
/// ## Serial transfer control (SC) - 0xFF02
/// - Transfer enable (Read/Write): If 1, a transfer is either requested or in progress.
/// - Clock speed [CGB Mode only] (Read/Write): If set to 1, enable high speed serial clock (~256 kHz in normal-speed mode)
/// - Clock select (Read/Write): 0 = External clock ("slave"), 1 = Internal clock ("master").
#[derive(Default)]
pub struct Serial {
    pub sb: u8,
    pub sc: u8,
    serial_listener: Option<Rc<RefCell<dyn SerialListener>>>,
    pub data: Vec<char>,
}

impl std::fmt::Debug for Serial {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Serial {{ sb: {:#04X}, sc: {:#04X} }}", self.sb, self.sc)
    }
}

impl Serial {
    pub fn new() -> Self {
        Self {
            sb: 0x00,
            sc: 0x7E,
            serial_listener: None,
            data: Vec::new(),
        }
    }

    bit_accessors!(target: sc; SC_TRANSFER_START, SC_CLOCK_SPEED, SC_CLOCK_SELECT);

    pub fn set_serial_listener(&mut self, listener: Rc<RefCell<dyn SerialListener>>) {
        self.serial_listener = Some(listener);
    }
}

impl Accessible<u16> for Serial {
    fn read(&self, address: u16) -> u8 {
        match address {
            SB => self.sb,
            SC => self.sc | 0x7E,
            _ => unreachable!(
                "Serial: read of address {address:04X} should have been handled by other components",
            ),
        }
    }

    fn write(&mut self, address: u16, value: u8) {
        let value = value & 0xFF;

        match address {
            SB => self.sb = value,

            // TODO: interrupt should be triggered at the end of the transfer
            SC => {
                self.sc = value;

                if self.sc_transfer_start() && self.sc_clock_select() {
                    self.data.push(self.sb as char);

                    if let Some(listener) = &mut self.serial_listener {
                        listener.borrow_mut().on_transfer(self.sb);
                    }

                    self.sc &= 0x7F;
                    self.sb = 0xFF;
                }
            }
            _ => unreachable!(
                "Serial: write of address {address:04X} should have been handled by other components",
            ),
        }
    }
}
