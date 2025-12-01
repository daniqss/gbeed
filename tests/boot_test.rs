use gbeed::Dmg;
use gbeed::prelude::*;

#[test]
fn test_disassembly_boot() -> Result<()> {
    let boot_rom = include_bytes!("../dmg_rom.bin").to_vec();
    let mut gb = Dmg::new(None, Some(boot_rom));

    loop {
        gb.run();
        if gb.cpu.pc >= 0x0100 {
            break;
        }

        // println!("");
        // std::thread::sleep(std::time::Duration::from_millis(16));

        // finish initing ram
        if gb.cpu.pc == 0x000C {
            break;
        }
    }
    println!("Boot ROM disassembly complete.");
    #[cfg(debug_assertions)]
    println!("CPU State after execution: {}", gb.cpu);
    assert_eq!(gb.cpu.pc, 0x0100);
    Ok(())
}
