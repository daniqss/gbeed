use gbeed_core::prelude::*;
use raylib::prelude::*;

use crate::renderer::{
    RaylibRenderer, HEADER_HEIGHT, PANEL_PADDING, SCALED_SCREEN_HEIGHT, SCALED_SCREEN_WIDTH,
};

#[derive(Default, Copy, Clone)]
pub struct ButtonStates {
    pub up: bool,
    pub down: bool,
    pub left: bool,
    pub right: bool,
    pub a: bool,
    pub b: bool,
    pub start: bool,
    pub select: bool,
}

impl ButtonStates {
    pub fn merge(&mut self, other: &Self) {
        self.up |= other.up;
        self.down |= other.down;
        self.left |= other.left;
        self.right |= other.right;
        self.a |= other.a;
        self.b |= other.b;
        self.start |= other.start;
        self.select |= other.select;
    }
}

pub fn update(renderer: &mut RaylibRenderer, joypad: &mut Joypad) {
    let mut input = read_key_input(&renderer.rl);
    let mouse_input = read_mouse_input(&renderer.rl);

    input.merge(&mouse_input);

    apply_joypad(&input, joypad);

    renderer.buttons = input;

    if renderer.rl.is_key_pressed(KeyboardKey::KEY_TAB)
        || renderer.rl.is_key_pressed(KeyboardKey::KEY_Q)
        || fps_btn_clicked(&renderer.rl)
    {
        renderer.cycle_fps();
    }
}

pub fn read_key_input(rl: &RaylibHandle) -> ButtonStates {
    ButtonStates {
        up: rl.is_key_down(KeyboardKey::KEY_UP) || rl.is_key_down(KeyboardKey::KEY_W),
        down: rl.is_key_down(KeyboardKey::KEY_DOWN) || rl.is_key_down(KeyboardKey::KEY_S),
        left: rl.is_key_down(KeyboardKey::KEY_LEFT) || rl.is_key_down(KeyboardKey::KEY_A),
        right: rl.is_key_down(KeyboardKey::KEY_RIGHT) || rl.is_key_down(KeyboardKey::KEY_D),
        a: rl.is_key_down(KeyboardKey::KEY_Z) || rl.is_key_down(KeyboardKey::KEY_J),
        b: rl.is_key_down(KeyboardKey::KEY_X) || rl.is_key_down(KeyboardKey::KEY_K),
        start: rl.is_key_down(KeyboardKey::KEY_L),
        select: rl.is_key_down(KeyboardKey::KEY_LEFT_SHIFT) || rl.is_key_down(KeyboardKey::KEY_SEMICOLON),
    }
}

pub fn read_mouse_input(rl: &RaylibHandle) -> ButtonStates {
    let game_x = PANEL_PADDING;
    let game_y = PANEL_PADDING + HEADER_HEIGHT;
    let screen_center_x = game_x + SCALED_SCREEN_WIDTH / 2;
    let controls_y = game_y + SCALED_SCREEN_HEIGHT + PANEL_PADDING * 2;

    let dpad_x = screen_center_x - 160;
    let dpad_y = controls_y + 50;
    let dpad_arm = 28_i32;
    let dpad_size = 17_i32;

    let start_select_center_x = screen_center_x;
    let start_select_width = 60_i32;
    let start_select_gap = 18_i32;
    let start_select_total = start_select_width * 2 + start_select_gap;
    let start_select_x = start_select_center_x - start_select_total / 2;
    let start_select_y = dpad_y - 10;

    let action_buttons_x = screen_center_x + 160;
    let action_buttons_y = controls_y + 24;
    let action_button_size = 36_i32;

    let is_down = rl.is_mouse_button_down(MouseButton::MOUSE_BUTTON_LEFT);

    ButtonStates {
        up: is_down && is_mouse_over_center(rl, dpad_x, dpad_y - dpad_arm, dpad_size),
        down: is_down && is_mouse_over_center(rl, dpad_x, dpad_y + dpad_arm, dpad_size),
        left: is_down && is_mouse_over_center(rl, dpad_x - dpad_arm, dpad_y, dpad_size),
        right: is_down && is_mouse_over_center(rl, dpad_x + dpad_arm, dpad_y, dpad_size),
        a: is_down
            && is_mouse_over(
                rl,
                action_buttons_x - action_button_size / 2 + 24,
                action_buttons_y,
                action_button_size,
                action_button_size,
            ),
        b: is_down
            && is_mouse_over(
                rl,
                action_buttons_x - action_button_size / 2 - 24,
                action_buttons_y + 44,
                action_button_size,
                action_button_size,
            ),
        select: is_down && is_mouse_over(rl, start_select_x, start_select_y, start_select_width, 20),
        start: is_down
            && is_mouse_over(
                rl,
                start_select_x + start_select_width + start_select_gap,
                start_select_y,
                start_select_width,
                20,
            ),
    }
}

pub fn fps_btn_clicked(rl: &RaylibHandle) -> bool {
    let game_x = PANEL_PADDING;
    let game_y = PANEL_PADDING + HEADER_HEIGHT;
    let screen_center_x = game_x + SCALED_SCREEN_WIDTH / 2;
    let controls_y = game_y + SCALED_SCREEN_HEIGHT + PANEL_PADDING * 2;

    let fps_button_width = 118_i32;
    let fps_button_height = 26_i32;
    let fps_button_x = screen_center_x - fps_button_width / 2;
    let fps_button_y = controls_y - 20;

    rl.is_mouse_button_pressed(MouseButton::MOUSE_BUTTON_LEFT)
        && is_mouse_over(
            rl,
            fps_button_x,
            fps_button_y,
            fps_button_width,
            fps_button_height,
        )
}

fn is_mouse_over(rl: &RaylibHandle, x: i32, y: i32, width: i32, height: i32) -> bool {
    let mp = rl.get_mouse_position();
    (mp.x as i32) >= x && (mp.x as i32) < x + width && (mp.y as i32) >= y && (mp.y as i32) < y + height
}

fn is_mouse_over_center(rl: &RaylibHandle, cx: i32, cy: i32, size: i32) -> bool {
    let mp = rl.get_mouse_position();
    (mp.x as i32) >= cx - size
        && (mp.x as i32) < cx + size
        && (mp.y as i32) >= cy - size
        && (mp.y as i32) < cy + size
}

pub fn apply_joypad(s: &ButtonStates, joypad: &mut Joypad) {
    joypad.button_down(JoypadButton::Up, s.up);
    joypad.button_down(JoypadButton::Down, s.down);
    joypad.button_down(JoypadButton::Left, s.left);
    joypad.button_down(JoypadButton::Right, s.right);
    joypad.button_down(JoypadButton::A, s.a);
    joypad.button_down(JoypadButton::B, s.b);
    joypad.button_down(JoypadButton::Start, s.start);
    joypad.button_down(JoypadButton::Select, s.select);
}
