use crate::{core::Accessible, mem_range, prelude::*};

mem_range!(SERIAL_REGISTER, SB, SC);

pub const SB: u16 = 0xFF01;
pub const SC: u16 = 0xFF02;

pub const SC_TRANSFER_START: u8 = 0x80;
pub const SC_CLOCK_SPEED: u8 = 0x02;
pub const SC_SHIFT_CLOCK: u8 = 0x01;

/// # Serial Data Transfer
/// Used for serial communication using a Link Cable between two Game Boys
/// ## Serial transfer data (SB) - 0xFF01
/// This register holds the data to be transferred/received via the serial link.
/// Each cycle the most significant bit is shifted out to the link cable, while the least significant bit is filled with data received from.
/// ## Serial transfer control (SC) - 0xFF02
/// - Transfer enable (Read/Write): If 1, a transfer is either requested or in progress.
/// - Clock speed [CGB Mode only] (Read/Write): If set to 1, enable high speed serial clock (~256 kHz in normal-speed mode)
/// - Clock select (Read/Write): 0 = External clock (“slave”), 1 = Internal clock (“master”).
#[derive(Debug, Default)]
pub struct Serial {
    pub sb: u8,
    pub sc: u8,
}

impl Serial {
    pub fn new() -> Self { Self { sb: 0x00, sc: 0x7E } }

    bit_accessors!(
        target: sc;

        SC_TRANSFER_START,
        SC_CLOCK_SPEED,
        SC_SHIFT_CLOCK
    );
}

impl Accessible<u16> for Serial {
    fn read(&self, address: u16) -> u8 {
        match address {
            SB => self.sb,
            SC => self.sc,
            _ => unreachable!(
                "Serial: read of address {address:04X} should have been handled by other components",
            ),
        }
    }

    fn write(&mut self, address: u16, value: u8) {
        match address {
            SB => self.sb = value,
            SC => self.sc = value,
            _ => unreachable!(
                "Serial: write of address {address:04X} should have been handled by other components",
            ),
        }
    }
}
