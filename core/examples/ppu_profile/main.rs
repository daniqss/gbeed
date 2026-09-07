use gbeed_core::{AudioPlayer, Renderer, SerialListener, prelude::*};

struct ProfileController {
    framebuffer: [[u32; DMG_SCREEN_WIDTH]; DMG_SCREEN_HEIGHT],
    colors: [u32; 4],
}

impl ProfileController {
    fn new() -> Self {
        Self {
            framebuffer: [[0; DMG_SCREEN_WIDTH]; DMG_SCREEN_HEIGHT],
            colors: [0xC4CFA1, 0x8B956D, 0x4D533C, 0x1F1F1F],
        }
    }
}

impl Renderer for ProfileController {
    fn read_pixel(&self, x: usize, y: usize) -> u32 { self.framebuffer[y][x] }

    fn write_pixel(&mut self, x: usize, y: usize, palette: u8, color_id: u8) {
        let shade = (palette >> (color_id * 2)) & 0x03;
        self.framebuffer[y][x] = self.colors[shade as usize];
    }
}

impl SerialListener for ProfileController {}
impl AudioPlayer for ProfileController {}

/// Simple program that uses gbeed to make it easier to profile.
/// Not limited by frame generation, with cleaner flamegraph results because of not having system library calls
///
/// ## Usage
/// ```sh
/// export RUSTFLAGS="-Cforce-frame-pointers=yes -Cforce-unwind-tables=yes"
/// cargo flamegraph --profile bench --features "${DISPLAY_FEATURES}" --example ppu_profile <rom> [frames]
/// ```
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut args = std::env::args().skip(1);
    let rom_path = args.next().ok_or("usage: ppu_profile <rom> [frames]")?;
    let frames: usize = args.next().map(|f| f.parse()).transpose()?.unwrap_or(20_000);

    let rom = std::fs::read(&rom_path)?;
    let cartridge = Cartridge::new(&rom, None).map_err(|e| format!("Failed to create cartridge: {e}"))?;
    let mut gb = Dmg::new(cartridge, None);
    let mut controller = ProfileController::new();

    let start = std::time::Instant::now();
    for _ in 0..frames {
        gb.run(&mut controller)?;
    }
    let elapsed = start.elapsed();

    // fingerprint of the last frame with FNV-1a hash function
    // usefull to check that a change to the drawing loops does not alter the rendered output
    let mut hash: u64 = 0xcbf29ce484222325;
    for row in &controller.framebuffer {
        for pixel in row {
            hash ^= *pixel as u64;
            hash = hash.wrapping_mul(0x100000001b3);
        }
    }
    println!("framebuffer hash: {hash:016x}");

    println!(
        "{rom_path}: {frames} frames in {:.3} s ({:.1} fps, {:.2}x realtime)",
        elapsed.as_secs_f64(),
        frames as f64 / elapsed.as_secs_f64(),
        (frames as f64 / elapsed.as_secs_f64()) / 59.7275,
    );

    Ok(())
}
