use crate::utils::{BACKGROUND, FOREGROUND, PRIMARY, SECONDARY};
use raylib::prelude::*;

pub fn draw_fps_btn(d: &mut RaylibDrawHandle, center_x: i32, controls_y: i32, text: &str) {
    let w = 118;
    let h = 26;
    let x = center_x - w / 2;
    let y = controls_y - 20;
    d.draw_rectangle(x, y, w, h, BACKGROUND);
    d.draw_rectangle_lines(x, y, w, h, PRIMARY);
    let tw = d.measure_text(text, 12);
    d.draw_text(text, x + (w - tw) / 2, y + (h - 15) / 2, 12, PRIMARY);
}

#[allow(clippy::too_many_arguments)]
pub fn draw_pad_btn(
    d: &mut RaylibDrawHandle,
    cx: i32,
    cy: i32,
    size: i32,
    rot: f32,
    key: &str,
    pressed: bool,
    is_mobile: bool,
) {
    let (bg, fg, border) = if pressed {
        (PRIMARY, BACKGROUND, PRIMARY)
    } else {
        (BACKGROUND, FOREGROUND, SECONDARY)
    };
    let sf = size as f32;
    let pts = [(-sf, -sf), (sf, -sf), (sf, 0.0), (0.0, sf), (-sf, 0.0)];
    let rad = rot.to_radians();
    let (s, c) = rad.sin_cos();
    let v: Vec<Vector2> = pts
        .iter()
        .map(|&(x, y)| Vector2::new(cx as f32 + x * c - y * s, cy as f32 + x * s + y * c))
        .collect();

    d.draw_triangle(v[0], v[3], v[1], bg);
    d.draw_triangle(v[0], v[4], v[3], bg);
    d.draw_triangle(v[1], v[3], v[2], bg);
    for i in 0..5 {
        d.draw_line_v(v[i], v[(i + 1) % 5], border);
    }

    if !is_mobile {
        let fs = 11;
        let tw = d.measure_text(key, fs);
        d.draw_text(key, cx - tw / 2, cy - fs / 2, fs, fg);
    }
}

#[allow(clippy::too_many_arguments)]
pub fn draw_action_btn(
    d: &mut RaylibDrawHandle,
    x: i32,
    y: i32,
    size: i32,
    label: &str,
    key: &str,
    pressed: bool,
    is_mobile: bool,
) {
    let (bg, fg, border) = if pressed {
        (PRIMARY, BACKGROUND, PRIMARY)
    } else {
        (BACKGROUND, FOREGROUND, SECONDARY)
    };
    d.draw_rectangle(x, y, size, size, bg);
    d.draw_rectangle_lines(x, y, size, size, border);
    let fs = if is_mobile { 28 } else { 18 };
    let tw = d.measure_text(label, fs);
    d.draw_text(label, x + (size - tw) / 2, y + (size - fs) / 2, fs, fg);
    if !is_mobile {
        let kfs = 8;
        let kw = d.measure_text(key, kfs);
        d.draw_text(key, x + (size - kw) / 2, y + size + 4, kfs, SECONDARY);
    }
}

#[allow(clippy::too_many_arguments)]
pub fn draw_small_btn(
    d: &mut RaylibDrawHandle,
    x: i32,
    y: i32,
    w: i32,
    h: i32,
    label: &str,
    key: &str,
    pressed: bool,
    is_mobile: bool,
) {
    let (bg, fg, border) = if pressed {
        (PRIMARY, BACKGROUND, PRIMARY)
    } else {
        (BACKGROUND, FOREGROUND, SECONDARY)
    };
    d.draw_rectangle(x, y, w, h, bg);
    d.draw_rectangle_lines(x, y, w, h, border);
    let fs = if is_mobile { 14 } else { 9 };
    let tw = d.measure_text(label, fs);
    d.draw_text(label, x + (w - tw) / 2, y + (h - fs) / 2, fs, fg);
    if !is_mobile {
        d.draw_text(key, x, y + h + 3, 7, SECONDARY);
    }
}
