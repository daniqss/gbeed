use std::fmt::Write;

use super::{
    InstructionDestination as ID, InstructionEffect, InstructionError, InstructionResult,
    InstructionTarget as IT,
};
use crate::{
    Dmg,
    core::{
        IO_REGISTERS_START,
        cpu::{
            flags::Flags,
            instructions::Instruction,
            {R8, R16},
        },
        memory::is_high_address,
    },
};

/// Load from/to high memory area instruction
/// Usually used to access memory mapped IO and HRAM,
/// so the used addresses are between 0xFF00 and 0xFFFF
pub struct Ldh {
    dst: ID,
    src: IT,
}

impl Ldh {
    pub fn new(dst: ID, src: IT) -> Box<Self> { Box::new(Self { dst, src }) }
}

impl Instruction for Ldh {
    fn exec(&mut self, gb: &mut Dmg) -> InstructionResult {
        let (dst, src, addr, cycles, len): (&mut u8, u8, Option<u16>, u8, u8) = match (&self.dst, &self.src) {
            // copy the src value in register A to the byte at 16 bits immediate address (that must be between 0xFF00 and 0xFFFF)
            (ID::PointedByA8(addr), IT::Reg8(src, reg)) if *reg == Reg::A => {
                let addr = IO_REGISTERS_START + (*addr as u16);
                (&mut gb[addr], *src, Some(addr), 3, 2)
            }
            // copy the src value in register A to the byte at address 0xFF00 + value in register C
            (ID::PointedByCPlusFF00, IT::Reg8(src, reg)) if *reg == Reg::A => {
                let addr = gb.cpu.c as u16 + IO_REGISTERS_START;
                (&mut gb[addr], *src, Some(addr), 2, 1)
            }
            // copy the src byte addressed by 16 bits immediate (that must be between 0xFF00 and 0xFFFF) into dst register A
            (ID::Reg8(reg), IT::PointedByA8(addr)) if *reg == Reg::A => {
                let addr = IO_REGISTERS_START + (*addr as u16);
                let pointed = gb[addr];
                (&mut gb[reg], pointed, Some(addr), 3, 2)
            }
            // copy the src byte addressed by 0xFF00 + C into dst register A
            (ID::Reg8(reg), IT::PointedByCPlusFF00) if *reg == Reg::A => {
                let addr = gb.cpu.c as u16 + IO_REGISTERS_START;
                let pointed = gb[addr];
                (&mut gb[reg], pointed, Some(addr), 2, 1)
            }

            _ => return Err(InstructionError::MalformedInstruction),
        };

        // if the destination target is a memory byte, check if the address is in range
        // otherwise return an error
        if let Some(addr) = addr
            && !is_high_address(addr)
        {
            return Err(InstructionError::AddressOutOfRange {
                addr,
                op: gb[gb.cpu.pc],
                pc: gb.cpu.pc,
            });
        }

        *dst = src;

        Ok(InstructionEffect::new(cycles, len, Flags::none()))
    }

    fn disassembly(&self) -> String { format!("ldh {},{}", self.dst, self.src) }
}
