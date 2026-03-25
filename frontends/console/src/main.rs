use gbeed_core::prelude::*;
use raylib::prelude::*;
use std::{fs, path::PathBuf};

const ROMS_DIR: &str = "/home/daniqss/roms";
const SAVE_DIR: &str = "/home/daniqss/saves";

const SCREEN_WIDTH: i32 = 240;
const SCREEN_HEIGHT: i32 = 240;

const PADDING_X: i32 = 5;
const PADDING_Y: i32 = 10;

const HEADER_H: i32 = 20;
const FOOTER_H: i32 = 20;
const SECTION_PAD: i32 = 6;

const SCROLLBAR_W: i32 = 4;
const SCROLLBAR_X: i32 = SCREEN_WIDTH - PADDING_X - SCROLLBAR_W;

const ITEM_H: i32 = 14;
const FONT_SIZE: i32 = 10;

// palette
const FOREGROUND: Color = Color::new(196, 207, 161, 255);
const PRIMARY: Color = Color::new(139, 149, 109, 255);
const SECONDARY: Color = Color::new(77, 83, 60, 255);
const BACKGROUND: Color = Color::new(31, 31, 31, 255);

enum EmulatorState {
    BootScreen,
    SelectionMenu,
    Emulation,
    GameMenu,
    SettingsMenu,
}

impl EmulatorState {
    fn get_hint(&self) -> Option<&'static str> {
        use EmulatorState::*;

        match self {
            BootScreen => None,
            SelectionMenu => Some("w/s move  enter select"),
            Emulation => None,
            GameMenu => Some("r resume  s save  l load  q quit"),
            SettingsMenu => Some("s save settings  q back"),
        }
    }
}

struct EmulatorApp {
    state: EmulatorState,
    gb: Option<Dmg>,
    // controller: ConsoleController,
    rom_path: Option<PathBuf>,
    save_path: Option<PathBuf>,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let (mut rl, thread) = raylib::init()
        .size(SCREEN_WIDTH, SCREEN_HEIGHT)
        .title("gbeed")
        .build();
    rl.set_target_fps(60);

    let mut roms: Vec<PathBuf> = fs::read_dir(ROMS_DIR)?
        .filter_map(|e| e.ok())
        .filter(|e| e.path().is_file() && e.path().extension().is_some_and(|x| x == "gb" || x == "gbc"))
        .map(|e| e.path())
        .collect();
    roms.sort();

    let mut selected: usize = 0;
    let mut scroll_offset: usize = 0;

    let visible_count = ((selector_bottom() - selector_top()) / ITEM_H) as usize;

    let mut chosen: Option<PathBuf> = None;

    let mut repeat_timer: f32 = 0.0;
    const REPEAT_DELAY: f32 = 0.3;
    const REPEAT_RATE: f32 = 0.08;

    while !rl.window_should_close() && chosen.is_none() {
        let dt = rl.get_frame_time();

        let up_held = rl.is_key_down(KeyboardKey::KEY_W) || rl.is_key_down(KeyboardKey::KEY_UP);
        let down_held = rl.is_key_down(KeyboardKey::KEY_S) || rl.is_key_down(KeyboardKey::KEY_DOWN);

        let up_pressed = rl.is_key_pressed(KeyboardKey::KEY_W) || rl.is_key_pressed(KeyboardKey::KEY_UP);
        let down_pressed = rl.is_key_pressed(KeyboardKey::KEY_S) || rl.is_key_pressed(KeyboardKey::KEY_DOWN);

        let mut move_up = up_pressed;
        let mut move_down = down_pressed;

        if up_held || down_held {
            repeat_timer += dt;
            if up_pressed || down_pressed {
                repeat_timer = 0.0;
            }
            if repeat_timer >= REPEAT_DELAY {
                let ticks = ((repeat_timer - REPEAT_DELAY) / REPEAT_RATE) as usize;
                let prev = ((repeat_timer - REPEAT_DELAY - dt.max(0.0)) / REPEAT_RATE) as usize;
                if ticks > prev {
                    if up_held {
                        move_up = true;
                    }
                    if down_held {
                        move_down = true;
                    }
                }
            }
        } else {
            repeat_timer = 0.0;
        }

        if !roms.is_empty() {
            if move_up && selected > 0 {
                selected -= 1;
            }
            if move_down && selected + 1 < roms.len() {
                selected += 1;
            }

            if selected < scroll_offset {
                scroll_offset = selected;
            }
            if selected >= scroll_offset + visible_count {
                scroll_offset = selected + 1 - visible_count;
            }
        }

        if (rl.is_key_pressed(KeyboardKey::KEY_ENTER) || rl.is_key_pressed(KeyboardKey::KEY_J))
            && !roms.is_empty()
        {
            chosen = Some(roms[selected].clone());
        }

        // draw selection menu
        rl.draw(&thread, |mut d| {
            d.clear_background(BACKGROUND);

            draw_header(&mut d, roms.len());
            draw_selector(&mut d, &roms, selected, scroll_offset);
            draw_footer(&mut d);
        });
    }

    if let Some(rom) = chosen {
        println!("{}", rom.display());
    }

    Ok(())
}

fn selector_top() -> i32 { PADDING_Y + HEADER_H + SECTION_PAD }
fn selector_bottom() -> i32 { SCREEN_HEIGHT - PADDING_Y - FOOTER_H - SECTION_PAD }

/// Draws the header with the title and rom count
fn draw_header(d: &mut RaylibDrawHandle, rom_count: usize) {
    let y = PADDING_Y + (HEADER_H - FONT_SIZE) / 2;

    d.draw_text("GBEED", PADDING_X, y, FONT_SIZE, FOREGROUND);

    let count_str = format!("{} roms", rom_count);
    let count_w = d.measure_text(&count_str, FONT_SIZE);
    d.draw_text(
        &count_str,
        SCREEN_WIDTH - PADDING_X - count_w,
        y,
        FONT_SIZE,
        FOREGROUND,
    );

    d.draw_line(
        PADDING_X,
        PADDING_Y + HEADER_H,
        SCREEN_WIDTH - PADDING_X,
        PADDING_Y + HEADER_H,
        SECONDARY,
    );
}

/// Draws a ROM list that can be scrolled through to select a game
fn draw_selector(d: &mut RaylibDrawHandle, roms: &[PathBuf], selected: usize, scroll_offset: usize) {
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
        let display = truncate_name(name, max_chars);

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
fn draw_footer(d: &mut RaylibDrawHandle) {
    let sep_y = SCREEN_HEIGHT - PADDING_Y - FOOTER_H;
    d.draw_line(0, sep_y, SCREEN_WIDTH, sep_y, SECONDARY);

    let hint = "w/s move  enter select";
    let hint_w = d.measure_text(hint, FONT_SIZE - 1);
    let y = sep_y + (FOOTER_H - (FONT_SIZE - 1)) / 2;
    d.draw_text(hint, (SCREEN_WIDTH - hint_w) / 2, y, FONT_SIZE - 1, PRIMARY);
}

fn truncate_name(name: &str, max_chars: usize) -> String {
    if name.len() <= max_chars {
        name.to_string()
    } else {
        format!("{}...", &name[..max_chars.saturating_sub(3)])
    }
}
