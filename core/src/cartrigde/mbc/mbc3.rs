use crate::cartrigde::{
    CartridgeResult, RamSize, RomSize,
    mbc::{CartridgeType, MbcFeatures},
};

use super::MemoryBankController;

#[derive(Debug, Default)]
pub struct Mbc3 {
    features: MbcFeatures,
}

impl MemoryBankController for Mbc3 {
    fn new(
        raw_rom: &[u8],
        cartridge_type: CartridgeType,
        rom_type: RomSize,
        ram_type: RamSize,
    ) -> CartridgeResult<Self> {
        Ok(Self {
            features: MbcFeatures::from(cartridge_type),
        })
    }

    fn read_rom(&self, address: u16) -> u8 { todo!() }
    fn write_rom(&mut self, address: u16, value: u8) { todo!() }
    fn read_ram(&self, address: u16) -> u8 { todo!() }
    fn write_ram(&mut self, address: u16, value: u8) { todo!() }
}
