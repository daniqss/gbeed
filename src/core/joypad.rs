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
#[derive(Debug, Default)]
pub struct Joypad {
    pub input: u8,
    joyp: u8,
}

impl Joypad {
    pub fn new() -> Self {
        Self {
            input: 0xFF,
            joyp: 0xCF,
        }
    }

    pub fn button_down(&mut self, btn: JoypadButton, is_down: bool) {
        let mask = btn as u8;

        if is_down {
            self.input &= !mask;
        } else {
            self.input |= mask;
        }
    }

    pub fn any_input(&self) -> bool { self.input != 0xFF }

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

impl std::fmt::Display for Joypad {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
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
