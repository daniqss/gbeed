#[derive(Debug, Default, Clone)]
pub struct Envelope {
    pub volume: u8,
    pub timer: u8,
}

impl Envelope {
    pub fn tick(&mut self, envelope_register: u8, enabled: bool) {
        if !enabled {
            return;
        }

        if self.timer > 0 {
            self.timer -= 1;
        }
        if self.timer == 0 {
            let pace = envelope_register & 0x07;
            let direction = (envelope_register >> 3) & 0x01;

            if pace > 0 {
                if direction == 0 && self.volume > 0 {
                    self.volume -= 1;
                } else if direction == 1 && self.volume < 15 {
                    self.volume += 1;
                }
            }
            self.timer = if pace > 0 { pace } else { 8 };
        }
    }

    #[inline(always)]
    pub fn trigger(&mut self, envelope_register: u8) {
        self.volume = (envelope_register & 0xF0) >> 4;
        self.timer = envelope_register & 0x07;
    }

    #[inline(always)]
    pub fn clear(&mut self) {
        self.volume = 0;
        self.timer = 0;
    }
}
