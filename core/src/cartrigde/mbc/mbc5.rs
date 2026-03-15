use crate::cartrigde::{
    CartridgeResult, RamSize, RomSize,
    features::{MbcFeatures, Rumble},
    header::CartridgeHeader,
    mbc::CartridgeType,
};

use super::MemoryBankController;

#[derive(Debug, Default)]
pub struct Mbc5 {
    features: MbcFeatures,
    rumble: Option<Rumble>,
}

impl MemoryBankController for Mbc5 {
    fn new(raw_rom: &[u8], header: &CartridgeHeader) -> CartridgeResult<Self> {
        Ok(Self {
            features: MbcFeatures::new(&header.cartridge_type),
            rumble: if header.cartridge_type.has_rumble() {
                Some(Rumble::new())
            } else {
                None
            },
        })
    }

    fn read_rom(&self, address: u16) -> u8 { todo!() }
    fn write_rom(&mut self, address: u16, value: u8) { todo!() }
    fn read_ram(&self, address: u16) -> u8 { todo!() }
    fn write_ram(&mut self, address: u16, value: u8) { todo!() }
}
