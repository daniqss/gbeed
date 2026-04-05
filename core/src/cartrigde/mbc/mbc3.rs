use crate::{
    BOOT_ROM_END, BOOT_ROM_START, EXTERNAL_RAM_SIZE, EXTERNAL_RAM_START, ROM_BANK00_END, ROM_BANK00_START,
    ROM_BANKNN_END, ROM_BANKNN_SIZE, ROM_BANKNN_START,
    cartrigde::{
        CartridgeError, CartridgeResult, RamSize, RomSize,
        features::{CartridgeFeatures, Rtc},
        header::CartridgeHeader,
    },
    prelude::*,
};

use super::MemoryBankController;

mem_range!(RAM_AND_TIMER_ENABLE, 0x0000, 0x1FFF);
mem_range!(ROM_BANK_NUMBER, 0x2000, 0x3FFF);
mem_range!(RAM_BANK_NUMBER_OR_RTC_SELECT, 0x4000, 0x5FFF);
mem_range!(LATCH_CLOCK_DATA, 0x6000, 0x7FFF);

#[derive(Debug, Default)]
pub struct Mbc3 {
    rom: Box<[u8]>,
    rom_size: RomSize,
    rom_selected_bank: u8,
    ram: Option<Box<[u8]>>,
    ram_enabled: bool,
    ram_size: RamSize,
    ram_selected_bank: u8,

    timer: Option<Rtc>,
    rtc_selected: bool,
    rtc_latch_state: u8,
}

impl MemoryBankController for Mbc3 {
    fn new(
        raw_rom: &[u8],
        save: Option<Vec<u8>>,
        features: &CartridgeFeatures,
        header: &CartridgeHeader,
    ) -> CartridgeResult<Self> {
        let rom: Box<[u8]> = if raw_rom.len() == header.rom_size.get_size() as usize {
            raw_rom.to_vec().into_boxed_slice()
        } else {
            return Err(CartridgeError::InvalidRomSize(
                Some(header.rom_size),
                "ROM size does not match the expected size for the cartridge",
            ));
        };

        let ram: Option<Box<[u8]>> = features.has_ram.then(|| {
            let ram_size = header.ram_size.get_size() as usize;
            save.filter(|s| features.has_battery && s.len() == ram_size)
                .unwrap_or_else(|| vec![0; ram_size])
                .into_boxed_slice()
        });

        let timer = if features.has_timer {
            Some(Rtc::new())
        } else {
            None
        };

        Ok(Self {
            rom,
            rom_size: header.rom_size,
            rom_selected_bank: 1,
            ram,
            ram_enabled: false,
            ram_size: header.ram_size,
            ram_selected_bank: 0,
            timer,
            rtc_selected: false,
            rtc_latch_state: 0xFF,
        })
    }

    fn read_rom(&self, address: u16) -> u8 {
        match address {
            ROM_BANK00_START..=ROM_BANK00_END => self.rom[address as usize],
            ROM_BANKNN_START..=ROM_BANKNN_END => {
                let bank = self.rom_selected_bank.max(1);

                let bank = bank % self.rom_size.get_banks_count() as u8;
                let offset =
                    (bank as usize * ROM_BANKNN_SIZE as usize) + (address - ROM_BANKNN_START) as usize;
                self.rom.get(offset).copied().unwrap_or(0xFF)
            }

            _ => unreachable!(
                "MBC3: ROM read at address: {:#04X} should be handle by other components",
                address
            ),
        }
    }
    fn write_rom(&mut self, address: u16, value: u8) {
        match address {
            RAM_AND_TIMER_ENABLE_START..=RAM_AND_TIMER_ENABLE_END => {
                let enabled = (value & 0x0F) == 0x0A;

                if self.ram.is_some() {
                    self.ram_enabled = enabled;
                }

                if let Some(timer) = &mut self.timer {
                    timer.enabled = enabled;
                }
            }

            ROM_BANK_NUMBER_START..=ROM_BANK_NUMBER_END => {
                self.rom_selected_bank = value & 0x7F;
            }

            RAM_BANK_NUMBER_OR_RTC_SELECT_START..=RAM_BANK_NUMBER_OR_RTC_SELECT_END => {
                if value <= 0x03 {
                    self.ram_selected_bank = value;
                    self.rtc_selected = false;
                } else if (0x08..=0x0C).contains(&value)
                    && let Some(timer) = &mut self.timer
                {
                    timer.select_register(value);
                    self.rtc_selected = true;
                }
            }

            LATCH_CLOCK_DATA_START..=LATCH_CLOCK_DATA_END => {
                if let Some(timer) = &mut self.timer {
                    if self.rtc_latch_state == 0x00 && value == 0x01 {
                        timer.latch();
                    }
                    self.rtc_latch_state = value;
                }
            }

            _ => unreachable!(
                "MBC3: ROM write at address: {:#04X} should be handle by other components",
                address
            ),
        }
    }

    fn read_ram(&self, address: u16) -> u8 {
        if !self.ram_enabled {
            return 0xFF;
        }

        if self.rtc_selected {
            if let Some(timer) = &self.timer {
                return timer.read();
            }
        } else if let Some(ram) = &self.ram
            && !ram.is_empty()
        {
            let banks_count = self.ram_size.get_banks_count().unwrap_or(0) as usize;
            if banks_count > 0 {
                let bank = (self.ram_selected_bank as usize) % banks_count;
                let offset = (bank * EXTERNAL_RAM_SIZE as usize) + (address - EXTERNAL_RAM_START) as usize;
                return ram[offset];
            }
        }

        0xFF
    }

    fn write_ram(&mut self, address: u16, value: u8) {
        if !self.ram_enabled {
            return;
        }

        if self.rtc_selected {
            if let Some(timer) = &mut self.timer {
                timer.write(value);
            }
        } else if let Some(ram) = &mut self.ram
            && !ram.is_empty()
        {
            let banks_count = self.ram_size.get_banks_count().unwrap_or(0) as usize;
            if banks_count > 0 {
                let bank = (self.ram_selected_bank as usize) % banks_count;
                let offset = (bank * EXTERNAL_RAM_SIZE as usize) + (address - EXTERNAL_RAM_START) as usize;

                ram[offset] = value;
            }
        }
    }

    fn get_ram(&self) -> Option<&[u8]> { self.ram.as_deref() }
    fn swap_boot_rom(&mut self, boot_rom: &mut [u8]) {
        let rom_slice = &mut self.rom[BOOT_ROM_START as usize..=BOOT_ROM_END as usize];
        let boot_rom_slice = &mut boot_rom[..=(BOOT_ROM_END - BOOT_ROM_START) as usize];
        rom_slice.swap_with_slice(boot_rom_slice);
    }
}
