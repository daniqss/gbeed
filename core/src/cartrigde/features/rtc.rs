#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub enum SelectedRTCRegister {
    #[default]
    Seconds = 0x08,
    Minutes = 0x09,
    Hours = 0x0A,
    DayLow = 0x0B,
    DayHigh = 0x0C,
}

/// # Real Time Clock
/// Uses a 32.768 kHz quartz oscillator and needs a battery to keep ticking without Game Boy power supply.
#[derive(Debug, Clone, Default)]
pub struct Rtc {
    pub enabled: bool,
    pub selected_register: Option<SelectedRTCRegister>,

    pub seconds: u8,
    pub minutes: u8,
    pub hours: u8,
    // lower 8 bits of the day counter
    pub day_low: u8,
    // upper bit of the day counter, also carry bit and halt flag
    pub day_high: u8,

    pub latched_seconds: u8,
    pub latched_minutes: u8,
    pub latched_hours: u8,
    pub latched_day_low: u8,
    pub latched_day_high: u8,
}

impl Rtc {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn select_register(&mut self, value: u8) {
        self.selected_register = match value {
            0x08 => Some(SelectedRTCRegister::Seconds),
            0x09 => Some(SelectedRTCRegister::Minutes),
            0x0A => Some(SelectedRTCRegister::Hours),
            0x0B => Some(SelectedRTCRegister::DayLow),
            0x0C => Some(SelectedRTCRegister::DayHigh),
            _ => None,
        };
    }

    pub fn latch(&mut self) {
        self.latched_seconds = self.seconds;
        self.latched_minutes = self.minutes;
        self.latched_hours = self.hours;
        self.latched_day_low = self.day_low;
        self.latched_day_high = self.day_high;
    }

    pub fn read(&self) -> u8 {
        match self.selected_register {
            Some(SelectedRTCRegister::Seconds) => self.latched_seconds,
            Some(SelectedRTCRegister::Minutes) => self.latched_minutes,
            Some(SelectedRTCRegister::Hours) => self.latched_hours,
            Some(SelectedRTCRegister::DayLow) => self.latched_day_low,
            Some(SelectedRTCRegister::DayHigh) => self.latched_day_high,
            None => 0xFF,
        }
    }

    pub fn write(&mut self, value: u8) {
        match self.selected_register {
            Some(SelectedRTCRegister::Seconds) => self.seconds = value & 0x3F,
            Some(SelectedRTCRegister::Minutes) => self.minutes = value & 0x3F,
            Some(SelectedRTCRegister::Hours) => self.hours = value & 0x1F,
            Some(SelectedRTCRegister::DayLow) => self.day_low = value,
            Some(SelectedRTCRegister::DayHigh) => self.day_high = value & 0xC1,
            None => {}
        }
    }
}