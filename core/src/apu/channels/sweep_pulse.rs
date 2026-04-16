use super::DUTY_TABLE;
use crate::apu::*;

pub struct SweepPulse {
    /// controls ch1's period sweep functionality
    pub sweep: u8,

    /// wave duty, controls the output waveform
    pub wave_duty: u8,

    /// initial length timer (write only)
    pub length_timer: u8,

    /// controls the initial volume, envelope direction and sweep pace
    /// - bit 7-4: initial volume
    /// - bit 3: envelope direction (0 = decrease, 1 = increase)
    /// - bit 2-0: sweep pace
    pub envelope: u8,

    /// eight first bits of the period value
    pub period_low: u8,

    /// last three bits of the period value and control bits
    /// - bit 7: trigger (write only)
    /// - bit 6: length enable (read/write)
    /// - bit 2-0: period high
    pub period_high: u8,

    pub enabled: bool,
    pub timer: u16,
    /// from 0 to 7 (the waveform is 8 samples long)
    pub duty_step: u8,
    pub current_volume: u8,
    pub env_timer: u8,
    pub sweep_timer: u8,
    /// internal shadow period used by the sweep
    pub shadow_period: u16,
}

impl SweepPulse {
    pub fn new() -> Self {
        Self {
            sweep: 0,
            wave_duty: 0,
            length_timer: 0,
            envelope: 0,
            period_low: 0,
            period_high: 0,

            enabled: false,
            timer: 0,
            duty_step: 0,
            current_volume: 0,
            env_timer: 0,
            sweep_timer: 0,
            shadow_period: 0,
        }
    }

    pub fn read(&self, addr: u16) -> u8 {
        match addr {
            NR10 => self.sweep | 0x80,
            NR11 => (self.wave_duty << 6) | 0x3F,
            NR12 => self.envelope,
            NR13 => 0xFF,
            NR14 => (self.period_high & 0x40) | 0xBF,

            _ => 0xFF,
        }
    }

    pub fn write(&mut self, addr: u16, value: u8) {
        match addr {
            NR10 => self.sweep = value,
            NR11 => {
                self.wave_duty = (value & 0xC0) >> 6;
                self.length_timer = value & 0x3F;
            }
            NR12 => {
                self.envelope = value;
                // if bits 3-7 are 0, dac turns off
                if value & 0xF8 == 0 {
                    self.enabled = false;
                }
            }
            NR13 => self.period_low = value,
            NR14 => {
                self.period_high = value;
                if value & 0x80 != 0 {
                    self.trigger();
                }
            }
            _ => {}
        }
    }

    fn trigger(&mut self) {
        // only activates if dac is on (nr12 bits 3-7 != 0)
        if self.envelope & 0xF8 != 0 {
            self.enabled = true;
        }

        // reset length timer if expired
        if self.length_timer == 0 {
            self.length_timer = 64;
        }

        // reset period and timer
        let period = self.get_period();
        self.shadow_period = period;
        self.timer = (2048 - period) * 4;

        // reset volume and envelope
        self.current_volume = (self.envelope & 0xF0) >> 4;
        self.env_timer = self.envelope & 0x07;

        // reset sweep
        let sweep_pace = (self.sweep & 0x70) >> 4;
        let sweep_step = self.sweep & 0x07;
        self.sweep_timer = if sweep_pace > 0 { sweep_pace } else { 8 };

        // if sweep step is not 0, an initial overflow calculation is made
        if sweep_step > 0 {
            self.calculate_sweep(false);
        }
    }

    #[inline(always)]
    fn get_period(&self) -> u16 { ((self.period_high as u16 & 0x07) << 8) | (self.period_low as u16) }

    pub fn get_sample(&self) -> i16 {
        if !self.enabled {
            return 0;
        }

        let duty_pattern = DUTY_TABLE[self.wave_duty as usize];
        if duty_pattern[self.duty_step as usize] == 1 {
            self.current_volume as i16
        } else {
            0
        }
    }

    pub fn tick(&mut self) {
        if self.timer > 0 {
            self.timer -= 1;
        }

        if self.timer == 0 {
            let period = self.get_period();
            self.timer = (2048 - period) * 4;
            self.duty_step = (self.duty_step + 1) % 8;
        }
    }

    /// period sweep logic (nr10)
    fn calculate_sweep(&mut self, update: bool) {
        let step = self.sweep & 0x07;
        let negate = (self.sweep & 0x08) != 0;
        let new_period = self.shadow_period >> step;

        let new_period = if negate {
            self.shadow_period.wrapping_sub(new_period)
        } else {
            self.shadow_period.wrapping_add(new_period)
        };

        // overflow check
        if new_period > 0x7FF {
            self.enabled = false;
        } else if step > 0 && update {
            // write back to registers
            self.shadow_period = new_period;
            self.period_low = (new_period & 0xFF) as u8;
            self.period_high = (self.period_high & 0xF8) | ((new_period >> 8) & 0x07) as u8;

            // re-check overflow immediately after updating
            self.calculate_sweep(false);
        }
    }
}
