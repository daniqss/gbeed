mod display;
mod sprite;

use display::Display;
pub use sprite::Sprite;

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

/// Macro used to generate easy bit getter functions
macro_rules! bit_getters {
    ($reg_name:ident, $bit:ident, $fn_name:ident) => {
        #[inline]
        pub fn $fn_name(&self) -> bool { (self.$reg_name & $bit) != 0 }
    };
}

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

            display: Display::new(),
        }
    }

    // lcd_control bit getters
    bit_getters!(lcd_control, LCD_DISPLAY_ENABLE, lcd_display_enable);
    bit_getters!(lcd_control, WINDOW_TILE_MAP_ADDRESS, window_tile_map_address);
    bit_getters!(lcd_control, WINDOW_ENABLE, window_enable);
    bit_getters!(lcd_control, BG_AND_WINDOW_TILE_DATA, bg_and_window_tile_data);
    bit_getters!(lcd_control, BG_TILE_MAP_ADDRESS, bg_tile_map_address);
    bit_getters!(lcd_control, OBJ_SIZE, obj_size);
    bit_getters!(lcd_control, OBJ_ENABLE, obj_enable);
    bit_getters!(lcd_control, BG_ENABLE, bg_enable);

    // ldc_status bit getters
    bit_getters!(lcd_status, LYC_EQ_LY_INTERRUPT, lyc_eq_ly_interrupt);
    bit_getters!(lcd_status, MODE_2_OAM_INTERRUPT, mode_2_oam_interrupt);
    bit_getters!(lcd_status, MODE_1_VBLANK_INTERRUPT, mode_1_vblank_interrupt);
    bit_getters!(lcd_status, MODE_0_HBLANK_INTERRUPT, mode_0_hblank_interrupt);
    bit_getters!(lcd_status, LYC_EQ_LY_FLAG, lyc_eq_ly_flag);

    // maybe its not best way to implement this
    pub fn get_mode(&self) -> bool { if self.lcd_status & 0x03 == 0 { false } else { true } }
}
