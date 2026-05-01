use gbeed_core::{
    AudioPlayer, Controller, DefaultAudioPlayer, DefaultRenderer, Ppu, Renderer, SerialListener, prelude::*,
};
use std::{fs, path::Path};

type Result<T = ()> = std::result::Result<T, Box<dyn std::error::Error>>;

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
        // print!("{}", data as char);

        self.received_data.push(data as char);

        if self.received_data.len() == self.rom_name.len() {
            assert_eq!(
                self.received_data[..self.rom_name.len()],
                self.rom_name[..],
                "ROM name mismatch: expected '{}', got '{}'",
                self.rom_name.iter().collect::<String>(),
                self.received_data.iter().collect::<String>()
            );

        // println!("ROM name received: {}", self.rom_name.iter().collect::<String>());
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

controller!(
    BlarggController,
    BlarggListener,
    DefaultRenderer,
    DefaultAudioPlayer
);

fn run_dmg_sound_test(rom_dir: &str, rom_name: &str) -> Result<()> {
    const STATUS_ADDR: u16 = 0xA000;
    const SIG_ADDR: u16 = 0xA001;
    const TEXT_ADDR: u16 = 0xA004;
    const RUNNING: u8 = 0x80;
    // one minute at 60 fps
    const TIMEOUT_FRAMES: u32 = 3600;

    let rom_path = format!("{}/{}", rom_dir, rom_name);
    let rom = fs::read(Path::new(&rom_path)).expect("Failed to read ROM file");
    let cartridge = Cartridge::new(&rom, None).map_err(|e| format!("Failed to create cartridge: {e}"))?;
    let mut controller = BlarggController {
        listener: BlarggListener::new(rom_name),
        renderer: DefaultRenderer::new(),
        audio_player: DefaultAudioPlayer::new(),
    };
    let mut gb = Dmg::new(cartridge, None);

    for frame in 0..TIMEOUT_FRAMES {
        gb.run(&mut controller)?;

        let sig = [gb.read(SIG_ADDR), gb.read(SIG_ADDR + 1), gb.read(SIG_ADDR + 2)];
        if sig != [0xDE, 0xB0, 0x61] {
            continue;
        }

        let status = gb.read(STATUS_ADDR);
        if status == RUNNING {
            continue;
        }

        let mut text = String::new();
        let mut addr = TEXT_ADDR;
        loop {
            let c = gb.read(addr);
            if c == 0 {
                break;
            }
            text.push(c as char);
            addr = addr.wrapping_add(1);
        }

        assert_eq!(
            status, 0,
            "Test failed (code {}) after {} frames:\n{}",
            status, frame, text
        );

        return Ok(());
    }

    panic!("Test timed out after {} frames", TIMEOUT_FRAMES);
}

fn run_blargg_test(rom_dir: &str, rom_name: &str) -> Result<()> {
    let rom_path = format!("{}/{}", rom_dir, rom_name);

    let rom = fs::read(Path::new(&rom_path)).expect("Failed to read ROM file");
    let cartridge = Cartridge::new(&rom, None).map_err(|e| format!("Failed to create cartridge: {e}"))?;
    let listener = BlarggListener::new(rom_name);
    let mut controller = BlarggController {
        listener,
        renderer: DefaultRenderer::new(),
        audio_player: DefaultAudioPlayer::new(),
    };
    let mut gb = Dmg::new(cartridge, None);

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
    fn special() -> Result<()> { run_blargg_test(CPU_INSTRS_DIR, "01-special.gb") }

    #[test]
    fn interrupts() -> Result<()> { run_blargg_test(CPU_INSTRS_DIR, "02-interrupts.gb") }

    #[test]
    fn op_sp_hl() -> Result<()> { run_blargg_test(CPU_INSTRS_DIR, "03-op sp,hl.gb") }

    #[test]
    fn op_r_imm() -> Result<()> { run_blargg_test(CPU_INSTRS_DIR, "04-op r,imm.gb") }

    #[test]
    fn op_rp() -> Result<()> { run_blargg_test(CPU_INSTRS_DIR, "05-op rp.gb") }

    #[test]
    fn ld_r_r() -> Result<()> { run_blargg_test(CPU_INSTRS_DIR, "06-ld r,r.gb") }

    #[test]
    fn jr() -> Result<()> { run_blargg_test(CPU_INSTRS_DIR, "07-jr,jp,call,ret,rst.gb") }

    #[test]
    fn misc_instrs() -> Result<()> { run_blargg_test(CPU_INSTRS_DIR, "08-misc instrs.gb") }

    #[test]
    fn op_r_r() -> Result<()> { run_blargg_test(CPU_INSTRS_DIR, "09-op r,r.gb") }

    #[test]
    fn bit_ops() -> Result<()> { run_blargg_test(CPU_INSTRS_DIR, "10-bit ops.gb") }

    #[test]
    fn op_a_hl() -> Result<()> { run_blargg_test(CPU_INSTRS_DIR, "11-op a,(hl).gb") }
}

#[cfg(test)]
mod instr_timing {
    use super::*;

    const INSTR_TIMINGS_DIR: &str = "../gb-test-roms/instr_timing/";

    #[test]
    fn instr_timing() -> Result<()> { run_blargg_test(INSTR_TIMINGS_DIR, "instr_timing.gb") }
}

#[cfg(test)]
mod mem_timing_2 {
    use super::*;

    const MEM_TIMING_DIR_2: &str = "../gb-test-roms/mem_timing-2/rom_singles/";

    #[ignore]
    #[test]
    fn read_timing2() -> Result<()> { run_blargg_test(MEM_TIMING_DIR_2, "01-read_timing.gb") }

    #[ignore]
    #[test]
    fn write_timing2() -> Result<()> { run_blargg_test(MEM_TIMING_DIR_2, "02-write_timing.gb") }

    #[ignore]
    #[test]
    fn modify_timing2() -> Result<()> { run_blargg_test(MEM_TIMING_DIR_2, "03-modify_timing.gb") }
}

#[cfg(test)]
mod mem_timing {
    use super::*;

    const MEM_TIMING_DIR: &str = "../gb-test-roms/mem_timing/individual/";

    #[ignore]
    #[test]
    fn read_timing() -> Result<()> { run_blargg_test(MEM_TIMING_DIR, "01-read_timing.gb") }

    #[ignore]
    #[test]
    fn write_timing() -> Result<()> { run_blargg_test(MEM_TIMING_DIR, "02-write_timing.gb") }

    #[ignore]
    #[test]
    fn modify_timing() -> Result<()> { run_blargg_test(MEM_TIMING_DIR, "03-modify_timing.gb") }
}

#[cfg(test)]
mod dmg_sound {
    use super::*;

    const DMG_SOUND_DIR: &str = "../gb-test-roms/dmg_sound/rom_singles";

    #[test]
    fn registers() -> Result<()> { run_dmg_sound_test(DMG_SOUND_DIR, "01-registers.gb") }

    #[test]
    fn len_ctr() -> Result<()> { run_dmg_sound_test(DMG_SOUND_DIR, "02-len ctr.gb") }

    #[test]
    fn trigger() -> Result<()> { run_dmg_sound_test(DMG_SOUND_DIR, "03-trigger.gb") }

    #[test]
    fn sweep() -> Result<()> { run_dmg_sound_test(DMG_SOUND_DIR, "04-sweep.gb") }

    #[test]
    fn sweep_details() -> Result<()> { run_dmg_sound_test(DMG_SOUND_DIR, "05-sweep details.gb") }

    #[test]
    fn overflow_on_trigger() -> Result<()> { run_dmg_sound_test(DMG_SOUND_DIR, "06-overflow on trigger.gb") }

    #[test]
    fn len_sweep_period_sync() -> Result<()> {
        run_dmg_sound_test(DMG_SOUND_DIR, "07-len sweep period sync.gb")
    }

    #[test]
    fn len_ctr_during_power() -> Result<()> {
        run_dmg_sound_test(DMG_SOUND_DIR, "08-len ctr during power.gb")
    }

    #[test]
    fn wave_read_while_on() -> Result<()> { run_dmg_sound_test(DMG_SOUND_DIR, "09-wave read while on.gb") }

    #[test]
    fn wave_trigger_while_on() -> Result<()> {
        run_dmg_sound_test(DMG_SOUND_DIR, "10-wave trigger while on.gb")
    }

    #[test]
    fn regs_after_power() -> Result<()> { run_dmg_sound_test(DMG_SOUND_DIR, "11-regs after power.gb") }

    #[test]
    fn wave_write_while_on() -> Result<()> { run_dmg_sound_test(DMG_SOUND_DIR, "12-wave write while on.gb") }
}
