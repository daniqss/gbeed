mod channels;
mod player;

pub use channels::*;
pub use player::*;

use crate::prelude::*;

mem_range!(APU_REGISTER, 0xFF10, 0xFF3F);

const NR10: u16 = 0xFF10;
const NR11: u16 = 0xFF11;
const NR12: u16 = 0xFF12;
const NR13: u16 = 0xFF13;
const NR14: u16 = 0xFF14;
const NR21: u16 = 0xFF16;
const NR22: u16 = 0xFF17;
const NR23: u16 = 0xFF18;
const NR24: u16 = 0xFF19;
const NR30: u16 = 0xFF1A;
const NR31: u16 = 0xFF1B;
const NR32: u16 = 0xFF1C;
const NR33: u16 = 0xFF1D;
const NR34: u16 = 0xFF1E;
const NR41: u16 = 0xFF20;
const NR42: u16 = 0xFF21;
const NR43: u16 = 0xFF22;
const NR44: u16 = 0xFF23;
const NR50: u16 = 0xFF24;
const NR51: u16 = 0xFF25;
const NR52: u16 = 0xFF26;

const AUDIO_ON_OFF: u8 = 0x80;

const CH4_LEFT: u8 = 0x80;
const CH3_LEFT: u8 = 0x40;
const CH2_LEFT: u8 = 0x20;
const CH1_LEFT: u8 = 0x10;
const CH4_RIGHT: u8 = 0x08;
const CH3_RIGHT: u8 = 0x04;
const CH2_RIGHT: u8 = 0x02;
const CH1_RIGHT: u8 = 0x01;
// const CH4_ON_FLAG: u8 = 0x08;
// const CH3_ON_FLAG: u8 = 0x04;
// const CH2_ON_FLAG: u8 = 0x02;
// const CH1_ON_FLAG: u8 = 0x01;

// const CHANNEL_DIVISORS: [u32; 8] = [8, 16, 32, 48, 64, 80, 96, 112];

// pub const FRAME_SEQUENCER_RATE: u32 = 512;
pub const CPU_FREQ: u32 = 4_194_304;

pub const SAMPLE_RATE: u32 = 44100;
pub const BUFFER_SIZE: usize = 4096;
pub const STEREO_BUFFER_SIZE: usize = BUFFER_SIZE * 2;

#[derive(Debug)]
pub struct Apu {
    sweep_pulse: SweepPulse,
    pulse: Pulse,
    wave: Wave,
    noise: Noise,

    /// - bit 7: works as `sound_panning` left but for the cartridge output (unused)
    /// - bit 6-4: master volume for left output
    /// - bit 3: works as `sound_panning` right but for the cartridge output (unused)
    /// - bit 2-0: master volume for right output
    ///
    /// A value of 0 is treated as a volume of 1 (very quiet), and a value of 7 is treated as a volume of 8 (no volume reduction).
    /// Importantly, the amplifier never mutes a non-silent input.
    master_volume: u8,

    /// channels can be panned left, center or right
    /// bits 7-4: channels to left (ch4-ch1)
    /// bits 3-0: channels to right (ch4-ch1)
    /// setting a bit to 1 enables the channel to go to the respective output
    sound_panning: u8,

    /// Audio master control
    /// - bit 7: audio on/off
    /// - bit 6-4: unused
    /// - bit 3-0: channels on/off (read-only)
    master_control: u8,

    frame_sequencer: u8,
    cycles: u32,

    length_counter_ch1: u16,
    length_counter_ch2: u16,
    length_counter_ch3: u16,
    length_counter_ch4: u16,

    envelope_volume_ch1: u8,
    envelope_volume_ch2: u8,
    envelope_volume_ch4: u8,

    envelope_timer_ch1: u8,
    envelope_timer_ch2: u8,
    envelope_timer_ch4: u8,

    sample_counter: u32,
}

impl Default for Apu {
    fn default() -> Self { Self::new() }
}

impl Apu {
    pub fn new() -> Self {
        Self {
            sweep_pulse: SweepPulse::new(),
            pulse: Pulse::new(),
            wave: Wave::new(),
            noise: Noise::new(),

            master_volume: 0x77,

            sound_panning: 0xF3,
            master_control: 0xF1,

            frame_sequencer: 0,
            cycles: 0,

            length_counter_ch1: 64,
            length_counter_ch2: 64,
            length_counter_ch3: 256,
            length_counter_ch4: 64,

            envelope_volume_ch1: 0,
            envelope_volume_ch2: 0,
            envelope_volume_ch4: 0,

            envelope_timer_ch1: 0,
            envelope_timer_ch2: 0,
            envelope_timer_ch4: 0,

            sample_counter: 0,
        }
    }
    pub fn is_active(&self) -> bool { self.master_control & AUDIO_ON_OFF != 0 }

    fn get_master_volume_left(&self) -> i32 { ((self.master_volume >> 4) & 0x07) as i32 + 1 }

    fn get_master_volume_right(&self) -> i32 { (self.master_volume & 0x07) as i32 + 1 }

    bit_accessors!(target: sound_panning; CH1_LEFT, CH2_LEFT, CH3_LEFT, CH4_LEFT, CH1_RIGHT, CH2_RIGHT, CH3_RIGHT, CH4_RIGHT);

    pub fn step<P: AudioPlayer>(&mut self, player: &mut P, delta: usize) {
        if !self.is_active() {
            return;
        }

        let total_cycles = delta as u32;

        for _ in 0..total_cycles {
            self.sweep_pulse.tick();
            self.pulse.tick();
            self.wave.tick();
            self.noise.tick();

            self.sample_counter += SAMPLE_RATE;
            if self.sample_counter >= CPU_FREQ {
                self.sample_counter -= CPU_FREQ;
                let (left, right) = self.mix();

                player.push_sample(left, right);
            }

            self.cycles += 1;
            if self.cycles >= 8192 {
                self.cycles -= 8192;
                self.frame_sequencer = (self.frame_sequencer + 1) % 8;
                self.tick_frame_sequencer();
            }
        }

        // player.flush_buffer();
    }

    /// ticks the frame sequencer, which controls the frequency of sound updates
    fn tick_frame_sequencer(&mut self) {
        match self.frame_sequencer {
            // length counter (256hz)
            0 | 4 => {
                self.tick_length();
            }
            // length counter (256hz) and period sweep (128hz)
            2 | 6 => {
                self.tick_length();
                self.tick_sweep();
            }
            // volume envelope (64hz)
            7 => {
                self.tick_envelope();
            }
            _ => {}
        }
    }

    fn tick_length(&mut self) {
        if self.sweep_pulse.enabled && self.sweep_pulse.period_high & 0x40 != 0 && self.length_counter_ch1 > 0
        {
            self.length_counter_ch1 -= 1;
            if self.length_counter_ch1 == 0 {
                self.sweep_pulse.enabled = false;
            }
        }

        if self.pulse.enabled && self.pulse.period_high & 0x40 != 0 && self.length_counter_ch2 > 0 {
            self.length_counter_ch2 -= 1;
            if self.length_counter_ch2 == 0 {
                self.pulse.enabled = false;
            }
        }

        if self.wave.enabled && self.wave.period_high & 0x40 != 0 && self.length_counter_ch3 > 0 {
            self.length_counter_ch3 -= 1;
            if self.length_counter_ch3 == 0 {
                self.wave.enabled = false;
            }
        }
        if self.noise.enabled && self.noise.control & 0x40 != 0 && self.length_counter_ch4 > 0 {
            self.length_counter_ch4 -= 1;
            if self.length_counter_ch4 == 0 {
                self.noise.enabled = false;
            }
        }
    }

    fn tick_envelope(&mut self) {
        if self.sweep_pulse.enabled {
            if self.envelope_timer_ch1 > 0 {
                self.envelope_timer_ch1 -= 1;
            }
            if self.envelope_timer_ch1 == 0 {
                let envelope = self.sweep_pulse.envelope;
                let pace = envelope & 0x07;
                let direction = (envelope >> 3) & 0x01;

                if pace > 0 {
                    if direction == 0 && self.envelope_volume_ch1 > 0 {
                        self.envelope_volume_ch1 -= 1;
                    } else if direction == 1 && self.envelope_volume_ch1 < 15 {
                        self.envelope_volume_ch1 += 1;
                    }
                }
                self.envelope_timer_ch1 = if pace > 0 { pace } else { 8 };
            }
        }

        if self.pulse.enabled {
            if self.envelope_timer_ch2 > 0 {
                self.envelope_timer_ch2 -= 1;
            }
            if self.envelope_timer_ch2 == 0 {
                let envelope = self.pulse.envelope;
                let pace = envelope & 0x07;
                let direction = (envelope >> 3) & 0x01;

                if pace > 0 {
                    if direction == 0 && self.envelope_volume_ch2 > 0 {
                        self.envelope_volume_ch2 -= 1;
                    } else if direction == 1 && self.envelope_volume_ch2 < 15 {
                        self.envelope_volume_ch2 += 1;
                    }
                }
                self.envelope_timer_ch2 = if pace > 0 { pace } else { 8 };
            }
        }

        if self.noise.enabled {
            if self.envelope_timer_ch4 > 0 {
                self.envelope_timer_ch4 -= 1;
            }
            if self.envelope_timer_ch4 == 0 {
                let envelope = self.noise.envelope;
                let pace = envelope & 0x07;
                let direction = (envelope >> 3) & 0x01;

                if pace > 0 {
                    if direction == 0 && self.envelope_volume_ch4 > 0 {
                        self.envelope_volume_ch4 -= 1;
                    } else if direction == 1 && self.envelope_volume_ch4 < 15 {
                        self.envelope_volume_ch4 += 1;
                    }
                }
                self.envelope_timer_ch4 = if pace > 0 { pace } else { 8 };
            }
        }
    }

    fn tick_sweep(&mut self) {
        if self.sweep_pulse.enabled {
            let sweep = self.sweep_pulse.sweep;
            let pace = (sweep >> 4) & 0x07;
            let step = sweep & 0x07;

            if pace > 0 || step > 0 {
                self.sweep_pulse.sweep_tick();
            }
        }
    }

    fn mix(&self) -> (i16, i16) {
        let ch1_vol = if self.sweep_pulse.enabled {
            self.sweep_pulse.get_sample(self.envelope_volume_ch1)
        } else {
            0
        };

        let ch2_vol = if self.pulse.enabled {
            self.pulse.get_sample(self.envelope_volume_ch2)
        } else {
            0
        };

        let ch3_vol = if self.wave.enabled {
            self.wave.get_sample()
        } else {
            0
        };

        let ch4_vol = if self.noise.enabled {
            self.noise.get_sample(self.envelope_volume_ch4)
        } else {
            0
        };

        let mut left = 0i32;
        let mut right = 0i32;

        if self.ch1_left() {
            left += ch1_vol as i32;
        }
        if self.ch1_right() {
            right += ch1_vol as i32;
        }

        if self.ch2_left() {
            left += ch2_vol as i32;
        }
        if self.ch2_right() {
            right += ch2_vol as i32;
        }

        if self.ch3_left() {
            left += ch3_vol as i32;
        }
        if self.ch3_right() {
            right += ch3_vol as i32;
        }

        if self.ch4_left() {
            left += ch4_vol as i32;
        }
        if self.ch4_right() {
            right += ch4_vol as i32;
        }

        left = left * self.get_master_volume_left() * 60;
        right = right * self.get_master_volume_right() * 60;

        left = left.clamp(i16::MIN as i32, i16::MAX as i32);
        right = right.clamp(i16::MIN as i32, i16::MAX as i32);

        (left as i16, right as i16)
    }

    fn sync_envelope(&mut self) {
        let env1 = self.sweep_pulse.envelope;
        self.envelope_volume_ch1 = (env1 >> 4) & 0x0F;
        self.envelope_timer_ch1 = env1 & 0x07;

        let env2 = self.pulse.envelope;
        self.envelope_volume_ch2 = (env2 >> 4) & 0x0F;
        self.envelope_timer_ch2 = env2 & 0x07;

        let env4 = self.noise.envelope;
        self.envelope_volume_ch4 = (env4 >> 4) & 0x0F;
        self.envelope_timer_ch4 = env4 & 0x07;
    }

    fn sync_length(&mut self) {
        self.length_counter_ch1 = 64 - (self.sweep_pulse.length_timer & 0x3F) as u16;
        self.length_counter_ch2 = 64 - (self.pulse.length_timer & 0x3F) as u16;
        self.length_counter_ch3 = 256 - self.wave.length_timer as u16;
        self.length_counter_ch4 = 64 - (self.noise.length_timer & 0x3F) as u16;
    }
}

impl Accessible<u16> for Apu {
    fn read(&self, address: u16) -> u8 {
        match address {
            NR10..=NR14 => self.sweep_pulse.read(address),
            NR21..=NR24 => self.pulse.read(address),
            NR30..=NR34 => self.wave.read(address),
            NR41..=NR44 => self.noise.read(address),

            NR50 => self.master_volume,
            NR51 => self.sound_panning,
            NR52 => {
                (self.master_control & AUDIO_ON_OFF)
                    | 0x70
                    | self.noise.is_enabled()
                    | self.wave.is_enabled()
                    | self.pulse.is_enabled()
                    | self.sweep_pulse.is_enabled()
            }

            WAVE_RAM_START..=WAVE_RAM_END => self.wave.wave_ram[(address - WAVE_RAM_START) as usize],

            _ => 0xFF,
        }
    }

    fn write(&mut self, address: u16, value: u8) {
        match address {
            NR10..=NR14 => {
                self.sweep_pulse.write(address, value);
                if address == NR11 {
                    self.length_counter_ch1 = 64 - (value & 0x3F) as u16;
                } else if address == NR14 && value & 0x80 != 0 {
                    if self.length_counter_ch1 == 0 {
                        self.length_counter_ch1 = 64;
                    }
                    self.envelope_volume_ch1 = (self.sweep_pulse.envelope & 0xF0) >> 4;
                    self.envelope_timer_ch1 = self.sweep_pulse.envelope & 0x07;
                }
            }
            NR21..=NR24 => {
                self.pulse.write(address, value);
                if address == NR21 {
                    self.length_counter_ch2 = 64 - (value & 0x3F) as u16;
                } else if address == NR24 && value & 0x80 != 0 {
                    if self.length_counter_ch2 == 0 {
                        self.length_counter_ch2 = 64;
                    }
                    self.envelope_volume_ch2 = (self.pulse.envelope & 0xF0) >> 4;
                    self.envelope_timer_ch2 = self.pulse.envelope & 0x07;
                }
            }
            NR30..=NR34 => {
                self.wave.write(address, value);
                if address == NR31 {
                    self.length_counter_ch3 = 256 - value as u16;
                } else if address == NR34 && value & 0x80 != 0 && self.length_counter_ch3 == 0 {
                    self.length_counter_ch3 = 256;
                }
            }
            NR41..=NR44 => {
                self.noise.write(address, value);
                if address == NR41 {
                    self.length_counter_ch4 = 64 - (value & 0x3F) as u16;
                } else if address == NR44 && value & 0x80 != 0 {
                    if self.length_counter_ch4 == 0 {
                        self.length_counter_ch4 = 64;
                    }
                    self.envelope_volume_ch4 = (self.noise.envelope & 0xF0) >> 4;
                    self.envelope_timer_ch4 = self.noise.envelope & 0x07;
                }
            }

            NR50 => self.master_volume = value,
            NR51 => self.sound_panning = value,
            NR52 => {
                let was_active = self.master_control & AUDIO_ON_OFF != 0;
                let now_active = value & AUDIO_ON_OFF != 0;

                if !was_active && now_active {
                    self.frame_sequencer = 0;
                    self.cycles = 0;
                    self.sync_envelope();
                    self.sync_length();
                } else if was_active && !now_active {
                    self.sweep_pulse.enabled = false;
                    self.pulse.enabled = false;
                    self.wave.enabled = false;
                    self.noise.enabled = false;
                }

                self.master_control = value & AUDIO_ON_OFF;
            }

            WAVE_RAM_START..=WAVE_RAM_END => self.wave.wave_ram[(address - WAVE_RAM_START) as usize] = value,

            _ => {}
        }
    }
}
