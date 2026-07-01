mod noise;
mod pulse;
mod sweep_pulse;
mod wave;

pub use noise::Noise;
pub use pulse::Pulse;
pub use sweep_pulse::SweepPulse;
pub use wave::{WAVE_RAM_END, WAVE_RAM_START, Wave};

use super::{Envelope, LengthCounter};

// duty cycles table -> 12.5%, 25%, 50%, 75%
pub const DUTY_TABLE: [[u8; 8]; 4] = [
    [0, 0, 0, 0, 0, 0, 0, 1],
    [1, 0, 0, 0, 0, 0, 0, 1],
    [1, 0, 0, 0, 0, 1, 1, 1],
    [0, 1, 1, 1, 1, 1, 1, 0],
];

/// To access the trigger bit of NRx4
pub const TRIGGER: u8 = 0x80;
/// To access the length enable bit of NRx4
pub const LENGTH_ENABLE: u8 = 0x40;

/// Handles the common NRx4 control write logic shared by all channels.
/// Manages length enable transition clocking, trigger length reload, and envelope trigger.
#[inline(always)]
pub fn handle_control_write(
    length: &mut LengthCounter,
    enabled: &mut bool,
    envelope: Option<(&mut Envelope, u8)>,
    was_length_enabled: bool,
    value: u8,
    even_step: bool,
) {
    let is_length_enabled = value & LENGTH_ENABLE != 0;
    let is_trigger = value & TRIGGER != 0;

    // extra clock on length enable transition (0 -> 1) at even step
    if !was_length_enabled && is_length_enabled && even_step {
        LengthCounter::clock_and_disable(length, enabled);
    }

    if is_trigger {
        let was_frozen = length.trigger_reload();
        if let Some((env_state, env_reg)) = envelope {
            env_state.trigger(env_reg);
        }

        // extra clock when trigger unfreezes length (was 0 -> max) with enable at even step
        if was_frozen && is_length_enabled && even_step {
            LengthCounter::clock_and_disable(length, enabled);
        }
    }
}
