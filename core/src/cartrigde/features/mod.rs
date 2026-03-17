mod rtc;
mod rumble;
mod sensor;

pub use rtc::Rtc;
pub use rumble::Rumble;
#[allow(unused_imports)]
pub use sensor::Sensor;

use crate::cartrigde::mbc::CartridgeType;

#[derive(Debug, Default)]
pub struct CartridgeFeatures {
    pub has_ram: bool,
    pub has_battery: bool,
    pub has_timer: bool,
    pub has_rumble: bool,
    pub has_sensor: bool,
}

impl CartridgeFeatures {
    pub fn new(cartridge_type: &CartridgeType) -> Self {
        CartridgeFeatures {
            has_ram: cartridge_type.has_ram(),
            has_battery: cartridge_type.has_battery(),
            has_timer: cartridge_type.has_timer(),
            has_rumble: cartridge_type.has_rumble(),
            has_sensor: cartridge_type.has_sensor(),
        }
    }

    pub fn supports_saves(&self) -> bool { self.has_ram && self.has_battery }
}
