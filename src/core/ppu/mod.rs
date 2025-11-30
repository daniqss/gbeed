mod display;
mod sprite;

use crate::prelude::*;
use display::Display;

pub const PPU_REGISTER_START: u16 = 0xFF40;
pub const PPU_REGISTER_END: u16 = 0xFF4B;

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
    pub fn new() -> Self {
        Self {
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
        }
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

impl Index<u16> for Ppu {
    type Output = u8;

    fn index(&self, address: u16) -> &Self::Output {
        match address {
            0xFF40 => &self.lcd_control,
            0xFF41 => &self.lcd_status,
            0xFF42 => &self.scroll_y,
            0xFF43 => &self.scroll_x,
            0xFF44 => &self.ly,
            0xFF45 => &self.lyc,
            0xFF46 => &self.dma,
            0xFF47 => &self.bg_palette,
            0xFF48 => &self.objp_0,
            0xFF49 => &self.objp_1,
            0xFF4A => &self.wy,
            0xFF4B => &self.wx,
            _ => panic!("PPU: Invalid read address {:#06X}", address),
        }
    }
}

impl IndexMut<u16> for Ppu {
    fn index_mut(&mut self, address: u16) -> &mut Self::Output {
        match address {
            0xFF40 => &mut self.lcd_control,
            0xFF41 => &mut self.lcd_status,
            0xFF42 => &mut self.scroll_y,
            0xFF43 => &mut self.scroll_x,
            0xFF44 => &mut self.ly,
            0xFF45 => &mut self.lyc,
            0xFF46 => &mut self.dma,
            0xFF47 => &mut self.bg_palette,
            0xFF48 => &mut self.objp_0,
            0xFF49 => &mut self.objp_1,
            0xFF4A => &mut self.wy,
            0xFF4B => &mut self.wx,
            _ => panic!("PPU: Invalid read address {:#06X}", address),
        }
    }
}
