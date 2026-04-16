use crate::apu::*;

pub struct Noise {
    /// initial length timer (write only)
    pub length_timer: u8,

    /// controls the initial volume, envelope direction and sweep pace
    /// - bit 7-4: initial volume
    /// - bit 3: envelope direction (0 = decrease, 1 = increase)
    /// - bit 2-0: sweep pace
    pub envelope: u8,

    /// controls the frequency and randomness of the noise output
    /// - bit 7-4: clock shift
    /// - bit 3: lfsr width (0 = 15-bit, 1 = 7-bit)
    /// - bit 2-0: clock divider (0 is treated as 0.5)
    pub frequency: u8,

    /// control bits (write only for trigger, read/write for length enable)
    /// - bit 7: trigger (write only)
    /// - bit 6: length enable (read/write)
    pub control: u8,

    pub enabled: bool,
    pub timer: u32,
    pub current_volume: u8,
    pub env_timer: u8,
    /// linear feedback shift register (15-bit or 7-bit depending on nr43 bit 3)
    pub lfsr: u16,
}

impl Noise {
    pub fn new() -> Self {
        Self {
            length_timer: 0,
            envelope: 0,
            frequency: 0,
            control: 0,

            enabled: false,
            timer: 0,
            current_volume: 0,
            env_timer: 0,
            lfsr: 0,
        }
    }

    pub fn read(&self, addr: u16) -> u8 {
        match addr {
            NR41 => 0xFF,
            NR42 => self.envelope,
            NR43 => self.frequency,
            NR44 => (self.control & 0x40) | 0xBF,

            _ => 0xFF,
        }
    }

    pub fn write(&mut self, addr: u16, value: u8) {
        match addr {
            NR41 => self.length_timer = value & 0x3F,
            NR42 => {
                self.envelope = value;
                // if bits 3-7 are 0, dac turns off
                if value & 0xF8 == 0 {
                    self.enabled = false;
                }
            }
            NR43 => self.frequency = value,
            NR44 => {
                self.control = value;
                if value & 0x80 != 0 {
                    self.trigger();
                }
            }
            _ => {}
        }
    }

    fn trigger(&mut self) {
        // only activates if dac is on (nr42 bits 3-7 != 0)
        if self.envelope & 0xF8 != 0 {
            self.enabled = true;
        }

        // reset length timer if expired
        if self.length_timer == 0 {
            self.length_timer = 64;
        }

        // reset timer
        self.timer = self.get_period();

        // reset volume and envelope
        self.current_volume = (self.envelope & 0xF0) >> 4;
        self.env_timer = self.envelope & 0x07;

        // reset lfsr (all bits set to 1)
        self.lfsr = 0x7FFF;
    }

    /// computes the timer period from the clock divider and clock shift
    /// following 262144 / divider / 2^shift hz (divider 0 treated as 0.5)
    /// shifts 14 and 15 stop the channel from being clocked entirely
    fn get_period(&self) -> u32 {
        let shift = (self.frequency >> 4) as u32;
        let divider = (self.frequency & 0x07) as u32;

        // shifts 14 and 15 disable clocking entirely
        if shift >= 14 {
            return u32::MAX;
        }

        // divider 0 is treated as 0.5, so the period is halved
        if divider == 0 {
            (1u32 << shift) * 8 / 2
        } else {
            divider * (1u32 << shift) * 8 * 2
        }
    }

    pub fn get_sample(&self) -> i16 {
        if !self.enabled {
            return 0;
        }

        // bit 0 of lfsr holds the current output: 0 = silence, 1 = volume
        if self.lfsr & 0x01 == 0 {
            self.current_volume as i16
        } else {
            0
        }
    }

    pub fn tick(&mut self) {
        if self.timer == u32::MAX {
            return;
        }

        if self.timer > 0 {
            self.timer -= 1;
        }

        if self.timer == 0 {
            self.timer = self.get_period();
            self.clock_lfsr();
        }
    }

    /// clocks the lfsr and shifts a new bit in based on xor of bits 0 and 1
    fn clock_lfsr(&mut self) {
        let feedback = self.lfsr & 0x01 ^ (self.lfsr >> 1) & 0x01;

        // shift right and place feedback into bit 14
        self.lfsr = (self.lfsr >> 1) | (feedback << 14);

        // in 7-bit mode (nr43 bit 3 set), also place feedback into bit 6
        if self.frequency & 0x08 != 0 {
            self.lfsr = (self.lfsr & !0x40) | (feedback << 6);
        }
    }
}
