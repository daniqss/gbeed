mod controller;

use gbeed_core::prelude::*;
use gbeed_raylib_common::texture::Texture;
use raylib::prelude::*;
use std::{
    fs, io,
    path::{Path, PathBuf},
};

use crate::controller::ConsoleController;

const ROMS_DIR: &str = "/home/daniqss/roms";
const _SAVE_DIR: &str = "/home/daniqss/saves";

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
    BootScreen {
        timer: f32,
    },
    SelectionMenu {
        roms: Vec<PathBuf>,
        selected: usize,
        scroll_offset: usize,
        repeat_timer: f32,
    },
    Emulation,
    GameMenu,
    SettingsMenu,
}

impl EmulatorState {
    fn get_hint(&self) -> Option<&'static str> {
        use EmulatorState::*;
        match self {
            BootScreen { .. } => None,
            SelectionMenu { .. } => Some("w/s move  enter select"),
            Emulation => None,
            GameMenu => Some("r resume  s save  l load  q quit"),
            SettingsMenu => Some("s save settings  q back"),
        }
    }
}

struct EmulatorApp {
    state: EmulatorState,
    gb: Option<Dmg>,
    rom_path: Option<PathBuf>,
    save_path: Option<PathBuf>,
    controller: ConsoleController,
}

impl EmulatorApp {
    pub fn new(mut rl: RaylibHandle, thread: RaylibThread) -> Self {
        let screen = Texture::new(
            &mut rl,
            &thread,
            DMG_SCREEN_WIDTH as i32,
            DMG_SCREEN_HEIGHT as i32,
        );

        Self {
            state: EmulatorState::BootScreen { timer: 0.0 },
            gb: None,
            rom_path: None,
            save_path: None,

            controller: ConsoleController { rl, thread, screen },
        }
    }

    pub fn update(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        let rl = &mut self.controller.rl;
        let dt = rl.get_frame_time();

        let next_state = match &mut self.state {
            EmulatorState::SelectionMenu {
                roms,
                selected,
                scroll_offset,
                repeat_timer,
            } => {
                let up_held = rl.is_key_down(KeyboardKey::KEY_W) || rl.is_key_down(KeyboardKey::KEY_UP);
                let down_held = rl.is_key_down(KeyboardKey::KEY_S) || rl.is_key_down(KeyboardKey::KEY_DOWN);
                let up_pressed =
                    rl.is_key_pressed(KeyboardKey::KEY_W) || rl.is_key_pressed(KeyboardKey::KEY_UP);
                let down_pressed =
                    rl.is_key_pressed(KeyboardKey::KEY_S) || rl.is_key_pressed(KeyboardKey::KEY_DOWN);

                let mut move_up = up_pressed;
                let mut move_down = down_pressed;

                const REPEAT_DELAY: f32 = 0.3;
                const REPEAT_RATE: f32 = 0.08;

                if up_held || down_held {
                    *repeat_timer += dt;
                    if up_pressed || down_pressed {
                        *repeat_timer = 0.0;
                    }
                    if *repeat_timer >= REPEAT_DELAY {
                        let ticks = ((*repeat_timer - REPEAT_DELAY) / REPEAT_RATE) as usize;
                        let prev = ((*repeat_timer - REPEAT_DELAY - dt.max(0.0)) / REPEAT_RATE) as usize;
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
                    *repeat_timer = 0.0;
                }

                let visible_count = ((selector_bottom() - selector_top()) / ITEM_H) as usize;

                if !roms.is_empty() {
                    if move_up && *selected > 0 {
                        *selected -= 1;
                    }
                    if move_down && *selected + 1 < roms.len() {
                        *selected += 1;
                    }

                    if *selected < *scroll_offset {
                        *scroll_offset = *selected;
                    }
                    if *selected >= *scroll_offset + visible_count {
                        *scroll_offset = *selected + 1 - visible_count;
                    }
                }

                if (rl.is_key_pressed(KeyboardKey::KEY_ENTER) || rl.is_key_pressed(KeyboardKey::KEY_J))
                    && !roms.is_empty()
                {
                    self.rom_path = Some(roms[*selected].clone());
                    Some(EmulatorState::Emulation)
                } else {
                    None
                }
            }
            EmulatorState::Emulation => {
                if let Some(gb) = &mut self.gb {
                    gb.run(&mut self.controller)?;
                    pub fn read_key_input(rl: &RaylibHandle) -> ButtonStates {
                        ButtonStates {
                            up: rl.is_key_down(KeyboardKey::KEY_UP) || rl.is_key_down(KeyboardKey::KEY_W),
                            down: rl.is_key_down(KeyboardKey::KEY_DOWN) || rl.is_key_down(KeyboardKey::KEY_S),
                            left: rl.is_key_down(KeyboardKey::KEY_LEFT) || rl.is_key_down(KeyboardKey::KEY_A),
                            right: rl.is_key_down(KeyboardKey::KEY_RIGHT)
                                || rl.is_key_down(KeyboardKey::KEY_D),
                            a: rl.is_key_down(KeyboardKey::KEY_Z) || rl.is_key_down(KeyboardKey::KEY_J),
                            b: rl.is_key_down(KeyboardKey::KEY_X) || rl.is_key_down(KeyboardKey::KEY_K),
                            start: rl.is_key_down(KeyboardKey::KEY_L),
                            select: rl.is_key_down(KeyboardKey::KEY_LEFT_SHIFT)
                                || rl.is_key_down(KeyboardKey::KEY_SEMICOLON),
                        }
                    }
                } else if let Some(game_path) = &self.rom_path {
                    let save_path = save_path_from_rom(game_path.to_str().unwrap_or_default());
                    self.save_path = Some(save_path.clone());

                    let game_data = fs::read(game_path).map_err(|e| {
                        io::Error::new(
                            io::ErrorKind::NotFound,
                            format!("Failed to read game ROM at {game_path:?}: {e}"),
                        )
                    })?;

                    // attempt to read the save file, if cartridge doesn't support saves it will discard it
                    let save = match fs::read(&save_path) {
                        Ok(data) => Some(data),
                        Err(e) if e.kind() == io::ErrorKind::NotFound => None,
                        Err(e) => {
                            return Err(Box::new(io::Error::other(format!(
                                "Failed to read save file at {:?}: {e}",
                                self.save_path
                            ))))
                        }
                    };

                    let game = Cartridge::new(&game_data, save).map_err(|e| {
                        io::Error::new(
                            io::ErrorKind::InvalidData,
                            format!("Failed to create cartridge from ROM at {game_path:?}: {e}"),
                        )
                    })?;

                    self.gb = Some(Dmg::new(game, None));
                }

                None
            }
            EmulatorState::GameMenu => None,
            EmulatorState::BootScreen { timer } => {
                *timer += dt;

                if *timer >= 2.0 {
                    Some(EmulatorState::SelectionMenu {
                        roms: fs::read_dir(ROMS_DIR)?
                            .filter_map(|e| e.ok())
                            .filter(|e| {
                                e.path().is_file()
                                    && e.path().extension().is_some_and(|x| x == "gb" || x == "gbc")
                            })
                            .map(|e| e.path())
                            .collect(),
                        selected: 0,
                        scroll_offset: 0,
                        repeat_timer: 0.0,
                    })
                } else {
                    None
                }
            }
            EmulatorState::SettingsMenu => None,
        };

        if let Some(state) = next_state {
            self.state = state;
        }

        Ok(())
    }

    pub fn draw(&mut self) {
        self.controller.rl.draw(&self.controller.thread, |mut d| {
            d.clear_background(BACKGROUND);

            match &self.state {
                EmulatorState::SelectionMenu {
                    roms,
                    selected,
                    scroll_offset,
                    ..
                } => {
                    // draw selection menu
                    draw_header(&mut d, roms.len());
                    draw_selector(&mut d, roms, *selected, *scroll_offset);
                }
                EmulatorState::Emulation => {
                    d.draw_texture_pro(
                        &self.controller.screen.texture,
                        Rectangle::new(0.0, 0.0, DMG_SCREEN_WIDTH as f32, DMG_SCREEN_HEIGHT as f32),
                        Rectangle::new(0.0, 0.0, SCREEN_WIDTH as f32, SCREEN_HEIGHT as f32),
                        Vector2::new(0.0, 0.0),
                        0.0,
                        Color::WHITE,
                    );
                }
                EmulatorState::GameMenu => {}
                EmulatorState::BootScreen { .. } => {}
                EmulatorState::SettingsMenu => {}
            }

            if let Some(hint) = self.state.get_hint() {
                draw_footer(&mut d, hint);
            }
        });
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let (mut rl, thread) = raylib::init()
        .size(SCREEN_WIDTH, SCREEN_HEIGHT)
        .title("gbeed")
        .build();
    rl.set_target_fps(60);

    let mut app = EmulatorApp::new(rl, thread);

    while !app.controller.rl.window_should_close() {
        app.update()?;
        app.draw();
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
fn draw_footer(d: &mut RaylibDrawHandle, hint: &str) {
    let sep_y = SCREEN_HEIGHT - PADDING_Y - FOOTER_H;
    d.draw_line(0, sep_y, SCREEN_WIDTH, sep_y, SECONDARY);

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

fn save_path_from_rom(rom_path: &str) -> PathBuf {
    let path = Path::new(rom_path);
    match path.extension().and_then(|e| e.to_str()) {
        Some("gb" | "gbc") => path.with_extension("sav"),
        _ => path.with_added_extension("sav"),
    }
}
