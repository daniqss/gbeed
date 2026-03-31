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
            escape: self.escape.iter().any(|k| rl.is_key_down(*k)),
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

pub struct InputManager {
    pub triggers: InputKeyTriggers,
    pub current: InputState,
    pub previous: InputState,
    pub repeat_timer: f32,
    pub debounce_timer: f32,
}

impl Default for InputManager {
    fn default() -> Self { Self::new() }
}

impl InputManager {
    pub fn new() -> Self {
        Self {
            triggers: InputKeyTriggers::default(),
            current: InputState::default(),
            previous: InputState::default(),
            repeat_timer: 0.0,
            debounce_timer: 0.0,
        }
    }

    pub fn with_debounce(debounce_time: f32) -> Self {
        let mut manager = Self::new();
        manager.debounce_timer = debounce_time;
        manager
    }

    pub fn update(&mut self, rl: &RaylibHandle, dt: f32) {
        if self.debounce_timer > 0.0 {
            self.debounce_timer -= dt;
            self.current = self.triggers.to_input(rl);
            self.previous = self.current;
            return;
        }

        self.previous = self.current;
        self.current = self.triggers.to_input(rl);

        let dirs_held = self.current.up || self.current.down || self.current.left || self.current.right;
        let dirs_pressed = self.is_pressed_up()
            || self.is_pressed_down()
            || self.is_pressed_left()
            || self.is_pressed_right();

        if dirs_held {
            self.repeat_timer += dt;
            if dirs_pressed {
                self.repeat_timer = 0.0;
            }
        } else {
            self.repeat_timer = 0.0;
        }
    }

    pub fn state(&self) -> InputState { self.current }

    pub fn is_pressed_up(&self) -> bool { self.current.up && !self.previous.up }
    pub fn is_pressed_down(&self) -> bool { self.current.down && !self.previous.down }
    pub fn is_pressed_left(&self) -> bool { self.current.left && !self.previous.left }
    pub fn is_pressed_right(&self) -> bool { self.current.right && !self.previous.right }
    pub fn is_pressed_a(&self) -> bool { self.current.a && !self.previous.a }
    pub fn is_pressed_b(&self) -> bool { self.current.b && !self.previous.b }
    pub fn is_pressed_start(&self) -> bool { self.current.start && !self.previous.start }
    pub fn is_pressed_select(&self) -> bool { self.current.select && !self.previous.select }
    pub fn is_pressed_escape(&self) -> bool { self.current.escape && !self.previous.escape }

    pub fn is_repeated_up(&self, dt: f32) -> bool {
        self.is_pressed_up() || (self.current.up && self.check_repeat(dt))
    }
    pub fn is_repeated_down(&self, dt: f32) -> bool {
        self.is_pressed_down() || (self.current.down && self.check_repeat(dt))
    }
    pub fn is_repeated_left(&self, dt: f32) -> bool {
        self.is_pressed_left() || (self.current.left && self.check_repeat(dt))
    }
    pub fn is_repeated_right(&self, dt: f32) -> bool {
        self.is_pressed_right() || (self.current.right && self.check_repeat(dt))
    }

    fn check_repeat(&self, dt: f32) -> bool {
        const REPEAT_DELAY: f32 = 0.3;
        const REPEAT_RATE: f32 = 0.08;
        if self.repeat_timer >= REPEAT_DELAY {
            let ticks = ((self.repeat_timer - REPEAT_DELAY) / REPEAT_RATE) as usize;
            let prev = ((self.repeat_timer - REPEAT_DELAY - dt.max(0.0)) / REPEAT_RATE) as usize;
            ticks > prev
        } else {
            false
        }
    }
}
