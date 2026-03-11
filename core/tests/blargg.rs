use gbeed_core::{prelude::*, Controller, DefaultRenderer, Renderer, SerialListener};
use std::{fs, path::Path};

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
                .chain(vec!['\n'])
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
        print!("{}", data as char);

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

controller!(BlarggController, BlarggListener, DefaultRenderer);

fn run_blargg_test(rom_dir: &str, rom_name: &str) -> Result<()> {
    let rom_path = format!("{}/{}", rom_dir, rom_name);

    let rom = fs::read(Path::new(&rom_path)).expect("Failed to read ROM file");
    let cartridge = Cartridge::new(rom);
    let listener = BlarggListener::new(rom_name);
    let mut controller = BlarggController {
        listener,
        renderer: DefaultRenderer::new(),
    };
    let mut gb = Dmg::new(cartridge, None);

    // should be enough for the all tests in cpu_instrs/individual at least
    let timeout_cycles = 100_000;
    let mut cycles = 0;

    while !controller.listener.test_passed && cycles < timeout_cycles {
        gb.run(&mut controller)?;
        cycles += gb.cpu.cycles;
    }

    assert!(
        controller.listener.test_passed,
        "Test did not pass within {} cycles",
        timeout_cycles
    );

    Ok(())
}

#[cfg(test)]
mod cpu_instrs {
    use super::*;

    const CPU_INSTRS_DIR: &str = "../gb-test-roms/cpu_instrs/individual";

    #[test]
    fn test_01_special() -> Result<()> { run_blargg_test(CPU_INSTRS_DIR, "01-special.gb") }

    #[test]
    fn test_02_interrupts() -> Result<()> { run_blargg_test(CPU_INSTRS_DIR, "02-interrupts.gb") }

    #[test]
    fn test_03_op_sp_hl() -> Result<()> { run_blargg_test(CPU_INSTRS_DIR, "03-op sp,hl.gb") }

    #[test]
    fn test_04_op_r_imm() -> Result<()> { run_blargg_test(CPU_INSTRS_DIR, "04-op r,imm.gb") }

    #[test]
    fn test_05_op_rp() -> Result<()> { run_blargg_test(CPU_INSTRS_DIR, "05-op rp.gb") }

    #[test]
    fn test_06_ld_r_r() -> Result<()> { run_blargg_test(CPU_INSTRS_DIR, "06-ld r,r.gb") }

    #[test]
    fn test_07_jr() -> Result<()> { run_blargg_test(CPU_INSTRS_DIR, "07-jr,jp,call,ret,rst.gb") }

    #[test]
    fn test_08_misc_instrs() -> Result<()> { run_blargg_test(CPU_INSTRS_DIR, "08-misc instrs.gb") }

    #[test]
    fn test_09_op_r_r() -> Result<()> { run_blargg_test(CPU_INSTRS_DIR, "09-op r,r.gb") }

    #[test]
    fn test_10_bit_ops() -> Result<()> { run_blargg_test(CPU_INSTRS_DIR, "10-bit ops.gb") }

    #[test]
    fn test_11_op_a_hl() -> Result<()> { run_blargg_test(CPU_INSTRS_DIR, "11-op a,(hl).gb") }
}

#[cfg(test)]
mod instr_timing {
    use super::*;

    const INSTR_TIMINGS_DIR: &str = "../gb-test-roms/instr_timing/";

    #[test]
    fn test_instr_timing() -> Result<()> { run_blargg_test(INSTR_TIMINGS_DIR, "instr_timing.gb") }
}

#[cfg(test)]
mod mem_timing_2 {
    use super::*;

    const MEM_TIMING_DIR_2: &str = "../gb-test-roms/mem_timing-2/rom_singles/";

    #[ignore]
    #[test]
    fn test_read_timing2() -> Result<()> { run_blargg_test(MEM_TIMING_DIR_2, "01-read_timing.gb") }

    #[ignore]
    #[test]
    fn test_write_timing2() -> Result<()> { run_blargg_test(MEM_TIMING_DIR_2, "02-write_timing.gb") }

    #[ignore]
    #[test]
    fn test_modify_timing2() -> Result<()> { run_blargg_test(MEM_TIMING_DIR_2, "03-modify_timing.gb") }
}

#[cfg(test)]
mod mem_timing {
    use super::*;

    const MEM_TIMING_DIR: &str = "../gb-test-roms/mem_timing/individual/";

    #[ignore]
    #[test]
    fn test_read_timing() -> Result<()> { run_blargg_test(MEM_TIMING_DIR, "01-read_timing.gb") }

    #[ignore]
    #[test]
    fn test_write_timing() -> Result<()> { run_blargg_test(MEM_TIMING_DIR, "02-write_timing.gb") }

    #[ignore]
    #[test]
    fn test_modify_timing() -> Result<()> { run_blargg_test(MEM_TIMING_DIR, "03-modify_timing.gb") }
}
