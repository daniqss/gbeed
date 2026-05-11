use crate::apu::STEREO_BUFFER_SIZE;

pub trait AudioPlayer {
    fn playing_stereo(&self) -> bool;
    fn push_sample(&mut self, left: i16, right: i16);
    fn flush_buffer(&mut self) {}
}

pub struct DefaultAudioPlayer {
    buffer: Box<[i16; STEREO_BUFFER_SIZE]>,
    sample_idx: usize,
}

impl DefaultAudioPlayer {
    pub fn new() -> Self {
        Self {
            buffer: Box::new([0i16; STEREO_BUFFER_SIZE]),
            sample_idx: 0,
        }
    }
}
impl Default for DefaultAudioPlayer {
    fn default() -> Self { Self::new() }
}

impl AudioPlayer for DefaultAudioPlayer {
    fn playing_stereo(&self) -> bool { true }
    fn push_sample(&mut self, left: i16, right: i16) {
        if self.sample_idx < STEREO_BUFFER_SIZE - 1 {
            self.buffer[self.sample_idx] = left;
            self.buffer[self.sample_idx + 1] = right;
            self.sample_idx += 2;
        }
    }
    fn flush_buffer(&mut self) {
        self.buffer.fill(0);
        self.sample_idx = 0;
    }
}
