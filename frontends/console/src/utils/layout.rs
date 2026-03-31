use std::path::PathBuf;

use gbeed_raylib_common::{BACKGROUND, FOREGROUND, PRIMARY, SECONDARY};
use raylib::prelude::*;

use crate::{scenes::EmulatorState, utils};

pub const SCREEN_WIDTH: i32 = 240;
pub const SCREEN_HEIGHT: i32 = 240;

pub const PADDING_X: i32 = 5;
pub const PADDING_Y: i32 = 10;

pub const HEADER_H: i32 = 20;
pub const FOOTER_H: i32 = 20;
pub const SECTION_PAD: i32 = 6;

pub const CONTENT_TOP: i32 = PADDING_Y + HEADER_H;
pub const CONTENT_BOTTOM: i32 = SCREEN_HEIGHT - PADDING_Y - FOOTER_H;
pub const CONTENT_HEIGHT: i32 = CONTENT_BOTTOM - CONTENT_TOP;

const SCROLLBAR_W: i32 = 4;
const SCROLLBAR_X: i32 = SCREEN_WIDTH - PADDING_X - SCROLLBAR_W;

pub const ITEM_H: i32 = 14;
const FONT_SIZE: i32 = 10;

pub fn selector_top() -> i32 { CONTENT_TOP + SECTION_PAD }
pub fn selector_bottom() -> i32 { CONTENT_BOTTOM - SECTION_PAD }

/// Draws the header with the tabs
pub fn draw_header(d: &mut RaylibDrawHandle, current_state: &EmulatorState) {
    let (sel_active, game_active, set_active) = match current_state {
        EmulatorState::SelectionMenu(_) => (true, false, false),
        EmulatorState::GameMenu(_) => (false, true, false),
        EmulatorState::SettingsMenu(_) => (false, false, true),
        EmulatorState::Emulation(_) => return,
    };

    let tab_style = |active: bool| {
        if active {
            (FONT_SIZE + 2, FOREGROUND)
        } else {
            (FONT_SIZE, PRIMARY)
        }
    };

    d.draw_rectangle(0, 0, SCREEN_WIDTH, CONTENT_TOP, BACKGROUND);

    let (sel_size, sel_color) = tab_style(sel_active);
    let (game_size, game_color) = tab_style(game_active);
    let (set_size, set_color) = tab_style(set_active);

    let y = PADDING_Y + (HEADER_H - FONT_SIZE) / 2;

    let sel_text = "Selection";
    let game_text = "Game";
    let set_text = "Settings";

    let game_w = d.measure_text(game_text, game_size);
    let set_w = d.measure_text(set_text, set_size);

    d.draw_text(sel_text, PADDING_X, y, sel_size, sel_color);
    d.draw_text(game_text, (SCREEN_WIDTH - game_w) / 2, y, game_size, game_color);
    d.draw_text(set_text, SCREEN_WIDTH - PADDING_X - set_w, y, set_size, set_color);

    d.draw_line(
        PADDING_X,
        PADDING_Y + HEADER_H,
        SCREEN_WIDTH - PADDING_X,
        PADDING_Y + HEADER_H,
        SECONDARY,
    );
}

/// Draws a ROM list that can be scrolled through to select a game
pub fn draw_selector(d: &mut RaylibDrawHandle, roms: &[PathBuf], selected: usize, scroll_offset: usize) {
    let top = selector_top();
    let bottom = selector_bottom();
    let height = bottom - top;
    let visible_count = (height / ITEM_H) as usize;

    if roms.is_empty() {
        d.draw_text("no roms found", PADDING_X, top + SECTION_PAD, FONT_SIZE, PRIMARY);
        return;
    }

    d.draw_rectangle(SCROLLBAR_X, top, SCROLLBAR_W, height, SECONDARY);

    let total = roms.len();
    let thumb_h: i32 = 6;
    let thumb_y = if total <= 1 {
        top
    } else {
        top + (((selected as f32) / (total - 1) as f32) * (height - thumb_h) as f32) as i32
    };
    d.draw_rectangle(SCROLLBAR_X, thumb_y, SCROLLBAR_W, thumb_h, FOREGROUND);

    let text_area_w = SCROLLBAR_X - PADDING_X - 4;
    let max_chars = (text_area_w / (FONT_SIZE / 2).max(1)) as usize;

    for i in 0..visible_count {
        let idx = scroll_offset + i;
        if idx >= roms.len() {
            break;
        }

        let y = top + i as i32 * ITEM_H;
        let name = roms[idx]
            .file_name()
            .and_then(|s| s.to_str())
            .unwrap_or("MissingNo");
        let display = utils::truncate_name(name, max_chars);

        if idx == selected {
            d.draw_rectangle(0, y, SCROLLBAR_X - 2, ITEM_H, FOREGROUND);
            d.draw_rectangle(0, y, 2, ITEM_H, PRIMARY);
            d.draw_text("$", PADDING_X, y + 2, FONT_SIZE, SECONDARY);
            d.draw_text(&display, PADDING_X + 12, y + 2, FONT_SIZE, BACKGROUND);
        } else {
            d.draw_text(&display, PADDING_X + 12, y + 2, FONT_SIZE, PRIMARY);
        }
    }
}

/// Draws the selection menu footer with control hints
pub fn draw_footer(d: &mut RaylibDrawHandle, state: &EmulatorState) {
    // TODO: real hints
    let hint = match state {
        EmulatorState::SelectionMenu(_) => "w/s move  enter select",
        EmulatorState::GameMenu(_) => "r resume  s save  l load  q quit",
        EmulatorState::SettingsMenu(_) => "s save settings  q back",
        EmulatorState::Emulation(_) => return,
    };

    d.draw_rectangle(0, CONTENT_BOTTOM, SCREEN_WIDTH, SCREEN_HEIGHT - CONTENT_BOTTOM, BACKGROUND);

    let sep_y = CONTENT_BOTTOM;
    d.draw_line(0, sep_y, SCREEN_WIDTH, sep_y, SECONDARY);

    let hint_w = d.measure_text(hint, FONT_SIZE - 1);
    let y = sep_y + (FOOTER_H - (FONT_SIZE - 1)) / 2;
    d.draw_text(hint, (SCREEN_WIDTH - hint_w) / 2, y, FONT_SIZE - 1, PRIMARY);
}
