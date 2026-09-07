pub trait AudioPlayer {
    fn playing_stereo(&self) -> bool { false }
    fn push_sample(&mut self, _left: i16, _right: i16) {}
    fn flush_buffer(&mut self) {}
}
