use crate::{bit_accessors, core::Accessible};

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

/// # Joypad Input
/// It uses 6 GPIO pins to read the state of the buttons.
/// | P14   | P15    |     |
/// | ----- | ------ | --- |
/// | Down  | Start  | P13 |
/// | Up    | Select | P12 |
/// | Left  | B      | P11 |
/// | Right | A      | P10 |
/// A button beeing pressed is seen as the corresponding bit being 0, not 1 as usual in other components.
#[derive(Debug, Default)]
pub struct Joypad {
    input: u8,
    joyp: u8,
}

impl Joypad {
    pub fn new() -> Self {
        Self {
            input: 0xFF,
            joyp: 0xCF,
        }
    }

    // TODO: change visibility in macro
    bit_accessors! {
        target: joyp;

        SELECT_BUTTONS,
        SELECT_DPAD,
        INPUT_DOWN_START,
        INPUT_UP_SELECT,
        INPUT_LEFT_B,
        INPUT_RIGHT_A
    }

    // TODO: use enum repr u8
    bit_accessors! {
        target: input;

        PRESS_START,
        PRESS_SELECT,
        PRESS_B,
        PRESS_A,
        PRESS_DOWN,
        PRESS_UP,
        PRESS_LEFT,
        PRESS_RIGHT
    }
}

impl Accessible<u16> for Joypad {
    fn read(&self, address: u16) -> u8 {
        match address {
            JOYP if !self.select_buttons() => SELECT_BUTTONS | (self.input >> 4),
            JOYP if !self.select_dpad() => SELECT_DPAD | (self.input & 0x0F),
            JOYP => self.joyp | 0x0F,
            _ => unreachable!(
                "Attempted to read from Joypad with invalid address {:04X}",
                address
            ),
        }
    }

    fn write(&mut self, address: u16, value: u8) {
        match address {
            // only bits SELECT_BUTTONS and SELECT_DPAD are writable
            JOYP => self.joyp = (self.joyp & 0xCF) | (value & 0x30),
            _ => unreachable!(
                "Attempted to write to Joypad with invalid address {:04X}",
                address
            ),
        }
    }
}
