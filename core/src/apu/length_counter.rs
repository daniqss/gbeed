#[derive(Debug, Clone)]
pub struct LengthCounter {
    pub counter: u16,
    pub max: u16,
}

impl Default for LengthCounter {
    fn default() -> Self { Self { counter: 64, max: 64 } }
}

impl LengthCounter {
    pub fn new(max: u16) -> Self { Self { counter: max, max } }

    /// Clock the length counter. Returns true if channel should be disabled.
    #[inline(always)]
    pub fn clock(&mut self) -> bool {
        if self.counter > 0 {
            self.counter -= 1;
            self.counter == 0
        } else {
            false
        }
    }

    /// Called on trigger: reload if frozen (counter == 0). Returns whether it was frozen.
    #[inline(always)]
    pub fn trigger_reload(&mut self) -> bool {
        let was_frozen = self.counter == 0;
        if was_frozen {
            self.counter = self.max;
        }
        was_frozen
    }

    #[inline(always)]
    pub fn clock_and_disable(length: &mut LengthCounter, enabled: &mut bool) {
        if length.clock() {
            *enabled = false;
        }
    }
}
