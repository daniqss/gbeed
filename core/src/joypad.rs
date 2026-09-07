use crate::{interrupts::Interrupt, prelude::*};

pub const JOYP: u16 = 0xFF00;

pub const SELECT_BUTTONS: u8 = 0x20;
pub const SELECT_DPAD: u8 = 0x10;
pub const INPUT_DOWN_START: u8 = 0x08;
pub const INPUT_UP_SELECT: u8 = 0x04;
pub const INPUT_LEFT_B: u8 = 0x02;
pub const INPUT_RIGHT_A: u8 = 0x01;

const PRESS_START: u8 = 0x80;
const PRESS_SELECT: u8 = 0x40;
const PRESS_B: u8 = 0x20;
const PRESS_A: u8 = 0x10;
const PRESS_DOWN: u8 = 0x08;
const PRESS_UP: u8 = 0x04;
const PRESS_LEFT: u8 = 0x02;
const PRESS_RIGHT: u8 = 0x01;

#[repr(u8)]
#[derive(Debug, Clone, Copy)]
pub enum JoypadButton {
    Right = PRESS_RIGHT,
    Left = PRESS_LEFT,
    Up = PRESS_UP,
    Down = PRESS_DOWN,
    A = PRESS_A,
    B = PRESS_B,
    Select = PRESS_SELECT,
    Start = PRESS_START,
}
/// # Joypad Input
/// It uses 6 GPIO pins to read the state of the buttons.
/// | P14   | P15    |     |
/// | ----- | ------ | --- |
/// | Down  | Start  | P13 |
/// | Up    | Select | P12 |
/// | Left  | B      | P11 |
/// | Right | A      | P10 |
/// A button beeing pressed is seen as the corresponding bit being 0, not 1 as usual in other components.
#[derive(Debug)]
pub struct Joypad {
    pub input: u8,
    joyp: u8,

    /// state of the P10-P13 lines on the previous step, used to detect falling edges
    previous_lines: u8,
}

impl Default for Joypad {
    fn default() -> Self { Joypad::new() }
}

impl Joypad {
    pub(crate) fn new() -> Self {
        Self {
            input: 0xFF,
            joyp: 0xCF,
            previous_lines: 0x0F,
        }
    }

    /// The interrupt is requested on a high to low edge of any of the four input lines,
    /// no matter if it comes from a button being pressed or from a write selecting the
    /// other nibble, so both cases are covered by sampling the lines once per step.
    pub(crate) fn step(&mut self, interrupt: &mut Interrupt) {
        let lines = self.lines();

        if self.previous_lines & !lines != 0 {
            interrupt.set_joypad_interrupt(true);
        }

        self.previous_lines = lines;
    }

    /// Current state of the P10-P13 lines, low means pressed
    fn lines(&self) -> u8 {
        let buttons = if self.select_buttons() {
            0x0F
        } else {
            self.input >> 4
        };

        let dpad = if self.select_dpad() {
            0x0F
        } else {
            self.input & 0x0F
        };

        buttons & dpad
    }

    pub fn button_down(&mut self, btn: JoypadButton, is_down: bool) {
        let mask = btn as u8;

        if is_down {
            self.input &= !mask;
        } else {
            self.input |= mask;
        }
    }

    bit_accessors! {
        pub(crate) target: joyp;

        SELECT_BUTTONS,
        SELECT_DPAD,
        INPUT_DOWN_START,
        INPUT_UP_SELECT,
        INPUT_LEFT_B,
        INPUT_RIGHT_A
    }
}

impl Accessible<u16> for Joypad {
    fn read(&self, address: u16) -> u8 {
        match address {
            // bits 7 and 6 are always high, bits 5 and 4 are the select bits, and bits 3 to 0 are the current input
            JOYP => 0xC0 | (self.joyp & 0x30) | self.lines(),
            _ => unreachable!(
                "Attempted to read from Joypad with invalid address {:04X}",
                address
            ),
        }
    }

    fn write(&mut self, address: u16, value: u8) {
        match address {
            // only bits SELECT_BUTTONS and SELECT_DPAD are writable,
            // the edge caused by changing the selected nibble is detected in `step`
            JOYP => self.joyp = (self.joyp & 0xCF) | (value & 0x30),
            _ => unreachable!(
                "Attempted to write to Joypad with invalid address {:04X}",
                address
            ),
        }
    }
}

impl core::fmt::Display for Joypad {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        let mut buttons = Vec::with_capacity(10);
        if self.input & PRESS_RIGHT == 0 {
            buttons.push("Right");
        }
        if self.input & PRESS_LEFT == 0 {
            buttons.push("Left");
        }
        if self.input & PRESS_UP == 0 {
            buttons.push("Up");
        }
        if self.input & PRESS_DOWN == 0 {
            buttons.push("Down");
        }
        if self.input & PRESS_A == 0 {
            buttons.push("A");
        }
        if self.input & PRESS_B == 0 {
            buttons.push("B");
        }
        if self.input & PRESS_SELECT == 0 {
            buttons.push("Select");
        }
        if self.input & PRESS_START == 0 {
            buttons.push("Start");
        }

        write!(f, "Joypad: [{}]", buttons.join(", "))
    }
}
