use crate::{
    AudioPlayer, GB_SCREEN_HEIGHT, GB_SCREEN_WIDTH, GB_STEREO_AUDIO_BUFFER_SIZE, Ppu, Renderer,
    SerialListener,
};

pub(crate) const FRAMEBUFFER_SIZE: usize = GB_SCREEN_WIDTH * GB_SCREEN_HEIGHT;

/// Default controller of the C interface, keeps the frame and the audio
/// in buffers the host reads once `gbeed_run_frame` returns
pub(crate) struct BufferedController {
    /// frame being drawn, one shade (0..=3) per pixel
    back: [u8; FRAMEBUFFER_SIZE],
    /// last complete frame, so the host never sees a partially drawn one
    front: [u8; FRAMEBUFFER_SIZE],

    /// interleaved stereo samples
    audio: [i16; GB_STEREO_AUDIO_BUFFER_SIZE],
    audio_len: usize,
}

impl BufferedController {
    pub fn framebuffer(&self) -> &[u8; FRAMEBUFFER_SIZE] { &self.front }

    /// Returns the samples generated since the last call, emptying the buffer
    pub fn drain_audio(&mut self) -> &[i16] {
        let len = core::mem::take(&mut self.audio_len);
        &self.audio[..len]
    }
}

impl Default for BufferedController {
    fn default() -> Self {
        Self {
            back: [0; FRAMEBUFFER_SIZE],
            front: [0; FRAMEBUFFER_SIZE],
            audio: [0; GB_STEREO_AUDIO_BUFFER_SIZE],
            audio_len: 0,
        }
    }
}

impl Renderer for BufferedController {
    fn read_pixel(&self, x: usize, y: usize) -> u32 { self.back[y * GB_SCREEN_WIDTH + x] as u32 }

    fn write_pixel(&mut self, x: usize, y: usize, palette: u8, color_id: u8) {
        self.back[y * GB_SCREEN_WIDTH + x] = (palette >> (color_id * 2)) & 0x03;
    }

    fn update_screen(&mut self, _: &Ppu) { self.front.copy_from_slice(&self.back); }
}

impl AudioPlayer for BufferedController {
    fn playing_stereo(&self) -> bool { true }

    fn push_sample(&mut self, left: i16, right: i16) {
        // drop new samples when the buffer is full instead of wrapping
        if self.audio_len + 2 > GB_STEREO_AUDIO_BUFFER_SIZE {
            return;
        }

        self.audio[self.audio_len] = left;
        self.audio[self.audio_len + 1] = right;
        self.audio_len += 2;
    }
}

impl SerialListener for BufferedController {}
