use gbeed_core::{Joypad, JoypadButton};
use raylib::prelude::*;

#[derive(Debug, Default, Copy, Clone)]
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

#[derive(Debug, Clone)]
pub struct InputKeyTriggers {
    pub up: Box<[KeyboardKey]>,
    pub down: Box<[KeyboardKey]>,
    pub left: Box<[KeyboardKey]>,
    pub right: Box<[KeyboardKey]>,
    pub a: Box<[KeyboardKey]>,
    pub b: Box<[KeyboardKey]>,
    pub start: Box<[KeyboardKey]>,
    pub select: Box<[KeyboardKey]>,
    pub escape: Box<[KeyboardKey]>,
    pub speed_up: Box<[KeyboardKey]>,
}

impl Default for InputKeyTriggers {
    fn default() -> Self {
        Self {
            up: [KeyboardKey::KEY_W, KeyboardKey::KEY_UP].into(),
            down: [KeyboardKey::KEY_S, KeyboardKey::KEY_DOWN].into(),
            left: [KeyboardKey::KEY_A, KeyboardKey::KEY_LEFT].into(),
            right: [KeyboardKey::KEY_D, KeyboardKey::KEY_RIGHT].into(),
            a: [KeyboardKey::KEY_J, KeyboardKey::KEY_Z].into(),
            b: [KeyboardKey::KEY_K, KeyboardKey::KEY_C].into(),
            start: [KeyboardKey::KEY_L, KeyboardKey::KEY_X].into(),
            select: [KeyboardKey::KEY_SEMICOLON, KeyboardKey::KEY_Z].into(),
            escape: [KeyboardKey::KEY_ESCAPE].into(),
            speed_up: [KeyboardKey::KEY_LEFT_SHIFT].into(),
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

#[derive(Debug, Default, Copy, Clone)]
pub struct MouseButtonArea {
    pub x: i32,
    pub y: i32,
    pub width: i32,
    pub height: i32,
}

impl MouseButtonArea {
    pub fn new(x: i32, y: i32, width: i32, height: i32) -> Self { Self { x, y, width, height } }

    #[inline(always)]
    pub fn contains(&self, mouse: Vector2) -> bool {
        let (mx, my) = (mouse.x as i32, mouse.y as i32);
        mx >= self.x && mx < self.x + self.width && my >= self.y && my < self.y + self.height
    }

    #[inline(always)]
    pub fn is_hovered(&self, rl: &RaylibHandle) -> bool { self.contains(rl.get_mouse_position()) }

    #[inline(always)]
    pub fn is_pressed(&self, rl: &RaylibHandle, mouse_button: MouseButton) -> bool {
        rl.is_mouse_button_down(mouse_button) && self.contains(rl.get_mouse_position())
    }
}

#[derive(Debug, Default, Copy, Clone)]
pub struct InputMouseTriggers {
    pub up: MouseButtonArea,
    pub down: MouseButtonArea,
    pub left: MouseButtonArea,
    pub right: MouseButtonArea,
    pub a: MouseButtonArea,
    pub b: MouseButtonArea,
    pub start: MouseButtonArea,
    pub select: MouseButtonArea,
    pub escape: Option<MouseButtonArea>,
    pub speed_up: Option<MouseButtonArea>,
}

impl ToInputState for InputMouseTriggers {
    fn to_input(&self, rl: &RaylibHandle) -> InputState {
        let mut state = InputState::default();
        let mouse_pos = rl.get_mouse_position();
        let mouse_down = rl.is_mouse_button_down(MouseButton::MOUSE_BUTTON_LEFT);
        let touch_count = rl.get_touch_point_count();

        // Recopilamos todas las posiciones activas UNA
        let active_positions: Vec<Vector2> = (0..touch_count)
            .map(|i| rl.get_touch_position(i))
            .chain(std::iter::once(mouse_pos).filter(|_| mouse_down))
            .collect();

        let required: &mut [(&mut bool, &MouseButtonArea)] = &mut [
            (&mut state.up, &self.up),
            (&mut state.down, &self.down),
            (&mut state.left, &self.left),
            (&mut state.right, &self.right),
            (&mut state.a, &self.a),
            (&mut state.b, &self.b),
            (&mut state.start, &self.start),
            (&mut state.select, &self.select),
        ];

        // iter active positions only once and set all required states in one pass
        'outer: for pos in &active_positions {
            for (pressed, area) in required.iter_mut() {
                if !**pressed && area.contains(*pos) {
                    **pressed = true;
                }
            }

            if required.iter().all(|(pressed, _)| **pressed) {
                break 'outer;
            }
        }

        if let Some(area) = &self.escape {
            state.escape = active_positions.iter().any(|p| area.contains(*p));
        }
        if let Some(area) = &self.speed_up {
            state.speed_up = active_positions.iter().any(|p| area.contains(*p));
        }

        state
    }
}

// sneak peek of future GPIO support
#[derive(Debug, Default, Copy, Clone)]
pub struct _InputGpioTriggers {}

impl InputState {
    pub fn merge(rl: &RaylibHandle, states: &[&dyn ToInputState]) -> InputState {
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

macro_rules! impl_input_methods {
    ($($name:ident),*) => {
        $(
            paste::paste! {
                #[inline(always)]
                pub fn [<is_held_ $name>](&self) -> bool {
                    self.current.$name
                }

                #[inline(always)]
                pub fn [<is_pressed_ $name>](&self) -> bool {
                    self.current.$name && !self.previous.$name
                }

                #[inline(always)]
                pub fn [<is_repeated_ $name>](&self, dt: f32) -> bool {
                    self.[<is_pressed_ $name>]() || (self.current.$name && self.check_repeat(dt))
                }
            }
        )*
    };
}

#[derive(Debug, Clone)]
pub struct InputManager {
    pub key_triggers: InputKeyTriggers,
    pub mouse_triggers: Option<InputMouseTriggers>,
    pub gpio_triggers: Option<_InputGpioTriggers>,
    pub current: InputState,
    pub previous: InputState,
    pub repeat_timer: f32,
    pub debounce_timer: f32,
}

impl Default for InputManager {
    fn default() -> Self { Self::new(0.08, None, None, None) }
}

impl InputManager {
    pub fn new(
        debounce: f32,
        key_triggers: Option<InputKeyTriggers>,
        mouse_triggers: Option<InputMouseTriggers>,
        gpio_triggers: Option<_InputGpioTriggers>,
    ) -> Self {
        Self {
            key_triggers: key_triggers.unwrap_or_default(),
            mouse_triggers,
            gpio_triggers,
            current: InputState::default(),
            previous: InputState::default(),
            repeat_timer: 0.0,
            debounce_timer: debounce,
        }
    }

    pub fn update(&mut self, rl: &RaylibHandle, dt: f32) {
        if self.debounce_timer > 0.0 {
            self.debounce_timer -= dt;
            self.current = self.get_input(rl);
            self.previous = self.current;
            return;
        }

        self.previous = self.current;
        self.current = self.get_input(rl);

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

    fn get_input(&self, rl: &RaylibHandle) -> InputState {
        let mut current = self.key_triggers.to_input(rl);
        if let Some(mouse) = &self.mouse_triggers {
            let mouse_state = mouse.to_input(rl);
            current.up |= mouse_state.up;
            current.down |= mouse_state.down;
            current.left |= mouse_state.left;
            current.right |= mouse_state.right;
            current.a |= mouse_state.a;
            current.b |= mouse_state.b;
            current.start |= mouse_state.start;
            current.select |= mouse_state.select;
            current.escape |= mouse_state.escape;
            current.speed_up |= mouse_state.speed_up;
        }
        current
    }

    pub fn state(&self) -> InputState { self.current }

    impl_input_methods!(up, down, left, right, a, b, start, select, escape, speed_up);

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
