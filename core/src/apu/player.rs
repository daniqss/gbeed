use crate::apu::BUFFER_SIZE;

pub trait AudioPlayer {
    fn playing_stereo(&self) -> bool;
    fn push_sample(&mut self, sample: i16);
    fn flush_buffer(&mut self) {}
}

pub struct DefaultAudioPlayer {
    buffer: Box<[i16; BUFFER_SIZE]>,
    sample_idx: usize,
}

impl DefaultAudioPlayer {
    pub fn new() -> Self {
        Self {
            buffer: Box::new([0i16; BUFFER_SIZE]),
            sample_idx: 0,
        }
    }
}
impl Default for DefaultAudioPlayer {
    fn default() -> Self { Self::new() }
}

impl AudioPlayer for DefaultAudioPlayer {
    fn playing_stereo(&self) -> bool { false }
    fn push_sample(&mut self, sample: i16) {
        if self.sample_idx < BUFFER_SIZE {
            self.buffer[self.sample_idx] = sample;
            self.sample_idx += 1;
        }
    }
    fn flush_buffer(&mut self) {
        self.buffer.fill(0);
        self.sample_idx = 0;
    }
}
