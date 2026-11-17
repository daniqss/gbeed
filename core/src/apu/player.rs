#[cfg_attr(feature = "ffi", gbeed_macros::ffi_callbacks)]
pub trait AudioPlayer {
    fn playing_stereo(&self) -> bool { false }
    /// Called for every sample generated, at `GB_SAMPLE_RATE`
    fn push_sample(&mut self, _left: i16, _right: i16) {}
    /// Called at the end of every frame
    fn flush_buffer(&mut self) {}
}
