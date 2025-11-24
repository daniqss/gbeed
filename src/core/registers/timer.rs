use crate::prelude::*;

pub const CLOCKS_SPEEDS: [u32; 4] = [4_096, 262_144, 65_536, 16_384];

pub const DIV: u16 = 0xFF04;
pub const TIMA: u16 = 0xFF05;
pub const TMA: u16 = 0xFF06;
pub const TAC: u16 = 0xFF07;

pub const TIMER_START: u8 = 0x04;
/// controlls the frequency at which time counter is incremented
pub const INPUT_CLOCK_SELECT: u8 = 0x03;

#[derive(Debug)]
pub struct TimerController {
    pub divider: u8,
    pub timer_counter: u8,
    pub timer_modulo: u8,
    pub timer_control: u8,
}

impl TimerController {
    pub fn new() -> Rc<RefCell<Self>> {
        Rc::new(RefCell::new(TimerController {
            divider: 0,
            timer_counter: 0,
            timer_modulo: 0,
            timer_control: 0,
        }))
    }

    bit_accessors!(
        target: timer_control;

        TIMER_START,
    );

    pub fn get_clock_speed(&self) -> u32 {
        let index = (self.timer_control & INPUT_CLOCK_SELECT) as usize;
        CLOCKS_SPEEDS[index]
    }

    /// possible speeds:
    /// - 00 -> 4096 Hz
    /// - 01 -> 262144 Hz
    /// - 10 -> 65536 Hz
    /// - 11 -> 16384 Hz
    pub fn set_clock_speed(&mut self, speed: u8) {
        if speed > 3 {
            panic!("Invalid clock speed: {}", speed);
        }
        self.timer_control = (self.timer_control & !INPUT_CLOCK_SELECT) | speed;
    }
}
