mod noise;
mod pulse;
mod sweep_pulse;
mod wave;

pub use noise::Noise;
pub use pulse::Pulse;
pub use sweep_pulse::SweepPulse;
pub use wave::{WAVE_RAM_END, WAVE_RAM_START, Wave};

// duty cycles table -> 12.5%, 25%, 50%, 75%
pub const DUTY_TABLE: [[u8; 8]; 4] = [
    [0, 0, 0, 0, 0, 0, 0, 1],
    [1, 0, 0, 0, 0, 0, 0, 1],
    [1, 0, 0, 0, 0, 1, 1, 1],
    [0, 1, 1, 1, 1, 1, 1, 0],
];
