use crate::{interrupts::Interrupt, prelude::*};

pub const DIV: u16 = 0xFF04;
pub const TIMA: u16 = 0xFF05;
pub const TMA: u16 = 0xFF06;
pub const TAC: u16 = 0xFF07;

pub const TIMER_ENABLE_MASK: u8 = 0b0000_0100;
pub const INPUT_CLOCK_SELECT_MASK: u8 = 0b0000_0011;

mem_range!(TIMER_REGISTER, 0xFF04, 0xFF07);

#[derive(Debug)]
pub struct Timer {
    pub div_cycles: usize,
    pub tima_cycles: usize,

    pub div: u8,
    pub tima: u8,
    pub tma: u8,
    pub tac: u8,
}

impl Timer {
    pub fn new() -> Self {
        Timer {
            div_cycles: 256,
            tima_cycles: 0,

            div: 0,
            tima: 0,
            tma: 0,
            tac: 0,
        }
    }

    pub fn step(&mut self, cycles: usize, interrupt: &mut Interrupt) {
        // DIV register always increments at 16384 Hz, every 256 CPU cycles in normal speed mode
        self.div_cycles += cycles;
        while self.div_cycles >= 256 {
            self.div = self.div.wrapping_add(1);
            self.div_cycles -= 256;
        }

        if (self.tac & TIMER_ENABLE_MASK) != 0 {
            self.tima_cycles += cycles;

            // Possible speeds in DMG, SGB2, CGB in normal-speed mode:
            // - 00 -> 4096 Hz
            // - 01 -> 262144 Hz
            // - 10 -> 65536 Hz
            // - 11 -> 16384 Hz
            let threshold = match self.tac & INPUT_CLOCK_SELECT_MASK {
                0 => 1024,
                1 => 16,
                2 => 64,
                3 => 256,
                _ => unreachable!(),
            };

            while self.tima_cycles >= threshold {
                self.tima_cycles -= threshold;
                let (result, overflow) = self.tima.overflowing_add(1);

                if overflow {
                    self.tima = self.tma;
                    interrupt.set_timer_interrupt(true);
                } else {
                    self.tima = result;
                }
            }
        }
    }
}

impl Default for Timer {
    fn default() -> Self { Self::new() }
}

impl Accessible<u16> for Timer {
    fn read(&self, address: u16) -> u8 {
        match address {
            DIV => self.div,
            TIMA => self.tima,
            TMA => self.tma,
            TAC => self.tac,
            _ => unreachable!(
                "Timer: read of address {address:04X} should have been handled by other components",
            ),
        }
    }

    fn write(&mut self, address: u16, value: u8) {
        match address {
            DIV => {
                self.div = 0;
                self.div_cycles = 0;
            }
            TIMA => self.tima = value,
            TMA => self.tma = value,
            TAC => self.tac = value,

            _ => unreachable!(
                "Timer: write of address {address:04X} should have been handled by other components",
            ),
        }
    }
}
