use gbeed_core::{Controller, DefaultRenderer, Renderer, SerialListener, prelude::*};
use std::{fs, path::Path};

type Result<T = ()> = std::result::Result<T, Box<dyn std::error::Error>>;

struct MooneyeListener {
    received_data: Vec<u8>,
    test_passed: Option<bool>,
}

impl MooneyeListener {
    fn new() -> Self {
        Self {
            received_data: Vec::new(),
            test_passed: None,
        }
    }
}

impl SerialListener for MooneyeListener {
    fn on_transfer(&mut self, data: u8) {
        self.received_data.push(data);

        // fibonacci sequence
        let success_sequence = [3, 5, 8, 13, 21, 34];
        let failure_sequence = [0x42, 0x42, 0x42, 0x42, 0x42, 0x42];

        if self.received_data.len() >= 6 {
            let last_6 = &self.received_data[self.received_data.len() - 6..];
            if last_6 == success_sequence {
                self.test_passed = Some(true);
            } else if last_6 == failure_sequence {
                self.test_passed = Some(false);
            }
        }
    }
}

controller!(MooneyeController, MooneyeListener, DefaultRenderer);

fn run_mooneye_test(rom_dir: &str, rom_name: &str) -> Result<()> {
    let rom_path = format!("{}/{}", rom_dir, rom_name);

    let rom = fs::read(Path::new(&rom_path)).expect("Failed to read ROM file");
    let cartridge = Cartridge::new(&rom, None).map_err(|e| format!("Failed to create cartridge: {e}"))?;
    let listener = MooneyeListener::new();
    let mut controller = MooneyeController {
        listener,
        renderer: DefaultRenderer::new(),
    };
    let mut gb = Dmg::new(cartridge, None);

    let timeout_cycles = 100_000;
    let mut cycles = 0;

    println!("Running Mooneye test: {}", rom_name);
    while controller.listener.test_passed.is_none() && cycles < timeout_cycles {
        gb.run(&mut controller)?;
        cycles += gb.cpu.cycles;
    }

    match controller.listener.test_passed {
        Some(true) => Ok(()),
        Some(false) => panic!("Test {} FAILED", rom_name),
        None => panic!("Test {} TIMEOUT ({} cycles)", rom_name, cycles),
    }
}

#[cfg(test)]
mod mbc1 {
    use super::*;
    const MBC1_DIR: &str = "../mts-20240926-1737-443f6e1/emulator-only/mbc1";

    #[test]
    fn bits_bank1() -> Result<()> { run_mooneye_test(MBC1_DIR, "bits_bank1.gb") }

    #[test]
    fn bits_bank2() -> Result<()> { run_mooneye_test(MBC1_DIR, "bits_bank2.gb") }

    #[test]
    fn bits_mode() -> Result<()> { run_mooneye_test(MBC1_DIR, "bits_mode.gb") }

    #[test]
    fn bits_ramg() -> Result<()> { run_mooneye_test(MBC1_DIR, "bits_ramg.gb") }

    #[test]
    fn multicart_rom_8mb() -> Result<()> { run_mooneye_test(MBC1_DIR, "multicart_rom_8Mb.gb") }

    #[test]
    fn ram_256kb() -> Result<()> { run_mooneye_test(MBC1_DIR, "ram_256kb.gb") }

    #[test]
    fn ram_64kb() -> Result<()> { run_mooneye_test(MBC1_DIR, "ram_64kb.gb") }

    #[test]
    fn rom_16mb() -> Result<()> { run_mooneye_test(MBC1_DIR, "rom_16Mb.gb") }

    #[test]
    fn rom_1mb() -> Result<()> { run_mooneye_test(MBC1_DIR, "rom_1Mb.gb") }

    #[test]
    fn rom_2mb() -> Result<()> { run_mooneye_test(MBC1_DIR, "rom_2Mb.gb") }

    #[test]

    fn rom_4mb() -> Result<()> { run_mooneye_test(MBC1_DIR, "rom_4Mb.gb") }

    #[test]
    fn rom_512kb() -> Result<()> { run_mooneye_test(MBC1_DIR, "rom_512kb.gb") }

    #[test]
    fn rom_8mb() -> Result<()> { run_mooneye_test(MBC1_DIR, "rom_8Mb.gb") }
}

#[cfg(test)]
mod mbc2 {
    use super::*;
    const MBC2_DIR: &str = "../mts-20240926-1737-443f6e1/emulator-only/mbc2";

    #[test]
    fn bits_ramg() -> Result<()> { run_mooneye_test(MBC2_DIR, "bits_ramg.gb") }
    #[test]
    fn bits_romb() -> Result<()> { run_mooneye_test(MBC2_DIR, "bits_romb.gb") }
    #[test]
    fn bits_unused() -> Result<()> { run_mooneye_test(MBC2_DIR, "bits_unused.gb") }
    #[test]
    fn ram() -> Result<()> { run_mooneye_test(MBC2_DIR, "ram.gb") }
    #[test]
    fn rom_1mb() -> Result<()> { run_mooneye_test(MBC2_DIR, "rom_1Mb.gb") }
    #[test]
    fn rom_2mb() -> Result<()> { run_mooneye_test(MBC2_DIR, "rom_2Mb.gb") }
    #[test]
    fn rom_512kb() -> Result<()> { run_mooneye_test(MBC2_DIR, "rom_512kb.gb") }
}

#[cfg(test)]
mod mbc5 {
    use super::*;
    const MBC5_DIR: &str = "../mts-20240926-1737-443f6e1/emulator-only/mbc5";

    #[test]
    fn rom_16mb() -> Result<()> { run_mooneye_test(MBC5_DIR, "rom_16Mb.gb") }
    #[test]
    fn rom_1mb() -> Result<()> { run_mooneye_test(MBC5_DIR, "rom_1Mb.gb") }
    #[test]
    fn rom_2mb() -> Result<()> { run_mooneye_test(MBC5_DIR, "rom_2Mb.gb") }
    #[test]
    fn rom_32mb() -> Result<()> { run_mooneye_test(MBC5_DIR, "rom_32Mb.gb") }
    #[test]
    fn rom_4mb() -> Result<()> { run_mooneye_test(MBC5_DIR, "rom_4Mb.gb") }
    #[test]
    fn rom_512kb() -> Result<()> { run_mooneye_test(MBC5_DIR, "rom_512kb.gb") }
    #[test]
    fn rom_64mb() -> Result<()> { run_mooneye_test(MBC5_DIR, "rom_64Mb.gb") }
    #[test]
    fn rom_8mb() -> Result<()> { run_mooneye_test(MBC5_DIR, "rom_8Mb.gb") }
}
