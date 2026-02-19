mod header;
mod license;
mod mbc;

use crate::{
    core::{
        Accessible, EXTERNAL_RAM_END, EXTERNAL_RAM_SIZE, EXTERNAL_RAM_START, ROM_BANK00_END, ROM_BANK00_SIZE,
        ROM_BANK00_START, ROM_BANKNN_END, ROM_BANKNN_SIZE, ROM_BANKNN_START,
    },
    prelude::*,
};

use header::CartridgeHeader;

#[derive(Debug)]
pub struct Cartridge {
    pub raw_rom: Vec<u8>,

    // header
    header: CartridgeHeader,

    // banks
    pub rom_bank00: Vec<u8>,
    rom_bank_nn: Vec<u8>,
    ram_bank: Vec<u8>,

    // checked in runtime to select currently used banks
    selected_rom_bank: u16,
    selected_ram_bank: u16,
    ram_enabled: bool,
    banking_mode: bool,
}

impl Default for Cartridge {
    fn default() -> Self { Cartridge::new(vec![0; (ROM_BANK00_SIZE + ROM_BANKNN_SIZE) as usize]) }
}

impl Cartridge {
    pub fn new(raw_rom: Vec<u8>) -> Self {
        let header = CartridgeHeader::new(&raw_rom);

        println!("Cartridge header: {header}");

        let rom_bank00: Vec<u8> = raw_rom
            .iter()
            .take(ROM_BANK00_SIZE as usize)
            .map(|&b| b)
            .collect();

        let rom_bank_nn: Vec<u8> = raw_rom
            .iter()
            .skip(ROM_BANK00_SIZE as usize)
            .take((header.rom_size - ROM_BANK00_SIZE as u32) as usize)
            .map(|&b| b)
            .collect();

        let ram_bank = vec![0; header.external_ram_size as usize];

        let cartridge = Self {
            header,
            raw_rom,
            rom_bank00,
            rom_bank_nn,
            ram_bank,
            selected_rom_bank: 1,
            selected_ram_bank: 0,
            ram_enabled: false,
            banking_mode: false,
        };

        #[cfg(not(test))]
        if let Err(e) = cartridge.check_header_checksum() {
            eprintln!(
                "Header checksum mismatch: the cartridge may be corrupted or the file may be malformed:\n{e}"
            );
        }

        cartridge
    }

    /// # Header checksum
    /// Checked by real hardware by the boot ROM
    pub fn check_header_checksum(&self) -> Result<()> {
        let header_sum = self.raw_rom[header::TITLE_START..=header::GAME_VERSION]
            .iter()
            .fold(0u8, |acc, &b| acc.wrapping_sub(b).wrapping_sub(1));

        match header_sum == self.header.header_checksum {
            true => Ok(()),
            false => Err(Error::Generic(format!(
                "Header checksum mismatch, {:#04X} != {:#04X}",
                header_sum, self.header.header_checksum
            ))),
        }
    }

    /// # Global checksum
    /// Not actually checked by real hardware
    /// We'll use in Cartridge creation for now to verify correct file parsing and integrity
    pub fn check_global_checksum(&self) -> Result<()> {
        let cartridge_sum: u16 = self.raw_rom.iter().enumerate().fold(0u16, |acc, (i, &b)| {
            match i != header::GLOBAL_CHECKSUM_START && i != header::GLOBAL_CHECKSUM_END {
                true => acc.wrapping_add(b as u16),
                false => acc,
            }
        });

        match cartridge_sum == self.header.global_checksum {
            true => Ok(()),
            false => Err(Error::Generic(format!(
                "Global checksum mismatch, {:#04X} != {:#04X}",
                cartridge_sum, self.header.global_checksum,
            ))),
        }
    }

    /// unmaps boot rom when boot reaches pc = 0x00FE, when load 1 in bank register (0xFF50)
    /// ```asm
    /// ld a, $01
    /// ld [0xFF50], a
    /// ```
    /// Next instruction will be the first `nop` in 0x0100, in the cartridge rom
    pub fn unmap_boot_rom(&mut self) {
        println!("Unmapping boot rom, switching to cartridge rom");
        self.rom_bank00
            .copy_from_slice(&self.raw_rom[..ROM_BANK00_SIZE as usize]);
    }
}

impl Accessible<u16> for Cartridge {
    fn read(&self, address: u16) -> u8 {
        match address {
            ROM_BANK00_START..=ROM_BANK00_END => self.rom_bank00[address as usize],
            ROM_BANKNN_START..=ROM_BANKNN_END => {
                let bank_offset = if self.header.rom_banks_count == 2 {
                    0
                } else {
                    (self.selected_rom_bank % self.header.rom_banks_count) as usize * ROM_BANKNN_SIZE as usize
                };
                self.rom_bank_nn[(address as usize - ROM_BANKNN_START as usize) + bank_offset]
            }
            EXTERNAL_RAM_START..=EXTERNAL_RAM_END if self.ram_enabled => {
                if let Some(bank_count) = self.header.external_ram_banks_count {
                    let bank_offset = (self.selected_ram_bank % bank_count) * EXTERNAL_RAM_SIZE;
                    self.ram_bank[(address as usize - EXTERNAL_RAM_START as usize) + bank_offset as usize]
                } else {
                    eprintln!(
                        "Cartrigde: Attempted to read from RAM at {address:04X} but cartridge has no RAM"
                    );
                    0xFF
                }
            }
            _ => {
                eprintln!("Cartrigde: Attempted to read from unmapped cartridge address: {address:04X}");
                0xFF
            }
        }
    }
    fn write(&mut self, address: u16, value: u8) {
        match address {
            ROM_BANK00_START..=0x1FFF => mbc::enable_ram(self, address, value),
            0x2000..=ROM_BANK00_END => mbc::select_rom_bank(self, address, value),
            ROM_BANKNN_START..=0x5FFF => mbc::select_ram_bank(self, value),
            0x6000..=ROM_BANKNN_END => mbc::select_banking_mode(self, value),

            EXTERNAL_RAM_START..=EXTERNAL_RAM_END if self.ram_enabled => {
                if let Some(bank_count) = self.header.external_ram_banks_count {
                    let bank_offset = (self.selected_ram_bank % bank_count) * EXTERNAL_RAM_SIZE;
                    self.ram_bank[(address as usize - EXTERNAL_RAM_START as usize) + bank_offset as usize] =
                        value;
                } else {
                    eprintln!(
                        "Cartrigde: Attempted to write to RAM at {address:04X} but cartridge has no RAM"
                    );
                }
            }

            _ => unreachable!(
                "Cartrigde: write of address {address:04X} should have been handled by other components"
            ),
        }
    }
}

impl std::fmt::Display for Cartridge {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result { writeln!(f, "{}", self.header) }
}
