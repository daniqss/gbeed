use crate::prelude::*;

pub const IF: u16 = 0xFF0F;
pub const IE: u16 = 0xFFFF;

// bit masks for interrupts
pub const JOYPAD_INTERRUPT: u8 = 0x10;
pub const SERIAL_INTERRUPT: u8 = 0x08;
pub const TIMER_INTERRUPT: u8 = 0x04;
pub const LCD_STAT_INTERRUPT: u8 = 0x02;
pub const VBLANK_INTERRUPT: u8 = 0x01;

/// # Interrupt Flag Register
/// Represents **requested** interrupts.
/// Bits are set by hardware or manually by the user using instructions `EI` and `DI`.
/// # Interrupt Enable Register
/// Controls whether specific interrupts are **allowed** to be serviced.
/// Even if requested in `IF`, the handler won't run unless the corresponding bit here is 1.
#[derive(Debug, Default)]
pub struct Interrupt(pub u8);

impl Interrupt {
    pub fn new() -> Self { Interrupt(0) }

    bit_accessors! {
        target: 0;

        JOYPAD_INTERRUPT,
        SERIAL_INTERRUPT,
        TIMER_INTERRUPT,
        LCD_STAT_INTERRUPT,
        VBLANK_INTERRUPT
    }
}
