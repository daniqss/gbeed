use crate::{interrupts::Interrupt, prelude::*};

mem_range!(SERIAL_REGISTER, SB, SC);

pub const SB: u16 = 0xFF01;
pub const SC: u16 = 0xFF02;

pub const SC_TRANSFER_START: u8 = 0x80;
pub const SC_CLOCK_SPEED: u8 = 0x02;
pub const SC_CLOCK_SELECT: u8 = 0x01;

/// The internal serial clock runs at 8192 Hz, so 4096 T-cycles for a whole byte .
const CYCLES_PER_BIT: i32 = 512;
/// 32 times faster, so 128 T-cycles for a whole byte.
const FAST_CYCLES_PER_BIT: i32 = CYCLES_PER_BIT / 32;
const TRANSFER_BITS: u8 = 8;

pub trait SerialListener {
    fn on_transfer(&mut self, _data: u8) {}
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
pub struct Serial {
    pub sb: u8,
    pub sc: u8,

    /// byte captured when the transfer started, forwarded to the listener once it finishes
    transfer_data: u8,
    remaining_bits: u8,
    counter: i32,
}

impl core::fmt::Debug for Serial {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "Serial {{ sb: {:#04X}, sc: {:#04X} }}", self.sb, self.sc)
    }
}

impl Default for Serial {
    fn default() -> Self { Serial::new() }
}

impl Serial {
    pub(crate) fn new() -> Self {
        Self {
            sb: 0x00,
            sc: 0x7E,
            transfer_data: 0x00,
            remaining_bits: 0,
            counter: 0,
        }
    }

    /// Shifts out one bit per serial clock tick.
    /// Transfer and interrupt are done once the eight bits are gone.)
    pub(crate) fn step<S: SerialListener>(
        &mut self,
        cycles: usize,
        listener: &mut S,
        interrupt: &mut Interrupt,
    ) {
        if self.remaining_bits == 0 {
            return;
        }

        self.counter -= cycles as i32;
        if self.counter > 0 {
            return;
        }

        // the counter already went past the first tick, so the remaining overshoot tells
        // how many further ticks fit in this step
        let cycles_per_bit = self.cycles_per_bit();
        let ticks = 1 + (-self.counter / cycles_per_bit);
        let bits = ticks.min(self.remaining_bits as i32) as u8;

        self.counter += ticks * cycles_per_bit;
        self.remaining_bits -= bits;

        // no link cable attached, so the incoming bits are always high
        self.sb = match bits {
            TRANSFER_BITS.. => 0xFF,
            _ => (self.sb << bits) | ((1 << bits) - 1),
        };

        if self.remaining_bits == 0 {
            self.set_sc_transfer_start(false);
            interrupt.set_serial_interrupt(true);
            listener.on_transfer(self.transfer_data);
        }
    }

    #[inline(always)]
    fn cycles_per_bit(&self) -> i32 {
        if self.sc_clock_speed() {
            FAST_CYCLES_PER_BIT
        } else {
            CYCLES_PER_BIT
        }
    }

    bit_accessors!(pub(crate) target: sc; SC_TRANSFER_START, SC_CLOCK_SPEED, SC_CLOCK_SELECT);
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
        match address {
            SB => self.sb = value,

            // a transfer only starts when this game boy provides the clock, otherwise it
            // stays pending waiting for an external clock that never arrives
            SC => {
                self.sc = value;

                if self.sc_transfer_start() && self.sc_clock_select() {
                    self.transfer_data = self.sb;
                    self.remaining_bits = TRANSFER_BITS;
                    self.counter = self.cycles_per_bit();
                } else if !self.sc_transfer_start() {
                    self.remaining_bits = 0;
                }
            }
            _ => unreachable!(
                "Serial: write of address {address:04X} should have been handled by other components",
            ),
        }
    }
}
