pub trait AudioPlayer {
    fn sample_rate(&self) -> u32;
    fn write_buffer(&mut self, samples: &[i16]);
    fn stereo(&self) -> bool;
}

pub struct DefaultAudioPlayer;

impl DefaultAudioPlayer {
    pub fn new() -> Self { Self }
}

impl Default for DefaultAudioPlayer {
    fn default() -> Self { Self::new() }
}

impl AudioPlayer for DefaultAudioPlayer {
    fn sample_rate(&self) -> u32 { 44100 }
    fn stereo(&self) -> bool { true }
    fn write_buffer(&mut self, _samples: &[i16]) {}
}
