use crate::{core::Accessible, mem_range, prelude::*};

pub const CLOCKS_SPEEDS: [u32; 4] = [4_096, 262_144, 65_536, 16_384];

pub const DIV: u16 = 0xFF04;
pub const TIMA: u16 = 0xFF05;
pub const TMA: u16 = 0xFF06;
pub const TAC: u16 = 0xFF07;

pub const TIMER_START: u8 = 0x04;
/// controlls the frequency at which time counter is incremented
pub const INPUT_CLOCK_SELECT: u8 = 0x03;

mem_range!(TIMER_REGISTER, 0xFF04, 0xFF07);

#[derive(Debug, Default)]
pub struct Timer {
    pub cycles: usize,

    pub divider: u8,
    pub timer_counter: u8,
    pub timer_modulo: u8,
    pub timer_control: u8,
}

impl Timer {
    pub fn new() -> Self {
        Timer {
            cycles: 0,

            divider: 0,
            timer_counter: 0,
            timer_modulo: 0,
            timer_control: 0,
        }
    }

    bit_accessors!(
        target: timer_control;

        TIMER_START,
    );

    /// Possible speeds in DMG, SGB2, CGB in normal-speed mode:
    /// - 00 -> 4096 Hz
    /// - 01 -> 262144 Hz
    /// - 10 -> 65536 Hz
    /// - 11 -> 16384 Hz
    pub fn get_clock_speed(&self) -> u32 {
        let index = (self.timer_control & INPUT_CLOCK_SELECT) as usize;
        CLOCKS_SPEEDS[index]
    }

    // probably different in GBC double speed mode??
    pub fn step(&mut self, cycles: usize) {
        self.cycles += cycles;

        if self.cycles >= 256 {
            self.cycles -= 256;
            self.divider = self.divider.wrapping_add(1);
        }
    }
}

impl Accessible<u16> for Timer {
    fn read(&self, address: u16) -> u8 {
        match address {
            DIV => self.divider,
            TIMA => self.timer_counter,
            TMA => self.timer_modulo,
            TAC => self.timer_control,
            _ => unreachable!(
                "Timer: read of address {address:04X} should have been handled by other components",
            ),
        }
    }

    fn write(&mut self, address: u16, value: u8) {
        match address {
            DIV => self.divider = 0,
            TIMA => self.timer_counter = value,
            TMA => self.timer_modulo = value,
            TAC => self.timer_control = (self.timer_control & !INPUT_CLOCK_SELECT) | value,

            _ => unreachable!(
                "Timer: write of address {address:04X} should have been handled by other components",
            ),
        }
    }
}
