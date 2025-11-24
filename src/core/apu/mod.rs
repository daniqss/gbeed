use crate::prelude::*;

const AUDIO_ON_OFF: u8 = 0x80;
const CH4_ON_FLAG: u8 = 0x08;
const CH3_ON_FLAG: u8 = 0x04;
const CH2_ON_FLAG: u8 = 0x02;
const CH1_ON_FLAG: u8 = 0x01;

const TRIGGER: u8 = 0x80;
const LENGTH_ENABLE: u8 = 0x40;

const DAC_ENABLE: u8 = 0x80;

/// Sweep (NR10) & Envelope (NRx2) bits
const SWEEP_DIRECTION: u8 = 0x08; // 0=Add, 1=Sub
const ENVELOPE_DIRECTION: u8 = 0x08;

const LFSR_WIDTH: u8 = 0x08;

/// | Channel     | Control (4) | Frequency (3) | Volume (2) | Length (1) | Sweep (0) |
/// | :---------- | :---------: | :-----------: | :--------: | :--------: | :-------: |
/// | **Voice 1** |    NR14     |     NR13      |    NR12    |    NR11    |   NR10    |
/// | **Voice 2** |    NR24     |     NR23      |    NR22    |    NR21    |     -     |
/// | **Voice 3** |    NR34     |     NR33      |    NR32    |    NR31    |   NR30    |
/// | **Voice 4** |    NR44     |     NR43      |    NR42    |    NR41    |     -     |
#[derive(Debug, Default)]
pub struct Apu {
    // channel one, pulse with sweep
    nr10: u8,
    nr11: u8,
    nr12: u8,
    nr13: u8,
    nr14: u8,

    // channel two, pulse
    nr21: u8,
    nr22: u8,
    nr23: u8,
    nr24: u8,

    // channel 3, Wave
    nr30: u8,
    nr31: u8,
    nr32: u8,
    nr33: u8,
    nr34: u8,
    wave_ram: [u8; 16],

    // channel 4, noise
    nr41: u8,
    nr42: u8,
    nr43: u8,
    nr44: u8,

    // global sound control registers
    nr50: u8,
    nr51: u8,
    nr52: u8,
}

impl Apu {
    pub fn new() -> Rc<RefCell<Self>> {
        Rc::new(RefCell::new(Self {
            nr10: 0x80,
            nr11: 0xBF,
            nr12: 0xF3,
            nr14: 0xBF,
            nr21: 0x3F,
            nr22: 0x00,
            nr24: 0xBF,
            nr30: 0x7F,
            nr31: 0xFF,
            nr32: 0x9F,
            nr34: 0xBF,
            nr41: 0xFF,
            nr42: 0x00,
            nr43: 0x00,
            nr44: 0xBF,
            nr50: 0x77,
            nr51: 0xF3,
            nr52: 0xF1,
            ..Default::default()
        }))
    }

    bit_accessors! {
        target: nr52;

        AUDIO_ON_OFF,
        CH1_ON_FLAG,
        CH2_ON_FLAG,
        CH3_ON_FLAG,
        CH4_ON_FLAG
    }

    bit_accessors! { target: nr10; SWEEP_DIRECTION }

    field_bit_accessors! { target: nr12; ENVELOPE_DIRECTION }
    field_bit_accessors! { target: nr22; ENVELOPE_DIRECTION }
    field_bit_accessors! { target: nr42; ENVELOPE_DIRECTION }

    bit_accessors! { target: nr30; DAC_ENABLE }

    field_bit_accessors! { target: nr14; TRIGGER, LENGTH_ENABLE }
    field_bit_accessors! { target: nr24; TRIGGER, LENGTH_ENABLE }
    field_bit_accessors! { target: nr34; TRIGGER, LENGTH_ENABLE }
    field_bit_accessors! { target: nr44; TRIGGER, LENGTH_ENABLE }

    bit_accessors! { target: nr43; LFSR_WIDTH }

    pub fn is_active(&self) -> bool { self.nr52 & AUDIO_ON_OFF != 0 }
}

impl Index<u16> for Apu {
    type Output = u8;

    fn index(&self, address: u16) -> &Self::Output {
        match address {
            0xFF10 => &self.nr10,
            0xFF11 => &self.nr11,
            0xFF12 => &self.nr12,
            0xFF13 => &self.nr13,
            0xFF14 => &self.nr14,
            0xFF16 => &self.nr21,
            0xFF17 => &self.nr22,
            0xFF18 => &self.nr23,
            0xFF19 => &self.nr24,
            0xFF1A => &self.nr30,
            0xFF1B => &self.nr31,
            0xFF1C => &self.nr32,
            0xFF1D => &self.nr33,
            0xFF1E => &self.nr34,
            0xFF20 => &self.nr41,
            0xFF21 => &self.nr42,
            0xFF22 => &self.nr43,
            0xFF23 => &self.nr44,
            0xFF24 => &self.nr50,
            0xFF25 => &self.nr51,
            0xFF26 => &self.nr52,
            addr @ 0xFF30..=0xFF3F => &self.wave_ram[(addr - 0xFF30) as usize],
            _ => panic!("Invalid APU register read at address {:04X}", address),
        }
    }
}

impl IndexMut<u16> for Apu {
    fn index_mut(&mut self, address: u16) -> &mut Self::Output {
        match address {
            0xFF10 => &mut self.nr10,
            0xFF11 => &mut self.nr11,
            0xFF12 => &mut self.nr12,
            0xFF13 => &mut self.nr13,
            0xFF14 => &mut self.nr14,
            0xFF16 => &mut self.nr21,
            0xFF17 => &mut self.nr22,
            0xFF18 => &mut self.nr23,
            0xFF19 => &mut self.nr24,
            0xFF1A => &mut self.nr30,
            0xFF1B => &mut self.nr31,
            0xFF1C => &mut self.nr32,
            0xFF1D => &mut self.nr33,
            0xFF1E => &mut self.nr34,
            0xFF20 => &mut self.nr41,
            0xFF21 => &mut self.nr42,
            0xFF22 => &mut self.nr43,
            0xFF23 => &mut self.nr44,
            0xFF24 => &mut self.nr50,
            0xFF25 => &mut self.nr51,
            // TODO: if(byte & 0x80) sound.nr52 |= 0x80; else sound.nr52 &= 0x7F;???
            // should not be necessary but just in case
            0xFF26 => &mut self.nr52,
            addr @ 0xFF30..=0xFF3F => &mut self.wave_ram[(addr - 0xFF30) as usize],
            _ => panic!("Invalid APU register read at address {:04X}", address),
        }
    }
}
