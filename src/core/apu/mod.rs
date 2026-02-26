use crate::{core::Accessible, mem_range, prelude::*};

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
    pub fn new() -> Self {
        Self {
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
        }
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

impl Accessible<u16> for Apu {
    fn read(&self, address: u16) -> u8 {
        match address {
            NR10 => self.nr10,
            NR11 => self.nr11,
            NR12 => self.nr12,
            NR13 => self.nr13,
            NR14 => self.nr14,

            NR21 => self.nr21,
            NR22 => self.nr22,
            NR23 => self.nr23,
            NR24 => self.nr24,
            NR30 => self.nr30,
            NR31 => self.nr31,
            NR32 => self.nr32,
            NR33 => self.nr33,
            NR34 => self.nr34,

            NR41 => self.nr41,
            NR42 => self.nr42,
            NR43 => self.nr43,
            NR44 => self.nr44,
            NR50 => self.nr50,
            NR51 => self.nr51,
            NR52 => self.nr52,
            addr @ 0xFF30..=0xFF3F => self.wave_ram[(addr - 0xFF30) as usize],

            0xFF15 | 0xFF1F | 0xFF27..=0xFF2F => {
                println!("Reads to unimplemented Apu register {address:04X} return 0xFF");
                0xFF
            }
            _ => unreachable!(
                "Apu: read of address {address:04X} should have been handled by other components"
            ),
        }
    }

    fn write(&mut self, address: u16, value: u8) {
        match address {
            NR10 => self.nr10 = value,
            NR11 => self.nr11 = value,
            NR12 => self.nr12 = value,
            NR13 => self.nr13 = value,
            NR14 => self.nr14 = value,
            0xFF15 => println!("Writes in unused Apu memory range are ignored, {address:04X}"),
            NR21 => self.nr21 = value,
            NR22 => self.nr22 = value,
            NR23 => self.nr23 = value,
            NR24 => self.nr24 = value,
            NR30 => self.nr30 = value,
            NR31 => self.nr31 = value,
            NR32 => self.nr32 = value,
            NR33 => self.nr33 = value,
            NR34 => self.nr34 = value,
            0xFF1F => println!("Writes in unused Apu memory range are ignored, {address:04X}"),
            NR41 => self.nr41 = value,
            NR42 => self.nr42 = value,
            NR43 => self.nr43 = value,
            NR44 => self.nr44 = value,
            NR50 => self.nr50 = value,
            NR51 => self.nr51 = value,
            // audio master control, if turning off audio, disable all channels
            NR52 => self.nr52 = (value & AUDIO_ON_OFF) | (self.nr52 & 0x7F),
            0xFF27..=0xFF2F => println!("Writes in unused Apu memory range are ignored, {address:04X}"),
            addr @ 0xFF30..=0xFF3F => self.wave_ram[(addr - 0xFF30) as usize] = value,

            _ => unreachable!(
                "Apu: write of address {address:04X} should have been handled by other components"
            ),
        }
    }
}
