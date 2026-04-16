use crate::apu::Apu;
pub use crate::prelude::*;
use crate::{
    Cartridge, Controller, Cpu, Interrupt, Joypad, Ppu, Serial, Timer,
    apu::{APU_REGISTER_END, APU_REGISTER_START},
    cpu::{Instruction, R8, R16},
    interrupts::{IE, IF},
    joypad::JOYP,
    memory::*,
    ppu::{DMA_REGISTER, PPU_REGISTER_END, PPU_REGISTER_START},
    serial::{SERIAL_REGISTER_END, SERIAL_REGISTER_START},
    timer::{TIMER_REGISTER_END, TIMER_REGISTER_START},
    utils::{high, low, to_u16},
};

const BANK_REGISTER: u16 = 0xFF50;

#[derive(Debug, Default)]
pub struct Dmg {
    pub cpu: Cpu,
    pub cartridge: Cartridge,
    pub memory: Memory,
    pub ppu: Ppu,
    pub joypad: Joypad,
    pub serial: Serial,
    pub timer: Timer,
    pub apu: Apu,
    pub interrupt_flag: Interrupt,
    pub interrupt_enable: Interrupt,
    pub bank: u8,
}

impl Dmg {
    pub fn new(mut cartridge: Cartridge, mut boot_rom: Option<Vec<u8>>) -> Dmg {
        // swap the boot rom with the cartridge data in the memory
        if let Some(boot) = &mut boot_rom {
            cartridge.swap_boot_rom(boot);
        }

        Dmg {
            cpu: Cpu::new(boot_rom.is_some()),
            cartridge,
            memory: Memory::new(boot_rom),
            ppu: Ppu::new(),
            joypad: Joypad::new(),
            serial: Serial::new(),
            timer: Timer::new(),
            apu: Apu::new(),
            interrupt_flag: Interrupt::new(),
            interrupt_enable: Interrupt::new(),
            bank: 0,
        }
    }

    pub fn reset(&mut self) { self.cpu.reset(); }

    /// Modifies the DMG state by executing one CPU instruction, and return the executed instruction
    pub fn run<C: Controller>(&mut self, controller: &mut C) -> Result<(), Box<dyn std::error::Error>> {
        // one frame == 70224 T-cycles == 17556 M-cycles
        while self.cpu.cycles < 17556 {
            let _instr = self.step(controller);

            // println!(
            //     "Executing instruction at {:04X} and {}: {}",
            //     self.cpu.pc,
            //     self.cpu.cycles,
            //     match &_instr {
            //         Some(instr) => format!("{}", instr),
            //         None => "None".to_string(),
            //     }
            // );
        }

        self.cpu.cycles = 0;
        self.apu.flush(controller);

        Ok(())
    }

    pub fn step<C: Controller>(&mut self, controller: &mut C) -> Option<InstructionBox<dyn Instruction>> {
        let prev_cycles = self.cpu.cycles;

        let instruction = Cpu::step(self);

        let delta = self.cpu.cycles.wrapping_sub(prev_cycles) * 4;

        self.ppu.step(controller, delta, &mut self.interrupt_flag);
        self.timer.step(delta, &mut self.interrupt_flag);
        self.serial.step(controller);
        self.apu.step(controller, delta);

        instruction
    }
}

impl Accessible<u16> for Dmg {
    fn read(&self, address: u16) -> u8 {
        match address {
            ROM_BANK00_START..=ROM_BANKNN_END => self.cartridge.read(address),
            VRAM_START..=VRAM_END => self.ppu.read(address),
            EXTERNAL_RAM_START..=EXTERNAL_RAM_END => self.cartridge.read(address),
            WRAM_BANK0_START..=WRAM_BANKN_END => self.memory.ram[(address - WRAM_BANK0_START) as usize],
            ECHO_RAM_START..=ECHO_RAM_END => {
                let offset = (address - ECHO_RAM_START) as usize;
                self.memory.ram[offset]
            }
            OAM_START..=OAM_END => self.ppu.read(address),

            NOT_USABLE_START..=NOT_USABLE_END => {
                // eprintln!(
                //     "Reads to prohibited memory region [{}, {}] with address {:04X} return 0xFF",
                //     NOT_USABLE_START, NOT_USABLE_END, address
                // );
                0xFF
            }

            IO_REGISTERS_START..=IO_REGISTERS_END => match address {
                JOYP => self.joypad.read(address),
                SERIAL_REGISTER_START..=SERIAL_REGISTER_END => self.serial.read(address),
                TIMER_REGISTER_START..=TIMER_REGISTER_END => self.timer.read(address),

                IF => self.interrupt_flag.0 | 0xE0,

                APU_REGISTER_START..=APU_REGISTER_END => self.apu.read(address),
                PPU_REGISTER_START..=PPU_REGISTER_END => self.ppu.read(address),

                BANK_REGISTER => self.bank,

                _ => {
                    // eprintln!("Reads to unimplemented IO register {:04X} return 0xFF", address);
                    0xFF
                }
            },
            HRAM_START..=HRAM_END => self.memory.hram[(address - HRAM_START) as usize],
            IE => self.interrupt_enable.0,
        }
    }

    fn write(&mut self, address: u16, value: u8) {
        match address {
            ROM_BANK00_START..=ROM_BANKNN_END => self.cartridge.write(address, value),
            VRAM_START..=VRAM_END => self.ppu.write(address, value),
            EXTERNAL_RAM_START..=EXTERNAL_RAM_END => self.cartridge.write(address, value),
            WRAM_BANK0_START..=WRAM_BANKN_END => {
                self.memory.ram[(address - WRAM_BANK0_START) as usize] = value
            }
            ECHO_RAM_START..=ECHO_RAM_END => {
                let offset = (address - ECHO_RAM_START) as usize;
                self.memory.ram[offset] = value;
            }
            OAM_START..=OAM_END => self.ppu.write(address, value),

            NOT_USABLE_START..=NOT_USABLE_END => {
                // eprintln!(
                //     "Writes to prohibited memory region [{}, {}] with address {:04X} are ignored",
                //     NOT_USABLE_START, NOT_USABLE_END, address
                // ),
            }

            IO_REGISTERS_START..=IO_REGISTERS_END => match address {
                JOYP => self.joypad.write(address, value),
                SERIAL_REGISTER_START..=SERIAL_REGISTER_END => self.serial.write(address, value),
                TIMER_REGISTER_START..=TIMER_REGISTER_END => self.timer.write(address, value),

                IF => self.interrupt_flag.0 = value,

                APU_REGISTER_START..=APU_REGISTER_END => self.apu.write(address, value),

                DMA_REGISTER => Ppu::dma_transfer(self, value),
                PPU_REGISTER_START..=PPU_REGISTER_END => self.ppu.write(address, value),

                // CGB CPU registers
                cgb::KEY0_SYS..=cgb::KEY1_SPD => {}

                BANK_REGISTER => {
                    // unmaps boot rom when boot reaches pc = 0x00FE, when load 1 in bank register (0xFF50)
                    // ```asm
                    // ld a, $01
                    // ld [0xFF50], a
                    // ```
                    // Next instruction will be the first `nop` in 0x0100, in the cartridge rom
                    if self.bank == 0
                        && let Some(boot) = &mut self.memory.boot_rom
                    {
                        self.cartridge.swap_boot_rom(boot);
                    }

                    self.bank = value;
                }

                cgb::VBK => {}
                cgb::HDMA1 => {}
                cgb::HDMA2 => {}
                cgb::HDMA3 => {}
                cgb::HDMA4 => {}
                cgb::HDMA5 => {}
                cgb::RP => {}
                cgb::BCPS_BGPI => {}
                cgb::BCPD_BGPD => {}
                cgb::OCPS_OBPI => {}
                cgb::OCPD_OBPD => {}
                cgb::OPRI => {}
                cgb::SVBK_WBK => {}
                cgb::PCM12 => {}
                cgb::PCM34 => {}

                _ => eprintln!("Writes to unimplemented IO register {:04X} are ignored", address),
            },

            HRAM_START..=HRAM_END => self.memory.hram[(address - HRAM_START) as usize] = value,
            IE => self.interrupt_enable.0 = value,
        }
    }
}

impl Accessible16<u16, u16> for Dmg {
    fn load(&self, address: u16) -> u16 { to_u16(self.read(address), self.read(address.wrapping_add(1))) }

    fn store(&mut self, address: u16, value: u16) {
        self.write(address, low(value));
        self.write(address.wrapping_add(1), high(value));
    }
}

impl Accessible<R8> for Dmg {
    fn read(&self, address: R8) -> u8 {
        match address {
            R8::A => self.cpu.a,
            R8::F => self.cpu.f,
            R8::B => self.cpu.b,
            R8::C => self.cpu.c,
            R8::D => self.cpu.d,
            R8::E => self.cpu.e,
            R8::H => self.cpu.h,
            R8::L => self.cpu.l,
        }
    }

    fn write(&mut self, address: R8, value: u8) {
        match address {
            R8::A => self.cpu.a = value,
            R8::F => self.cpu.f = value & 0xF0,
            R8::B => self.cpu.b = value,
            R8::C => self.cpu.c = value,
            R8::D => self.cpu.d = value,
            R8::E => self.cpu.e = value,
            R8::H => self.cpu.h = value,
            R8::L => self.cpu.l = value,
        }
    }
}

impl Accessible16<R16, R8> for Dmg {
    fn load(&self, address: R16) -> u16 {
        match address {
            R16::AF => self.cpu.af(),
            R16::BC => self.cpu.bc(),
            R16::DE => self.cpu.de(),
            R16::HL => self.cpu.hl(),
        }
    }

    fn store(&mut self, address: R16, value: u16) {
        match address {
            R16::AF => self.cpu.set_af(value),
            R16::BC => self.cpu.set_bc(value),
            R16::DE => self.cpu.set_de(value),
            R16::HL => self.cpu.set_hl(value),
        };
    }
}
