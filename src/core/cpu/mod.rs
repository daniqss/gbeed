mod flags;
mod instructions;
mod registers;

use crate::{
    core::memory::MemoryBus,
    prelude::utils::{to_u8, to_u16},
};
use instructions::{InstructionTarget as IT, *};
use registers::{Register8, Register16};
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

    pub cycles: usize,
    bus: MemoryBus,
}

impl Cpu {
    pub fn new(bus: MemoryBus) -> Cpu {
        Cpu {
            a: 0x00,
            f: 0x00,
            b: 0x00,
            c: 0x00,
            d: 0x00,
            e: 0x00,
            h: 0x00,
            l: 0x00,
            pc: 0x0100,
            sp: 0x0000,

            cycles: 0,
            bus,
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
        self.pc = 0x0100;
        self.sp = 0x0000;
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

    /// execute instruction based on the opcode
    /// return a result with the effect of the instruction or an instruction error (e.g unused opcode)
    pub fn exec(&mut self, opcode: u8) -> InstructionResult {
        // maybe I should create a instruction struct, and then run it
        // (trait Instruction), this way I could pattern match the instruction one time
        // and run and disassemble it with the same instruction, with the same args
        // this way I can implement better error handling
        let effect = match opcode {
            0x00 => todo!("NOP"), // NOP
            /*
            0x40 => return Err(InstructionError::NoOp(opcode, self.pc)), // LD B,B
            0x41 => ld_r8_r8(&mut self.b, self.c),                       // LD B,C
            0x42 => ld_r8_r8(&mut self.b, self.d),                       // LD B,D
            0x43 => ld_r8_r8(&mut self.b, self.e),                       // LD B,E
            0x44 => ld_r8_r8(&mut self.b, self.h),                       // LD B,H
            0x45 => ld_r8_r8(&mut self.b, self.l),                       // LD B,L
            0x46 => {
                let addr = self.get_hl();
                let value = self.bus.borrow()[addr];
                ld_r8_hl(&mut self.b, value)
            } // LD B,(HL)
            0x47 => ld_r8_r8(&mut self.b, self.a),                       // LD B,A
            0x48 => ld_r8_r8(&mut self.c, self.b),                       // LD C,B
            0x49 => return Err(InstructionError::NoOp(opcode, self.pc)), // LD C,C
            0x4A => ld_r8_r8(&mut self.c, self.d),                       // LD C,D
            0x4B => ld_r8_r8(&mut self.c, self.e),                       // LD C,E
            0x4C => ld_r8_r8(&mut self.c, self.h),                       // LD C,H
            0x4D => ld_r8_r8(&mut self.c, self.l),                       // LD C,L
            0x4E => {
                let addr = self.get_hl();
                let value = self.bus.borrow()[addr];
                ld_r8_hl(&mut self.c, value)
            } // LD C,(HL)
            0x4F => ld_r8_r8(&mut self.c, self.a),                       // LD C,A
            0x50 => ld_r8_r8(&mut self.d, self.b),                       // LD D,B
            0x51 => ld_r8_r8(&mut self.d, self.c),                       // LD D,C
            0x52 => return Err(InstructionError::NoOp(opcode, self.pc)), // LD D,D
            0x53 => ld_r8_r8(&mut self.d, self.e),                       // LD D,E
            0x54 => ld_r8_r8(&mut self.d, self.h),                       // LD D,H
            0x55 => ld_r8_r8(&mut self.d, self.l),                       // LD D,L
            0x56 => {
                let addr = self.get_hl();
                let value = self.bus.borrow()[addr];
                ld_r8_hl(&mut self.d, value)
            } // LD D,(HL)
            0x57 => ld_r8_r8(&mut self.d, self.a),                       // LD D,A
            0x58 => ld_r8_r8(&mut self.e, self.b),                       // LD E,B
            0x59 => ld_r8_r8(&mut self.e, self.c),                       // LD E,C
            0x5A => ld_r8_r8(&mut self.e, self.d),                       // LD E,D
            0x5B => return Err(InstructionError::NoOp(opcode, self.pc)), // LD E,E
            0x5C => ld_r8_r8(&mut self.e, self.h),                       // LD E,H
            0x5D => ld_r8_r8(&mut self.e, self.l),                       // LD E,L
            0x5E => {
                let addr = self.get_hl();
                let value = self.bus.borrow()[addr];
                ld_r8_hl(&mut self.e, value)
            } // LD E,(HL)
            0x5F => ld_r8_r8(&mut self.e, self.a),                       // LD E,A
            0x60 => ld_r8_r8(&mut self.h, self.b),                       // LD H,B
            0x61 => ld_r8_r8(&mut self.h, self.c),                       // LD H,C
            0x62 => ld_r8_r8(&mut self.h, self.d),                       // LD H,D
            0x63 => ld_r8_r8(&mut self.h, self.e),                       // LD H,E
            0x64 => return Err(InstructionError::NoOp(opcode, self.pc)), // LD H,H
            0x65 => ld_r8_r8(&mut self.h, self.l),                       // LD H,L
            0x66 => {
                let addr = self.get_hl();
                let value = self.bus.borrow()[addr];
                ld_r8_hl(&mut self.h, value)
            } // LD H,(HL)
            0x67 => ld_r8_r8(&mut self.h, self.a),                       // LD H,A
            0x68 => ld_r8_r8(&mut self.l, self.b),                       // LD L,B
            0x69 => ld_r8_r8(&mut self.l, self.c),                       // LD L,C
            0x6A => ld_r8_r8(&mut self.l, self.d),                       // LD L,D
            0x6B => ld_r8_r8(&mut self.l, self.e),                       // LD L,E
            0x6C => ld_r8_r8(&mut self.l, self.h),                       // LD L,H
            0x6D => return Err(InstructionError::NoOp(opcode, self.pc)), // LD L,L
            0x6E => {
                let addr = self.get_hl();
                let value = self.bus.borrow()[addr];
                ld_r8_hl(&mut self.l, value)
            } // LD L,(HL)
            0x6F => ld_r8_r8(&mut self.l, self.a),                       // LD L,A
            0x70 => ld_hl_r8(&mut self.bus.borrow_mut()[self.get_hl()], self.a), // LD (HL),B
            0x71 => ld_hl_r8(&mut self.bus.borrow_mut()[self.get_hl()], self.c), // LD (HL),C
            0x72 => ld_hl_r8(&mut self.bus.borrow_mut()[self.get_hl()], self.d), // LD (HL),D
            0x73 => ld_hl_r8(&mut self.bus.borrow_mut()[self.get_hl()], self.e), // LD (HL),E
            0x74 => ld_hl_r8(&mut self.bus.borrow_mut()[self.get_hl()], self.h), // LD (HL
            0x75 => ld_hl_r8(&mut self.bus.borrow_mut()[self.get_hl()], self.l), // LD (HL),L
            0x76 => return Err(InstructionError::NotImplemented(opcode, self.pc)), // HALT
            0x77 => ld_hl_r8(&mut self.bus.borrow_mut()[self.get_hl()], self.a), // LD (HL),A
            0x78 => ld_r8_r8(&mut self.a, self.b),                       // LD A,B
            0x79 => ld_r8_r8(&mut self.a, self.c),                       // LD A,C
            0x7A => ld_r8_r8(&mut self.a, self.d),                       // LD A,D
            0x7B => ld_r8_r8(&mut self.a, self.e),                       // LD A,E
            0x7C => ld_r8_r8(&mut self.a, self.h),                       // LD A,H
            0x7D => ld_r8_r8(&mut self.a, self.l),                       // LD A,L
            // 0x7E => {
            //     let addr = self.get_hl();
            //     ld_r8_hl(&mut self.a, self.bus.borrow()[addr])
            // } // LD A,(HL)
            // 0x7F => return Err(InstructionError::NoOp(opcode, self.pc)), // LD A,A
            // 0xE0 => LDH::exec(
            //     IT::DstPointedByN16(&mut self.bus.borrow_mut()[self.pc + 1], self.pc + 1),
            //     IT::RegisterA(self.a),
            // )?, // LDH [n16],A
            // 0xE2 => LDH::exec(
            //     IT::DstPointedByCPlusFF00(
            //         &mut self.bus.borrow_mut()[0xFF00 + self.c as u16],
            //         self.c as u16,
            //     ),
            //     IT::RegisterA(self.a),
            // )?, // LDH [C],A
            // 0xF0 => LDH::exec(
            //     IT::DstRegisterA(&mut self.a),
            //     IT::PointedByN16(self.bus.borrow()[self.pc + 1], self.pc + 1),
            // )?, // LDH A,[n16]
            // 0xF2 => LDH::exec(
            //     IT::DstRegisterA(&mut self.a),
            //     IT::PointedByCPlusFF00(
            //         self.bus.borrow()[0xFF00 + self.c as u16],
            //         self.c as u16,
            //     ),
            // )?, // LDH A,[C]
            */
            0xF2 => {
                let mut instr = LDH::new(
                    IT::DstRegister8(&mut self.a, Register8::A),
                    IT::PointedByCPlusFF00(self.bus.borrow()[0xFF00 + self.c as u16], self.c as u16),
                );

                instr.exec()?
            }

            _ => return Err(InstructionError::NotImplemented(opcode, self.pc)),
        };

        Ok(effect)
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

// #[cfg(test)]
// mod cpu_tests {
//     use crate::core::memory::Memory;

//     #[test]
//     fn test_cpu() {
//         let mut cpu = super::Cpu::new(Memory::new(None, None));
//         let (af_value, bc_value, de_value, hl_value) = (0x1234, 0x5678, 0x9ABC, 0xDEF0);
//         cpu.set_af(af_value);
//         cpu.set_bc(bc_value);
//         cpu.set_de(de_value);
//         cpu.set_hl(hl_value);

//         println!("{}", cpu);
//         println!(
//             "{:4X} {:4X} {:4x} {:4X}",
//             af_value, bc_value, de_value, hl_value
//         );

//         assert_eq!(cpu.get_af(), af_value);
//         assert_eq!(cpu.get_bc(), bc_value);
//         assert_eq!(cpu.get_de(), de_value);
//         assert_eq!(cpu.get_hl(), hl_value);

//         cpu.reset();
//         assert_eq!(cpu.a, 0x00);
//         assert_eq!(cpu.f, 0x00);
//         assert_eq!(cpu.b, 0x00);
//         assert_eq!(cpu.c, 0x00);
//         assert_eq!(cpu.d, 0x00);
//         assert_eq!(cpu.e, 0x00);
//         assert_eq!(cpu.h, 0x00);
//         assert_eq!(cpu.l, 0x00);
//         assert_eq!(cpu.pc, 0x0100);
//         assert_eq!(cpu.sp, 0x0000);
//         assert_eq!(cpu.cycles, 0);
//     }
// }
