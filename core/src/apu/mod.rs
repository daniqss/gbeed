mod channels;
mod envelope;
mod length_counter;
mod player;

use channels::*;
use envelope::Envelope;
use length_counter::LengthCounter;
pub use player::*;

use crate::{cpu, prelude::*};

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

const CH4_ON_FLAG: u8 = 0x08;
const CH3_ON_FLAG: u8 = 0x04;
const CH2_ON_FLAG: u8 = 0x02;
const CH1_ON_FLAG: u8 = 0x01;

pub const SAMPLE_RATE: u32 = 44100;
pub const BUFFER_SIZE: usize = 4096;
pub const STEREO_BUFFER_SIZE: usize = BUFFER_SIZE * 2;

/// The base rate for the filter, this is used to calculate the
/// filter rate based on the clock frequency and the sampling rate.
const FILTER_RATE_BASE: f64 = 0.999958;

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

    sample_counter: u32,

    /// Per-channel state for the DC-blocking high-pass filter ([left, right]).
    filter_diff: [f32; 2],
    filter_rate: f32,
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

            sample_counter: 0,

            filter_diff: [0.0; 2],
            filter_rate: FILTER_RATE_BASE.powf(cpu::FREQUENCY as f64 / SAMPLE_RATE as f64) as f32,
        }
    }

    #[inline(always)]
    pub fn is_active(&self) -> bool { self.master_control & AUDIO_ON_OFF != 0 }

    #[inline(always)]
    fn get_master_volume_left(&self) -> i32 { ((self.master_volume >> 4) & 0x07) as i32 + 1 }

    #[inline(always)]
    fn get_master_volume_right(&self) -> i32 { (self.master_volume & 0x07) as i32 + 1 }

    bit_accessors!(target: master_control; AUDIO_ON_OFF, CH1_ON_FLAG, CH2_ON_FLAG, CH3_ON_FLAG, CH4_ON_FLAG);
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
            if self.sample_counter >= cpu::FREQUENCY {
                self.sample_counter -= cpu::FREQUENCY;
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
                self.sweep_pulse.tick_sweep();
            }
            // volume envelope (64hz)
            7 => {
                self.tick_envelope();
            }
            _ => {}
        }
    }

    fn tick_length(&mut self) {
        if self.sweep_pulse.period_high_length_enable() && self.sweep_pulse.length.clock() {
            self.sweep_pulse.enabled = false;
        }
        if self.pulse.period_high_length_enable() && self.pulse.length.clock() {
            self.pulse.enabled = false;
        }
        if self.wave.period_high_length_enable() && self.wave.length.clock() {
            self.wave.enabled = false;
        }
        if self.noise.control_length_enable() && self.noise.length.clock() {
            self.noise.enabled = false;
        }
    }

    fn tick_envelope(&mut self) {
        self.sweep_pulse
            .envelope_state
            .tick(self.sweep_pulse.envelope, self.sweep_pulse.enabled);
        self.pulse
            .envelope_state
            .tick(self.pulse.envelope, self.pulse.enabled);
        self.noise
            .envelope_state
            .tick(self.noise.envelope, self.noise.enabled);
    }

    /// applies the DC-blocking high-pass filter to a single channel sample
    /// removes the DC offset that the unsigned DAC outputs would otherwise leave
    #[inline(always)]
    fn high_pass(&mut self, channel: usize, sample: f32) -> f32 {
        let output = sample - self.filter_diff[channel];
        self.filter_diff[channel] = sample - (sample - self.filter_diff[channel]) * self.filter_rate;
        output
    }

    fn mix(&mut self) -> (i16, i16) {
        let ch1_vol = if self.sweep_pulse.enabled {
            self.sweep_pulse
                .get_sample(self.sweep_pulse.envelope_state.volume)
        } else {
            0
        };

        let ch2_vol = if self.pulse.enabled {
            self.pulse.get_sample(self.pulse.envelope_state.volume)
        } else {
            0
        };

        let ch3_vol = if self.wave.enabled {
            self.wave.get_sample()
        } else {
            0
        };

        let ch4_vol = if self.noise.enabled {
            self.noise.get_sample(self.noise.envelope_state.volume)
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

        // raw sum is 0..=60 per side; after master volume (1..=8) and the
        // high-pass filter the signal sits around 0 with peaks near +-240,
        // so ~130x fills i16 without routinely clipping
        const OUTPUT_GAIN: f32 = 130.0;

        let left = (left * self.get_master_volume_left()) as f32;
        let right = (right * self.get_master_volume_right()) as f32;

        let left = self.high_pass(0, left) * OUTPUT_GAIN;
        let right = self.high_pass(1, right) * OUTPUT_GAIN;

        let left = left.clamp(i16::MIN as f32, i16::MAX as f32) as i16;
        let right = right.clamp(i16::MIN as f32, i16::MAX as f32) as i16;

        (left, right)
    }

    fn sync_envelope(&mut self) {
        self.sweep_pulse.envelope_state.trigger(self.sweep_pulse.envelope);
        self.pulse.envelope_state.trigger(self.pulse.envelope);
        self.noise.envelope_state.trigger(self.noise.envelope);
    }
}

impl Accessible<u16> for Apu {
    fn read(&mut self, address: u16) -> u8 {
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
        // When APU is off, ignore writes to all registers except NR52, wave RAM,
        // and length timer registers (DMG behavior: NR11, NR21, NR31, NR41 are writable when off)
        if !self.is_active() && address != NR52 && !(WAVE_RAM_START..=WAVE_RAM_END).contains(&address) {
            match address {
                NR11 => self.sweep_pulse.length.counter = 64 - (value & 0x3F) as u16,
                NR21 => self.pulse.length.counter = 64 - (value & 0x3F) as u16,
                NR31 => self.wave.length.counter = 256 - value as u16,
                NR41 => self.noise.length.counter = 64 - (value & 0x3F) as u16,
                _ => {}
            }
            return;
        }

        let even_step = self.frame_sequencer.is_multiple_of(2);

        match address {
            NR10..=NR14 => self.sweep_pulse.write(address, value, even_step),
            NR21..=NR24 => self.pulse.write(address, value, even_step),
            NR30..=NR34 => self.wave.write(address, value, even_step),
            NR41..=NR44 => self.noise.write(address, value, even_step),

            NR50 => self.master_volume = value,
            NR51 => self.sound_panning = value,
            NR52 => {
                let was_active = self.audio_on_off();
                let now_active = value & AUDIO_ON_OFF != 0;

                if !was_active && now_active {
                    //start at step 7 so the first tick wraps to step 0 (length clock),
                    // matching hardware behavior where power-on offsets the next frame time by 8192 T-cycles.
                    self.frame_sequencer = 7;

                    self.cycles = 0;
                    self.sync_envelope();
                }
                // clear all registers NR10-NR51 when APU is turned off
                else if was_active && !now_active {
                    self.sweep_pulse.clear_registers();
                    self.pulse.clear_registers();
                    self.wave.clear_registers();
                    self.noise.clear_registers();
                    self.master_volume = 0;
                    self.sound_panning = 0;

                    // on DMG, length counters are preserved across power off/on
                }

                self.master_control = value & AUDIO_ON_OFF;
            }

            WAVE_RAM_START..=WAVE_RAM_END => self.wave.wave_ram[(address - WAVE_RAM_START) as usize] = value,
            _ => {}
        }
    }
}
