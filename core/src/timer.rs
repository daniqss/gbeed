use crate::{interrupts::Interrupt, prelude::*};

pub const DIV: u16 = 0xFF04;
pub const TIMA: u16 = 0xFF05;
pub const TMA: u16 = 0xFF06;
pub const TAC: u16 = 0xFF07;

pub const TIMER_ENABLE_MASK: u8 = 0b0000_0100;
pub const INPUT_CLOCK_SELECT_MASK: u8 = 0b0000_0011;

const CLOCK_BITS: [u8; 4] = [9, 3, 5, 7];

mem_range!(TIMER_REGISTER, 0xFF04, 0xFF07);

#[derive(Debug)]
pub struct Timer {
    pub internal_counter: u16,
    pub tima: u8,
    pub tma: u8,
    pub tac: u8,

    previous_bit: bool,
    overflow_pending: bool,
    overflow_delay: i32,
}

impl Timer {
    pub fn new() -> Self {
        Timer {
            internal_counter: 0xABCC,
            tima: 0,
            tma: 0,
            tac: 0,
            previous_bit: false,
            overflow_pending: false,
            overflow_delay: 0,
        }
    }

    pub fn step(&mut self, cycles: usize, interrupt: &mut Interrupt) {
        for _ in 0..cycles {
            if self.overflow_pending {
                self.overflow_delay -= 1;
                if self.overflow_delay <= 0 {
                    self.tima = self.tma;
                    interrupt.set_timer_interrupt(true);
                    self.overflow_pending = false;
                }
            }

            self.internal_counter = self.internal_counter.wrapping_add(1);

            if self.is_timer_enabled() {
                let current_bit = self.get_selected_bit();
                if self.previous_bit && !current_bit {
                    self.increment_tima();
                }
                self.previous_bit = current_bit;
            }
        }
    }

    fn is_timer_enabled(&self) -> bool { (self.tac & TIMER_ENABLE_MASK) != 0 }

    fn get_selected_bit(&self) -> bool {
        let bit_pos = CLOCK_BITS[(self.tac & INPUT_CLOCK_SELECT_MASK) as usize];
        ((self.internal_counter >> bit_pos) & 1) != 0
    }

    fn increment_tima(&mut self) {
        self.tima = self.tima.wrapping_add(1);
        if self.tima == 0 {
            self.overflow_pending = true;
            self.overflow_delay = 4;
        }
    }
}

impl Default for Timer {
    fn default() -> Self { Self::new() }
}

impl Accessible<u16> for Timer {
    fn read(&self, address: u16) -> u8 {
        match address {
            DIV => (self.internal_counter >> 8) as u8,
            TIMA => self.tima,
            TMA => self.tma,
            TAC => self.tac | 0xF8,
            _ => unreachable!(
                "Timer: read of address {address:04X} should have been handled by other components",
            ),
        }
    }

    fn write(&mut self, address: u16, value: u8) {
        match address {
            DIV => {
                let old_bit = self.is_timer_enabled() && self.get_selected_bit();
                self.internal_counter = 0;
                let new_bit = self.is_timer_enabled() && self.get_selected_bit();
                if old_bit && !new_bit {
                    self.increment_tima();
                }
                self.previous_bit = new_bit;
            }
            TIMA => {
                if !self.overflow_pending {
                    self.tima = value;
                }
            }
            TMA => self.tma = value,
            TAC => {
                let old_clock_bit = CLOCK_BITS[(self.tac & INPUT_CLOCK_SELECT_MASK) as usize];
                let old_enabled = self.is_timer_enabled();
                let old_bit = old_enabled && ((self.internal_counter >> old_clock_bit) & 1) != 0;

                self.tac = value & 0x07;

                let new_clock_bit = CLOCK_BITS[(self.tac & INPUT_CLOCK_SELECT_MASK) as usize];
                let new_enabled = self.is_timer_enabled();
                let new_bit = new_enabled && ((self.internal_counter >> new_clock_bit) & 1) != 0;

                if old_bit && !new_bit {
                    self.increment_tima();
                }
                self.previous_bit = new_bit;
            }
            _ => unreachable!("Timer: write {address:04X}"),
        }
    }
}
