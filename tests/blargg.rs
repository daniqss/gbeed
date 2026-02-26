use gbeed::{Cartridge, Dmg, SerialListener, prelude::*};
use std::{fs, path::Path, rc::Rc};

struct BlarggListener {
    rom_name: Vec<char>,
    separator: Vec<char>,
    passed_line: Vec<char>,
    received_data: Vec<char>,

    test_passed: bool,
}

impl BlarggListener {
    fn new(rom_name: &str) -> Self {
        Self {
            rom_name: rom_name
                .chars()
                .take_while(|c| *c != '.')
                .collect::<Vec<char>>()
                .into_iter()
                .chain(vec!['\n'].into_iter())
                .collect(),
            separator: "\n\n".chars().collect(),
            passed_line: "Passed\n".chars().collect(),
            received_data: Vec::new(),
            test_passed: false,
        }
    }
}

impl SerialListener for BlarggListener {
    fn on_transfer(&mut self, data: u8) {
        self.received_data.push(data as char);

        if self.received_data.len() == self.rom_name.len() {
            assert_eq!(
                self.received_data[..self.rom_name.len()],
                self.rom_name[..],
                "ROM name mismatch: expected '{}', got '{}'",
                self.rom_name.iter().collect::<String>(),
                self.received_data.iter().collect::<String>()
            );

            println!("ROM name received: {}", self.rom_name.iter().collect::<String>());
        } else if self.received_data.len() == self.rom_name.len() + self.separator.len() {
            let separator_start = self.rom_name.len();
            let separator_end = separator_start + self.separator.len();

            assert_eq!(
                self.received_data[separator_start..separator_end],
                self.separator[..],
                "Separator mismatch: expected '{}', got '{}'",
                self.separator.iter().collect::<String>(),
                &self.received_data.iter().collect::<String>()
            );
        } else if self.received_data.len()
            >= self.rom_name.len() + self.separator.len() + self.passed_line.len()
        {
            let passed_line_start = self.rom_name.len() + self.separator.len();
            let passed_line_end = passed_line_start + self.passed_line.len();

            assert_eq!(
                self.received_data[passed_line_start..passed_line_end],
                self.passed_line[..],
                "Test did not pass: expected '{}', got '{}'",
                self.passed_line.iter().collect::<String>(),
                &self.received_data.iter().collect::<String>()
            );

            self.test_passed = true;
        }
    }
}

fn run_blargg_test(dir_path: &str, rom_name: &str) -> Result<()> {
    let rom_path = format!("{}/{}", dir_path, rom_name);

    let rom = fs::read(Path::new(&rom_path)).expect("Failed to read ROM file");
    let cartridge = Cartridge::new(rom);
    let mut gb = Dmg::new(cartridge, None);

    let listener = Rc::new(RefCell::new(BlarggListener::new(rom_name)));
    gb.serial.set_serial_listener(listener.clone());

    // should be enough for the all tests in cpu_instrs/individual at least
    let timeout_cycles = 1_000_000;
    let mut cycles = 0;

    while !listener.borrow().test_passed && cycles < timeout_cycles {
        gb.run()?;
        cycles += gb.cpu.cycles;
    }

    assert!(
        listener.borrow().test_passed,
        "Test did not pass within {} cycles",
        timeout_cycles
    );

    Ok(())
}

#[cfg(test)]
mod cpu_instrs {
    use super::*;

    const DIR_PATH: &str = "gb-test-roms/cpu_instrs/individual";

    #[test]
    fn test_01_special() -> Result<()> { run_blargg_test(DIR_PATH, "01-special.gb") }

    // #[test]
    // fn test_02_interrupts() -> Result<()> { run_blargg_test(DIR_PATH, "02-interrupts.gb") }

    #[test]
    fn test_03_op_sp_hl() -> Result<()> { run_blargg_test(DIR_PATH, "03-op sp,hl.gb") }

    #[test]
    fn test_04_op_r_imm() -> Result<()> { run_blargg_test(DIR_PATH, "04-op r,imm.gb") }

    #[test]
    fn test_05_op_rp() -> Result<()> { run_blargg_test(DIR_PATH, "05-op rp.gb") }

    #[test]
    fn test_06_ld_r_r() -> Result<()> { run_blargg_test(DIR_PATH, "06-ld r,r.gb") }

    #[test]
    fn test_07_jr() -> Result<()> { run_blargg_test(DIR_PATH, "07-jr,jp,call,ret,rst.gb") }

    #[test]
    fn test_08_misc_instrs() -> Result<()> { run_blargg_test(DIR_PATH, "08-misc instrs.gb") }

    #[test]
    fn test_09_op_r_r() -> Result<()> { run_blargg_test(DIR_PATH, "09-op r,r.gb") }

    #[test]
    fn test_10_bit_ops() -> Result<()> { run_blargg_test(DIR_PATH, "10-bit ops.gb") }

    #[test]
    fn test_11_op_a_hl() -> Result<()> { run_blargg_test(DIR_PATH, "11-op a,(hl).gb") }
}
