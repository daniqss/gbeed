use crate::cartrigde::{CartridgeResult, header::CartridgeHeader, mbc::MbcFeatures};

use super::MemoryBankController;

#[derive(Debug, Default)]
pub struct Mbc2 {
    features: MbcFeatures,
}

impl MemoryBankController for Mbc2 {
    fn new(raw_rom: &[u8], header: &CartridgeHeader) -> CartridgeResult<Self> {
        Ok(Self {
            features: MbcFeatures::new(&header.cartridge_type),
        })
    }

    fn read_rom(&self, address: u16) -> u8 { todo!() }
    fn write_rom(&mut self, address: u16, value: u8) { todo!() }
    fn read_ram(&self, address: u16) -> u8 { todo!() }
    fn write_ram(&mut self, address: u16, value: u8) { todo!() }
}
