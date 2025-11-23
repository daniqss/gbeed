mod display;
mod sprite;

use crate::prelude::*;
use display::Display;
use std::{cell::RefCell, rc::Rc};

/// LCD Control Register (R/W) bits
const LCD_DISPLAY_ENABLE: u8 = 0x80;
const WINDOW_TILE_MAP_ADDRESS: u8 = 0x40;
const WINDOW_ENABLE: u8 = 0x20;
const BG_AND_WINDOW_TILE_DATA: u8 = 0x10;
const BG_TILE_MAP_ADDRESS: u8 = 0x08;
const OBJ_SIZE: u8 = 0x04;
const OBJ_ENABLE: u8 = 0x02;
const BG_ENABLE: u8 = 0x01;

/// LCDC Status Register (R/W) bits
const LYC_EQ_LY_INTERRUPT: u8 = 0x40;
const MODE_2_OAM_INTERRUPT: u8 = 0x20;
const MODE_1_VBLANK_INTERRUPT: u8 = 0x10;
const MODE_0_HBLANK_INTERRUPT: u8 = 0x08;
const LYC_EQ_LY_FLAG: u8 = 0x04;

#[derive(Debug, Default)]
pub struct Ppu {
    lcd_control: u8,
    lcd_status: u8,
    scroll_y: u8,
    scroll_x: u8,
    ly: u8,
    lyc: u8,
    dma: u8,
    bg_palette: u8,
    objp_0: u8,
    objp_1: u8,
    wy: u8,
    wx: u8,

    display: Display,
}

impl Ppu {
    pub fn new() -> Rc<RefCell<Self>> {
        Rc::new(RefCell::new(Self {
            lcd_control: 0x91,
            lcd_status: 0,
            scroll_y: 0,
            scroll_x: 0,
            ly: 0,
            lyc: 0,
            dma: 0,
            bg_palette: 0xFC,
            objp_0: 0xFF,
            objp_1: 0xFF,
            wy: 0,
            wx: 0,

            display: Display::default(),
        }))
    }

    // lcd_control bit access functions
    bit_accessors! {
        target: lcd_control;

        LCD_DISPLAY_ENABLE,
        WINDOW_TILE_MAP_ADDRESS,
        WINDOW_ENABLE,
        BG_AND_WINDOW_TILE_DATA,
        BG_TILE_MAP_ADDRESS,
        OBJ_SIZE,
        OBJ_ENABLE,
        BG_ENABLE
    }

    // lcd_status bit access functions
    bit_accessors! {
        target: lcd_status;

        LYC_EQ_LY_INTERRUPT,
        MODE_2_OAM_INTERRUPT,
        MODE_1_VBLANK_INTERRUPT,
        MODE_0_HBLANK_INTERRUPT,
        LYC_EQ_LY_FLAG
    }

    // maybe its not best way to implement this
    pub fn get_mode(&self) -> bool { if self.lcd_status & 0x03 == 0 { false } else { true } }
}
