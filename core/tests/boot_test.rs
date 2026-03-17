use gbeed_core::{Cpu, DefaultController, ROM_BANK00_START, prelude::*};

// copied from cartridge/mod.rs to not expose it in the library
const NINTENDO_LOGO: [u8; 48] = [
    0xCE, 0xED, 0x66, 0x66, 0xCC, 0x0D, 0x00, 0x0B, 0x03, 0x73, 0x00, 0x83, 0x00, 0x0C, 0x00, 0x0D, 0x00,
    0x08, 0x11, 0x1F, 0x88, 0x89, 0x00, 0x0E, 0xDC, 0xCC, 0x6E, 0xE6, 0xDD, 0xDD, 0xD9, 0x99, 0xBB, 0xBB,
    0x67, 0x63, 0x6E, 0x0E, 0xEC, 0xCC, 0xDD, 0xDC, 0x99, 0x9F, 0xBB, 0xB9, 0x33, 0x3E,
];
mem_range!(CARTRIDGE_LOGO, 0x0104, 0x0104 + NINTENDO_LOGO.len() as u16 - 1);

#[test]
fn test_disassembly_boot() -> Result<(), Box<dyn std::error::Error>> {
    let boot_rom_data = std::fs::read("../dmg_boot.bin")?;
    let game_data = (ROM_BANK00_START..ROM_BANK00_START + 0x8000)
        .map(|addr| {
            if (CARTRIDGE_LOGO_START..=CARTRIDGE_LOGO_END).contains(&addr) {
                NINTENDO_LOGO[(addr - CARTRIDGE_LOGO_START) as usize]
            } else {
                0
            }
        })
        .collect::<Vec<u8>>();

    let game = Cartridge::new(&game_data, None).map_err(|e| format!("Failed to create cartridge: {e}"))?;
    // it actually needs a game to compare the logos
    let mut gb = Dmg::new(game, Some(boot_rom_data));
    let mut init_ram = false;
    let mut set_audio = false;
    let mut setup_logo = false;

    let mut controller = DefaultController::new();

    while gb.cpu.pc < 0x0100 {
        let _instr = gb.step(&mut controller);

        if gb.cpu.cycles >= 70224 {
            gb.cpu.cycles = 0;
            gb.ppu.last_cycles = 0;
        }

        // finish initing ram
        if gb.cpu.pc == 0x000C && !init_ram {
            println!("Cpu after initing RAM: {}", gb.cpu);
            assert_eq!(gb.cpu.f, 0xA0);
            assert_eq!(gb.cpu.h, 0x7F);
            assert_eq!(gb.cpu.l, 0xFF);
            assert_eq!(gb.cpu.sp, 0xFFFE);
            assert_eq!(gb.cpu.cycles, 57350);

            init_ram = true;
        }

        // finish setting audio
        if gb.cpu.pc == 0x001D && !set_audio {
            println!("Cpu after setting audio: {}", gb.cpu);
            assert_eq!(gb.cpu.a, 0x77);
            assert_eq!(gb.cpu.c, 0x12);
            assert_eq!(gb.cpu.hl(), 0xFF24);
            assert_eq!(gb.read(gb.cpu.hl()), 0x77);
            assert_eq!(gb.cpu.cycles, 57372);

            set_audio = true;
        }

        // set up logo
        if gb.cpu.pc == 0x0053 && !setup_logo {
            println!("Cpu after setting up logo: {}", gb.cpu);
            assert_eq!(gb.cpu.af(), 0x0DC0);
            assert_eq!(gb.cpu.bc(), 0x0000);
            assert_eq!(gb.cpu.de(), 0x00E0);
            assert_eq!(gb.cpu.hl(), 0x990F);
            assert_eq!(gb.cpu.sp, 0xFFFE);
            assert_eq!(gb.cpu.cycles, 66482);

            setup_logo = true;
        }
    }

    // new used boot rom has different cpu state at the end the boot sequence
    if gb.cpu.pc == 0x0100 {
        assert_eq!(
            gb.cpu,
            Cpu {
                a: 1,
                f: 192,
                b: 0,
                c: 0,
                d: 0,
                e: 113,
                h: 129,
                l: 208,
                pc: 256,
                sp: 65534,
                cycles: 54462,
                ime: false,
                halted: false
            }
        );
        println!("Boot sequence completed successfully!");
        println!("Cpu after boot: {}", gb.cpu);
    }

    Ok(())
}
