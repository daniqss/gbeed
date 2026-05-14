use criterion::{Criterion, criterion_group, criterion_main};
use gbeed_core::{
    AudioPlayer, Controller, DefaultAudioPlayer, DefaultRenderer, Ppu, Renderer, SerialListener, prelude::*,
};
use std::{fs, hint::black_box, path::PathBuf, time::Duration};

struct PassedListener {
    tail: [u8; 7],
    len: usize,
    passed: bool,
}

impl PassedListener {
    fn new() -> Self {
        Self {
            tail: [0; 7],
            len: 0,
            passed: false,
        }
    }
}

impl SerialListener for PassedListener {
    fn on_transfer(&mut self, data: u8) {
        if self.len < self.tail.len() {
            self.tail[self.len] = data;
            self.len += 1;
        } else {
            self.tail.rotate_left(1);
            self.tail[self.tail.len() - 1] = data;
        }
        if self.len == self.tail.len() && &self.tail == b"Passed\n" {
            self.passed = true;
        }
    }
}

controller!(
    BenchController,
    PassedListener,
    DefaultRenderer,
    DefaultAudioPlayer
);

fn make_controller() -> BenchController {
    BenchController {
        listener: PassedListener::new(),
        renderer: DefaultRenderer::new(),
        audio_player: DefaultAudioPlayer::new(),
    }
}

fn rom_path(rel: &str) -> PathBuf {
    let base = std::env::var("GB_TEST_ROMS_DIR")
        .unwrap_or_else(|_| format!("{}/../gb-test-roms", env!("CARGO_MANIFEST_DIR")));
    PathBuf::from(base).join(rel)
}

fn load_rom(rel: &str) -> Vec<u8> {
    let p = rom_path(rel);
    fs::read(&p).unwrap_or_else(|e| {
        panic!(
            "Failed to read benchmark ROM at {}: {}. Run `just fetch-test-roms` first.",
            p.display(),
            e
        )
    })
}

// Safety net so a regression that never signals Passed can't hang the bench forever.
const FRAME_TIMEOUT: u32 = 7200;

fn run_until_passed(rom_bytes: &[u8]) {
    let cartridge = Cartridge::new(rom_bytes, None).expect("cartridge");
    let mut gb = Dmg::new(cartridge, None);
    let mut controller = make_controller();

    let mut frames = 0u32;
    while !controller.listener.passed {
        gb.run(&mut controller).expect("emulator run failed");
        frames += 1;
        if frames >= FRAME_TIMEOUT {
            panic!("ROM did not signal Passed within {FRAME_TIMEOUT} frames");
        }
    }
    black_box(&gb);
}

fn bench_cpu_instrs(c: &mut Criterion) {
    let mut group = c.benchmark_group("cpu_instrs");
    group.sample_size(10);
    group.measurement_time(Duration::from_secs(20));

    let roms = [
        ("01_special", "cpu_instrs/individual/01-special.gb"),
        ("02_interrupts", "cpu_instrs/individual/02-interrupts.gb"),
        ("03_op_sp_hl", "cpu_instrs/individual/03-op sp,hl.gb"),
        ("04_op_r_imm", "cpu_instrs/individual/04-op r,imm.gb"),
        ("05_op_rp", "cpu_instrs/individual/05-op rp.gb"),
        ("06_ld_r_r", "cpu_instrs/individual/06-ld r,r.gb"),
        (
            "07_jr_jp_call_ret_rst",
            "cpu_instrs/individual/07-jr,jp,call,ret,rst.gb",
        ),
        ("08_misc_instrs", "cpu_instrs/individual/08-misc instrs.gb"),
        ("09_op_r_r", "cpu_instrs/individual/09-op r,r.gb"),
        ("10_bit_ops", "cpu_instrs/individual/10-bit ops.gb"),
        ("11_op_a_hl", "cpu_instrs/individual/11-op a,(hl).gb"),
    ];

    for (name, rel) in roms {
        let rom = load_rom(rel);
        group.bench_function(name, |b| {
            b.iter(|| run_until_passed(black_box(&rom)));
        });
    }
    group.finish();
}

fn bench_misc(c: &mut Criterion) {
    let mut group = c.benchmark_group("misc");
    group.sample_size(10);
    group.measurement_time(Duration::from_secs(20));

    let rom = load_rom("instr_timing/instr_timing.gb");
    group.bench_function("instr_timing", |b| {
        b.iter(|| run_until_passed(black_box(&rom)));
    });
    group.finish();
}

criterion_group!(benches, bench_cpu_instrs, bench_misc);
criterion_main!(benches);
