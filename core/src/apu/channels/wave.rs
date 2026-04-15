use crate::apu::*;
use crate::prelude::*;

mem_range!(WAVE_RAM, 0xFF30, 0xFF3F);

pub struct Wave {
    /// This register controls CH3’s DAC. Like other channels, turning the DAC off immediately turns the channel off as well.
    pub dac_enable: bool,

    /// This register controls the channel’s length timer. (write only)
    pub length_timer: u8,

    /// Controls the channel's volume
    /// 00 => mute
    /// 01 => 100% volume
    /// 10 => 50% volume
    /// 11 => 25% volume
    pub output_level: u8,

    /// Eight first bits of the period value, three remaining bits are stored in NR34
    pub period_low: u8,

    /// Last three bits of the period value and control bits
    /// - bit 7: trigger (write only),
    /// - bit 6: length enable (read/write)
    pub period_high: u8,

    /// Each byte holds two samples, the channel reads it left to right, upper nibble first
    pub wave_ram: [u8; 16],

    pub enabled: bool,
    pub timer: u16,
    // from 0 to 31
    pub sample_idx: usize,
}

impl Wave {
    pub fn new() -> Self {
        Self {
            dac_enable: false,
            length_timer: 0,
            output_level: 0,
            period_low: 0,
            period_high: 0,
            wave_ram: [0; 16],

            enabled: false,
            timer: 0,
            sample_idx: 0,
        }
    }

    pub fn read(&self, addr: u16) -> u8 {
        match addr {
            NR30 => (if self.dac_enable { 0x80 } else { 0x00 } | 0x7F),
            NR31 => 0xFF,
            NR32 => (self.output_level << 5) | 0x9F,
            NR33 => 0xFF,
            NR34 => (self.period_high & 0x40) | 0xBF,
            WAVE_RAM_START..=WAVE_RAM_END => {
                // TODO: mmm
                self.wave_ram[(addr - WAVE_RAM_START) as usize]
            }
            _ => 0xFF,
        }
    }

    pub fn write(&mut self, addr: u16, value: u8) {
        match addr {
            NR30 => {
                self.dac_enable = (value & 0x80) == 0x80;

                // if DAC is off, the channel immediatly offs
                if !self.dac_enable {
                    self.enabled = false;
                }
            }
            NR31 => self.length_timer = value,
            NR32 => self.output_level = (value & 0x60) >> 5,
            NR33 => self.period_low = value,
            NR34 => {
                if value & 0x80 != 0 {
                    self.trigger();
                }
                self.period_high = value;
            }
            WAVE_RAM_START..=WAVE_RAM_END => self.wave_ram[(addr - 0xFF30) as usize] = value,

            _ => {}
        }
    }

    fn trigger(&mut self) {
        if self.dac_enable {
            self.enabled = true;
        }

        // reset the period
        let period = self.get_period();
        self.timer = (2048 - period) * 2;

        self.sample_idx = 1;
    }

    #[inline(always)]
    fn get_period(&self) -> u16 { ((self.period_high as u16 & 0x07) << 8) | (self.period_low as u16) }

    pub fn get_sample(&self) -> u8 {
        if !self.enabled || !self.dac_enable {
            return 0;
        }

        let byte_idx = self.sample_idx / 2;
        let byte = self.wave_ram[byte_idx];

        // read first high nibble
        let sample = if self.sample_idx % 2 == 0 {
            byte >> 4
        } else {
            byte & 0x0F
        };

        match self.output_level {
            0b00 => 0,
            0b01 => sample,
            0b10 => sample >> 1,
            0b11 => sample >> 2,

            _ => 0,
        }
    }
}
