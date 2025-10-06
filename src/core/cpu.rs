use crate::prelude::{utils::to_u8, utils::to_u16};
use std::fmt::{self, Display, Formatter};

/// # CPU
/// Gameboy CPU, with a mix of Intel 8080 and Zilog Z80 features and instruction set.
/// Most of its register are 8-bits ones, that are commonly used as pairs to perform 16-bits operations.
/// The only 16-bits registers are the stack pointer (SP) and the program counter (PC).
pub struct Cpu {
    pub a: u8,
    pub f: u8,
    pub b: u8,
    pub c: u8,
    pub d: u8,
    pub e: u8,
    pub h: u8,
    pub l: u8,
    pub pc: u16,
    pub sp: u16,

    cycles: usize,
}

impl Cpu {
    pub fn new() -> Cpu {
        Cpu {
            a: 0x00,
            f: 0x00,
            b: 0x00,
            c: 0x00,
            d: 0x00,
            e: 0x00,
            h: 0x00,
            l: 0x00,
            pc: 0x0000,
            sp: 0x0100,

            cycles: 0,
        }
    }

    pub fn reset(&mut self) {
        self.a = 0x00;
        self.f = 0x00;
        self.b = 0x00;
        self.c = 0x00;
        self.d = 0x00;
        self.e = 0x00;
        self.h = 0x00;
        self.l = 0x00;
        self.pc = 0x0000;
        self.sp = 0x1000;
        self.cycles = 0;
    }

    pub fn get_af(&self) -> u16 { to_u16(self.a, self.f) }
    pub fn get_bc(&self) -> u16 { to_u16(self.b, self.c) }
    pub fn get_de(&self) -> u16 { to_u16(self.d, self.e) }
    pub fn get_hl(&self) -> u16 { to_u16(self.h, self.l) }

    pub fn set_af(&mut self, value: u16) -> () { (self.a, self.f) = to_u8(value); }
    pub fn set_bc(&mut self, value: u16) -> () { (self.b, self.c) = to_u8(value); }
    pub fn set_de(&mut self, value: u16) -> () { (self.d, self.e) = to_u8(value); }
    pub fn set_hl(&mut self, value: u16) -> () { (self.h, self.l) = to_u8(value); }

    /// Flags are set if occurs a condition in the last math operation
    pub fn set_flags(&mut self, operation_result: u8) -> () { self.f = operation_result; }

    pub fn exec_next(&mut self, instruction: u16) -> () {
        // most instructions are 8 bits, and 16 instructions are differentiated from the rest from the first 8 bits
        let opcode = (instruction >> 8) as u8;

        match opcode {
            0x00 => println!("opcode -> {}, instruction -> {}", opcode, instruction),
            _ => todo!("xd"),
        };
    }
}

impl Display for Cpu {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        write!(
            f,
            "a: {:02X} f: {:02X} b: {:02X} c: {:02X} d: {:02X} e: {:02X} h: {:02X} l: {:02X} pc: {:04X} sp: {:04X}",
            self.a, self.f, self.b, self.c, self.d, self.e, self.h, self.l, self.pc, self.sp
        )
    }
}

#[cfg(test)]
mod cpu_tests {
    #[test]
    fn test_cpu() {
        let mut cpu = super::Cpu::new();
        let (af_value, bc_value, de_value, hl_value) = (0x1234, 0x5678, 0x9ABC, 0xDEF0);
        cpu.set_af(af_value);
        cpu.set_bc(bc_value);
        cpu.set_de(de_value);
        cpu.set_hl(hl_value);

        println!("{}", cpu);
        println!(
            "{:4X} {:4X} {:4x} {:4X}",
            af_value, bc_value, de_value, hl_value
        );

        assert_eq!(cpu.get_af(), af_value);
        assert_eq!(cpu.get_bc(), bc_value);
        assert_eq!(cpu.get_de(), de_value);
        assert_eq!(cpu.get_hl(), hl_value);

        cpu.reset();
        assert_eq!(cpu.a, 0x00);
        assert_eq!(cpu.f, 0x00);
        assert_eq!(cpu.b, 0x00);
        assert_eq!(cpu.c, 0x00);
        assert_eq!(cpu.d, 0x00);
        assert_eq!(cpu.e, 0x00);
        assert_eq!(cpu.h, 0x00);
        assert_eq!(cpu.l, 0x00);
        assert_eq!(cpu.pc, 0x0000);
        assert_eq!(cpu.sp, 0x0000);
        assert_eq!(cpu.cycles, 0);
    }
}
