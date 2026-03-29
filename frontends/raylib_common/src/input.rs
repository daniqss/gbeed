use gbeed_core::{Joypad, JoypadButton};
use raylib::prelude::*;

#[derive(Default, Copy, Clone)]
pub struct InputState {
    pub up: bool,
    pub down: bool,
    pub left: bool,
    pub right: bool,
    pub a: bool,
    pub b: bool,
    pub start: bool,
    pub select: bool,
    pub escape: bool,
    pub speed_up: bool,
}

pub trait ToInputState {
    fn to_input(&self, rl: &RaylibHandle) -> InputState;
}

pub struct InputKeyTriggers {
    up: [KeyboardKey; 2],
    down: [KeyboardKey; 2],
    left: [KeyboardKey; 2],
    right: [KeyboardKey; 2],
    a: [KeyboardKey; 2],
    b: [KeyboardKey; 2],
    start: [KeyboardKey; 2],
    select: [KeyboardKey; 2],
    escape: [KeyboardKey; 2],
    speed_up: [KeyboardKey; 2],
}

impl Default for InputKeyTriggers {
    fn default() -> Self {
        Self {
            up: [KeyboardKey::KEY_W, KeyboardKey::KEY_UP],
            down: [KeyboardKey::KEY_S, KeyboardKey::KEY_DOWN],
            left: [KeyboardKey::KEY_A, KeyboardKey::KEY_LEFT],
            right: [KeyboardKey::KEY_D, KeyboardKey::KEY_RIGHT],
            a: [KeyboardKey::KEY_J, KeyboardKey::KEY_ENTER],
            b: [KeyboardKey::KEY_K, KeyboardKey::KEY_C],
            start: [KeyboardKey::KEY_L, KeyboardKey::KEY_X],
            select: [KeyboardKey::KEY_SEMICOLON, KeyboardKey::KEY_Z],
            escape: [KeyboardKey::KEY_ESCAPE, KeyboardKey::KEY_BACKSPACE],
            speed_up: [KeyboardKey::KEY_LEFT_SHIFT, KeyboardKey::KEY_SPACE],
        }
    }
}

impl ToInputState for InputKeyTriggers {
    fn to_input(&self, rl: &RaylibHandle) -> InputState {
        InputState {
            up: self.up.iter().any(|k| rl.is_key_down(*k)),
            down: self.down.iter().any(|k| rl.is_key_down(*k)),
            left: self.left.iter().any(|k| rl.is_key_down(*k)),
            right: self.right.iter().any(|k| rl.is_key_down(*k)),
            a: self.a.iter().any(|k| rl.is_key_down(*k)),
            b: self.b.iter().any(|k| rl.is_key_down(*k)),
            start: self.start.iter().any(|k| rl.is_key_down(*k)),
            select: self.select.iter().any(|k| rl.is_key_down(*k)),
            escape: self.escape.iter().any(|k| rl.is_key_pressed(*k)),
            speed_up: self.speed_up.iter().any(|k| rl.is_key_down(*k)),
        }
    }
}

pub struct MouseButtonArea {
    x: i32,
    y: i32,
    width: i32,
    height: i32,
}

impl MouseButtonArea {
    pub fn new(x: i32, y: i32, width: i32, height: i32) -> Self { Self { x, y, width, height } }

    pub fn is_mouse_over(&self, mouse_pos: Vector2) -> bool {
        (mouse_pos.x as i32) >= self.x
            && (mouse_pos.x as i32) < self.x + self.width
            && (mouse_pos.y as i32) >= self.y
            && (mouse_pos.y as i32) < self.y + self.height
    }
}

pub struct InputMouseTriggers {
    up: MouseButtonArea,
    down: MouseButtonArea,
    left: MouseButtonArea,
    right: MouseButtonArea,
    a: MouseButtonArea,
    b: MouseButtonArea,
    start: MouseButtonArea,
    select: MouseButtonArea,
    escape: Option<MouseButtonArea>,
    speed_up: Option<MouseButtonArea>,
}

impl ToInputState for InputMouseTriggers {
    fn to_input(&self, rl: &RaylibHandle) -> InputState {
        let mouse_pos = rl.get_mouse_position();

        InputState {
            up: self.up.is_mouse_over(mouse_pos),
            down: self.down.is_mouse_over(mouse_pos),
            left: self.left.is_mouse_over(mouse_pos),
            right: self.right.is_mouse_over(mouse_pos),
            a: self.a.is_mouse_over(mouse_pos),
            b: self.b.is_mouse_over(mouse_pos),
            start: self.start.is_mouse_over(mouse_pos),
            select: self.select.is_mouse_over(mouse_pos),
            escape: self
                .escape
                .as_ref()
                .is_some_and(|area| area.is_mouse_over(mouse_pos)),
            speed_up: self
                .speed_up
                .as_ref()
                .is_some_and(|area| area.is_mouse_over(mouse_pos)),
        }
    }
}

impl InputState {
    pub fn merge(rl: &RaylibHandle, states: &[impl ToInputState]) -> InputState {
        states.iter().fold(InputState::default(), |mut acc, state| {
            let to_state = &state.to_input(rl);

            acc.up |= to_state.up;
            acc.down |= to_state.down;
            acc.left |= to_state.left;
            acc.right |= to_state.right;
            acc.a |= to_state.a;
            acc.b |= to_state.b;
            acc.start |= to_state.start;
            acc.select |= to_state.select;
            acc.escape |= to_state.escape;
            acc.speed_up |= to_state.speed_up;

            acc
        })
    }

    pub fn apply(self, joypad: &mut Joypad) {
        joypad.button_down(JoypadButton::Up, self.up);
        joypad.button_down(JoypadButton::Down, self.down);
        joypad.button_down(JoypadButton::Left, self.left);
        joypad.button_down(JoypadButton::Right, self.right);
        joypad.button_down(JoypadButton::A, self.a);
        joypad.button_down(JoypadButton::B, self.b);
        joypad.button_down(JoypadButton::Start, self.start);
        joypad.button_down(JoypadButton::Select, self.select);
    }
}
