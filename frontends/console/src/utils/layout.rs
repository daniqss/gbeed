use gbeed_raylib_common::Palette;
use raylib::prelude::*;

use crate::scenes::EmulatorState;

pub const SCREEN_WIDTH: i32 = 240;
pub const SCREEN_HEIGHT: i32 = 240;

pub const PADDING_X: i32 = 5;
pub const PADDING_Y: i32 = 10;

pub const HEADER_H: i32 = 20;
pub const FOOTER_H: i32 = 20;
pub const SECTION_PAD: i32 = 6;

pub const CONTENT_TOP: i32 = PADDING_Y + HEADER_H;
pub const CONTENT_BOTTOM: i32 = SCREEN_HEIGHT - PADDING_Y - FOOTER_H;

pub const SCROLLBAR_W: i32 = 4;
pub const SCROLLBAR_X: i32 = SCREEN_WIDTH - PADDING_X - SCROLLBAR_W;

pub const ITEM_H: i32 = 14;
pub const FONT_SIZE: i32 = 10;

pub const VISIBLE_TOP: i32 = CONTENT_TOP + SECTION_PAD;
pub const VISIBLE_BOTTOM: i32 = CONTENT_BOTTOM - SECTION_PAD;

/// Draws the header with the tabs
pub fn draw_header(d: &mut RaylibDrawHandle, current_state: &EmulatorState, palette: Palette) {
    let (sel_active, game_active, set_active) = match current_state {
        EmulatorState::SelectionMenu(_) => (true, false, false),
        EmulatorState::GameMenu(_) => (false, true, false),
        EmulatorState::SettingsMenu(_) => (false, false, true),
        EmulatorState::Emulation(_) => return,
    };

    let tab_style = |active: bool| {
        if active {
            (FONT_SIZE + 2, palette.foreground())
        } else {
            (FONT_SIZE, palette.primary())
        }
    };

    d.draw_rectangle(0, 0, SCREEN_WIDTH, CONTENT_TOP, palette.background());

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
        palette.secondary(),
    );
}

/// Draws a scrollable menu list with the given items, highlighting the selected one
pub fn draw_menu_list(
    d: &mut RaylibDrawHandle,
    items: &[(&str, &str)],
    selected: usize,
    scroll_offset: usize,
    palette: Palette,
) {
    let height = VISIBLE_TOP - VISIBLE_BOTTOM;
    let visible_count = (height / ITEM_H) as usize;

    d.draw_rectangle(
        SCROLLBAR_X,
        VISIBLE_BOTTOM,
        SCROLLBAR_W,
        height,
        palette.secondary(),
    );

    let total = items.len();
    let thumb_h: i32 = 6;
    let thumb_y = if total <= 1 {
        VISIBLE_TOP
    } else {
        VISIBLE_TOP + (((selected as f32) / (total - 1) as f32) * (height - thumb_h) as f32) as i32
    };
    d.draw_rectangle(SCROLLBAR_X, thumb_y, SCROLLBAR_W, thumb_h, palette.foreground());

    for i in 0..visible_count {
        let idx = scroll_offset + i;
        if idx >= items.len() {
            break;
        }

        let y = VISIBLE_TOP + i as i32 * ITEM_H;
        let (label, value) = items[idx];
        let val_display = (!value.is_empty()).then(|| format!("< {} >", value));

        if idx == selected {
            d.draw_rectangle(0, y, SCROLLBAR_X - 2, ITEM_H, palette.foreground());
            d.draw_rectangle(0, y, 2, ITEM_H, palette.primary());
            d.draw_text("$", PADDING_X, y + 2, FONT_SIZE, palette.secondary());
            d.draw_text(label, PADDING_X + 12, y + 2, FONT_SIZE, palette.background());
            if let Some(ref val) = val_display {
                let val_w = d.measure_text(val, FONT_SIZE);
                d.draw_text(
                    val,
                    SCROLLBAR_X - val_w - 4,
                    y + 2,
                    FONT_SIZE,
                    palette.background(),
                );
            }
        } else {
            d.draw_text(label, PADDING_X + 12, y + 2, FONT_SIZE, palette.primary());
            if let Some(ref val) = val_display {
                let val_w = d.measure_text(val, FONT_SIZE);
                d.draw_text(
                    val,
                    SCROLLBAR_X - val_w - 4,
                    y + 2,
                    FONT_SIZE,
                    palette.secondary(),
                );
            }
        }
    }
}

/// Draws the selection menu footer with control hints
pub fn draw_footer(d: &mut RaylibDrawHandle, state: &EmulatorState, palette: Palette) {
    // TODO: real hints
    let hint = match state {
        EmulatorState::SelectionMenu(_) => "w/s move  enter select",
        EmulatorState::GameMenu(_) => "r resume  s save  l load  q quit",
        EmulatorState::SettingsMenu(_) => "s save settings  q back",
        EmulatorState::Emulation(_) => return,
    };

    d.draw_rectangle(
        0,
        CONTENT_BOTTOM,
        SCREEN_WIDTH,
        SCREEN_HEIGHT - CONTENT_BOTTOM,
        palette.background(),
    );

    let sep_y = CONTENT_BOTTOM;
    d.draw_line(0, sep_y, SCREEN_WIDTH, sep_y, palette.secondary());

    let hint_w = d.measure_text(hint, FONT_SIZE - 1);
    let y = sep_y + (FOOTER_H - (FONT_SIZE - 1)) / 2;
    d.draw_text(
        hint,
        (SCREEN_WIDTH - hint_w) / 2,
        y,
        FONT_SIZE - 1,
        palette.primary(),
    );
}
