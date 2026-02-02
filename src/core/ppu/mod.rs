mod fifo;
mod sprite;

use crate::{
    Dmg,
    core::{
        memory::{OAM_END, OAM_START, VRAM_START},
        ppu::sprite::{MAX_SPRITES_IN_OAM, MAX_SPRITES_PER_LINE, Sprite},
    },
    prelude::*,
    utils::to_u16,
};

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
const OAM_INTERRUPT: u8 = 0x20;
const VBLANK_INTERRUPT: u8 = 0x10;
const HBLANK_INTERRUPT: u8 = 0x08;
const LYC_EQ_LY_FLAG: u8 = 0x04;

// screen dimensions
pub const DMG_SCREEN_WIDTH: usize = 160;
pub const DMG_SCREEN_HEIGHT: usize = 144;

const DOTS_PER_SCANLINE: usize = 456;
const SCANLINES_PER_FRAME: usize = 154;

const FINISH_OAM_SCAN_DOTS: usize = 80;
const FINISH_DRAWING_DOTS: usize = 172;
const FINISH_HBLANK_DOTS: usize = 204;
const FINISH_VBLANK_DOTS: usize = DOTS_PER_SCANLINE;

const DEFAULT_DMG_COLORS: [u32; 4] = [0xC4CFA1, 0x8B956D, 0x4D533C, 0x1F1F1F];

const DEFAULT_WINDOW_BASE_ADDR: u16 = 0x9800;
const COND_WINDOW_BASE_ADDR: u16 = 0x9C00;

/// Represents the current mode of the LCD display
///   Mode                  | Action                                     | Duration                             | Accessible video memory
/// -----------------------:|--------------------------------------------|--------------------------------------|-------------------------
///   2 - OAM scan          | Searching for OBJs which overlap this line | 80 dots                              | VRAM, CGB palettes
///   3 - Drawing pixels    | Sending pixels to the LCD                  | Between 172 and 289 dots, see below  | None
///   0 - Horizontal Blank  | Waiting until the end of the scanline      | 376 - mode 3's duration              | VRAM, OAM, CGB palettes
///   1 - Vertical Blank    | Waiting until the next frame               | 4560 dots (10 scanlines)             | VRAM, OAM, CGB palettes
pub enum LCDMode {
    OAMScan = 2,
    Drawing = 3,
    HBlank = 0,
    VBlank = 1,
}

impl LCDMode {
    pub fn reset_cycles_at_finish(&self) -> usize {
        match self {
            LCDMode::OAMScan => FINISH_OAM_SCAN_DOTS,
            LCDMode::Drawing => FINISH_DRAWING_DOTS,
            LCDMode::HBlank => FINISH_HBLANK_DOTS,
            LCDMode::VBlank => FINISH_VBLANK_DOTS,
        }
    }
}

// pub struct PixelFifo {
//     tile: [u8; 16],
//     head: usize,
//     tail: usize,
// }

// impl PixelFifo {
//     pub fn push
// }

#[derive(Debug)]
pub struct Ppu {
    /// A "dot" = one 222 Hz (aprox 4.194 MHz) time unit. A frame is not exactly one 60th of a second: the Game Boy runs slightly slower than 60 Hz, as one frame takes ~16.74 ms instead of ~16.67
    dots: usize,
    frames: usize,

    lcd_control: u8,
    lcd_status: u8,
    scroll_y: u8,
    scroll_x: u8,
    /// currently drawn line
    ly: u8,
    lyc: u8,
    dma: u8,
    bg_palette: u8,
    obj0_palette: u8,
    obj1_palette: u8,
    wy: u8,
    wx: u8,
    window_line_counter: u8,

    framebuffer: [[u32; DMG_SCREEN_WIDTH]; DMG_SCREEN_HEIGHT],

    pub colors: [u32; 4],
}

impl Default for Ppu {
    fn default() -> Self { Self::new() }
}

impl Ppu {
    pub fn new() -> Self {
        Self {
            dots: 0,
            frames: 0,

            lcd_control: 0x91,
            lcd_status: 0,
            scroll_y: 0,
            scroll_x: 0,
            ly: 0,
            lyc: 0,
            dma: 0,
            bg_palette: 0xFC,
            obj0_palette: 0xFF,
            obj1_palette: 0xFF,
            wy: 0,
            wx: 0,
            window_line_counter: 0,

            framebuffer: [[0; DMG_SCREEN_WIDTH]; DMG_SCREEN_HEIGHT],

            colors: DEFAULT_DMG_COLORS,
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
        OAM_INTERRUPT,
        VBLANK_INTERRUPT,
        HBLANK_INTERRUPT,
        LYC_EQ_LY_FLAG
    }

    #[inline]
    fn get_mode(&self) -> LCDMode {
        match self.lcd_status & 0x03 {
            0 => LCDMode::HBlank,
            1 => LCDMode::VBlank,
            2 => LCDMode::OAMScan,
            3 => LCDMode::Drawing,
            _ => unreachable!("LCD cannot have mode than two bits"),
        }
    }

    #[inline]
    fn set_mode(&mut self, mode: LCDMode) { self.lcd_status = (self.lcd_status & 0xFC) | mode as u8; }

    #[inline]
    /// If LY equals LYC, the LYC=LY flag in STAT register is set.
    /// And if the corresponding interrupt is enabled, the LCD STAT interrupt is requested.
    fn ly_equals_lyc_check(gb: &mut Dmg) {
        if gb.ppu.ly == gb.ppu.lyc {
            gb.ppu.set_lyc_eq_ly_flag(true);

            if gb.ppu.lyc_eq_ly_interrupt() {
                gb.interrupt_enable.set_lcd_stat_interrupt(true);
            }
        } else {
            gb.ppu.set_lyc_eq_ly_flag(false);
        }
    }

    pub fn get_color(&self, palette: u8, color_id: u8) -> u32 {
        let shade = (palette >> (color_id * 2)) & 0x03;
        self.colors[shade as usize]
    }

    // # Step the PPU by the number of cycles the last instruction took
    //
    //            |  20 dots  | 43+ dots  | 51- dots
    // -----------:-------------------------------------
    // 144 lines  | Oam       | Pixel     |
    //            | Search    | Transfer  | HBlank
    // -------------------------------------------------
    // 10 lines   |             VBlank
    pub fn step(gb: &mut Dmg, instruction_cycles: u8) {
        if !gb.ppu.lcd_display_enable() {
            return;
        }
        gb.ppu.dots += instruction_cycles as usize;
        // this should be done on register writes
        if gb.ppu.dma != 0 {
            Ppu::dma_transfer(gb);
        }

        let mode = gb.ppu.get_mode();

        // look current mode
        let next_mode = match mode {
            LCDMode::OAMScan if gb.ppu.dots >= FINISH_OAM_SCAN_DOTS => Some(LCDMode::Drawing),

            LCDMode::Drawing if gb.ppu.dots >= FINISH_DRAWING_DOTS => {
                // render scanline if we're not at the bottom of the screen yet
                if gb.ppu.ly < DMG_SCREEN_HEIGHT as u8 {
                    // render scanline
                    Ppu::draw_scanline(gb);
                }

                // set interrupt flag if hblank interrupt is needed
                if gb.ppu.hblank_interrupt() {
                    gb.interrupt_enable.set_lcd_stat_interrupt(true);
                }

                Some(LCDMode::HBlank)
            }

            LCDMode::HBlank if gb.ppu.dots >= FINISH_HBLANK_DOTS => {
                gb.ppu.ly += 1;

                // check for LYC=LY coincidence
                Ppu::ly_equals_lyc_check(gb);

                let next_mode = if gb.ppu.ly == DMG_SCREEN_HEIGHT as u8 {
                    // frame draw is finished
                    if gb.ppu.vblank_interrupt() {
                        gb.interrupt_enable.set_lcd_stat_interrupt(true);
                    }

                    LCDMode::VBlank
                }
                // frame not done yet
                else {
                    if gb.ppu.oam_interrupt() {
                        gb.interrupt_enable.set_lcd_stat_interrupt(true);
                    }
                    LCDMode::OAMScan
                };

                Some(next_mode)
            }

            LCDMode::VBlank => {
                // set VBlank interrupt at the start of VBlank period
                if gb.ppu.lcd_display_enable() && gb.ppu.ly >= DMG_SCREEN_HEIGHT as u8 {
                    gb.interrupt_enable.set_vblank_interrupt(true);
                }

                if gb.ppu.dots >= FINISH_VBLANK_DOTS {
                    gb.ppu.ly += 1;

                    // check for LYC=LY coincidence
                    Ppu::ly_equals_lyc_check(gb);

                    // if we finished all VBlank lines, go to next frame
                    if gb.ppu.ly > SCANLINES_PER_FRAME as u8 - 1 {
                        gb.ppu.ly = 0;
                        gb.ppu.frames += 1;

                        if gb.ppu.oam_interrupt() {
                            gb.interrupt_enable.set_lcd_stat_interrupt(true);
                        }
                        Some(LCDMode::OAMScan)
                    } else {
                        None
                    }
                } else {
                    None
                }
            }

            // continue in the same mode
            _ => None,
        };

        // reset dots and update mode in memory
        if let Some(next_mode) = next_mode {
            gb.ppu.dots -= mode.reset_cycles_at_finish();
            gb.ppu.set_mode(next_mode);
        }
    }

    pub fn draw_scanline(gb: &mut Dmg) {
        // draw background
        // Ppu::draw_background(gb);

        // draw window
        if gb.ppu.window_enable() {
            Ppu::draw_window(gb);
        }

        // draw sprites
        if gb.ppu.obj_enable() {
            Ppu::draw_sprites(gb)
        };
    }

    fn draw_sprites(gb: &mut Dmg) {
        let current_line = gb.ppu.ly;
        let sprite_height = if gb.ppu.obj_size() { 16 } else { 8 };
        let mut drawn_sprites = 0u8;
        let mut sprites_count = 0u8;

        while drawn_sprites < MAX_SPRITES_PER_LINE && sprites_count < MAX_SPRITES_IN_OAM {
            let oam_addr = OAM_START + (sprites_count as u16) * 4;
            let sprite = Sprite::from_oam(gb, oam_addr);

            // is sprite in current line?
            if current_line < sprite.ypos || current_line >= sprite.ypos + sprite_height {
                sprites_count += 1;
                continue;
            }

            let mut line_in_sprite = if sprite.yflip() {
                sprite_height - 1 - (current_line - sprite.ypos)
            } else {
                current_line - sprite.ypos
            };

            // adjust if sprite is 8x16
            let mut tile_index = sprite.tile_index;
            if sprite_height == 16 {
                tile_index &= 0xFE;
                if line_in_sprite >= 8 {
                    tile_index += 1;
                    line_in_sprite -= 8;
                }
            }

            let tile_addr = VRAM_START + (tile_index as u16) * 16 + (line_in_sprite as u16) * 2;
            let low_tile_byte = gb[tile_addr];
            let high_tile_byte = gb[tile_addr + 1];

            let palette = if sprite.palette_number() {
                gb.ppu.obj1_palette
            } else {
                gb.ppu.obj0_palette
            };

            // maybe we can use this tile draw for bg and windows
            for pixel in 0..8 {
                let bit_index = if sprite.xflip() { pixel } else { 7 - pixel };

                let low_bit = (low_tile_byte >> bit_index) & 0x01;
                let high_bit = (high_tile_byte >> bit_index) & 0x01;
                let color_id = (high_bit << 1) | low_bit;

                // transparent pixel
                if color_id == 0 {
                    continue;
                }

                // out of screen
                let screen_x = sprite.xpos.wrapping_add(pixel);
                if screen_x >= DMG_SCREEN_WIDTH as u8 {
                    continue;
                }

                // sprite under de background
                if sprite.priority() {
                    let bg_pixel = gb.ppu.framebuffer[current_line as usize][screen_x as usize];
                    if bg_pixel != gb.ppu.colors[0] {
                        continue;
                    }
                }

                gb.ppu.framebuffer[current_line as usize][screen_x as usize] =
                    gb.ppu.get_color(palette, color_id);
            }

            drawn_sprites += 1;
            sprites_count += 1;
        }
    }

    fn draw_window(gb: &mut Dmg) {
        // window enable check is done in draw_scanline
        let current_line = gb.ppu.ly;
        let window_y = gb.ppu.wy;
        let window_x = gb.ppu.wx as isize - 7;

        if current_line < window_y {
            return;
        }

        if current_line == window_y {
            gb.ppu.window_line_counter = 0;
        }

        let tile_map_base = if gb.ppu.window_tile_map_address() {
            COND_WINDOW_BASE_ADDR
        } else {
            DEFAULT_WINDOW_BASE_ADDR
        };

        let mut window_rendered = false;

        for pixel in 0..DMG_SCREEN_WIDTH {
            if (pixel as isize) < window_x {
                continue;
            }
            window_rendered = true;

            let window_column = (pixel as isize) - window_x;
            let tile_x = window_column / 8;
            let tile_y = gb.ppu.window_line_counter as isize / 8;

            let tile_index = tile_y * 32 + tile_x;

            let tile_address_in_map = tile_map_base + tile_index as u16;
            let tile_number = gb[tile_address_in_map];

            // The "$8000 method" uses $8000 as its base pointer and uses an unsigned addressing,
            // meaning that tiles 0-127 are in block 0, and tiles 128-255 are in block 1.
            //
            // The "$8800 method" uses $9000 as its base pointer and uses a signed addressing,
            // meaning that tiles 0-127 are in block 2, and tiles -128 to -1 are in block 1; or, to put it differently,
            // "$8800 addressing" takes tiles 0-127 from block 2 and tiles 128-255 from block 1.
            let tile_data_base = if gb.ppu.bg_and_window_tile_data() || tile_number >= 128 {
                0x8000
            } else {
                0x9000
            };
            let tile_address = tile_data_base + (tile_number as u16) * 16;

            let line_in_tile = gb.ppu.window_line_counter % 8;

            let first_byte = gb[tile_address + (line_in_tile as u16) * 2];
            let second_byte = gb[tile_address + (line_in_tile as u16) * 2 + 1];

            let bit_index = 7 - (window_column % 8);

            let low_pixel = (first_byte >> bit_index) & 1;
            let high_pixel = (second_byte >> bit_index) & 1;

            let color_id = (high_pixel << 1) | low_pixel;
            let color = gb.ppu.get_color(gb.ppu.bg_palette, color_id);

            gb.ppu.framebuffer[current_line as usize][pixel] = color;
        }

        if window_rendered {
            gb.ppu.window_line_counter += 1;
        }
    }

    fn draw_bg(gb: &mut Dmg) {
        let current_line = gb.ppu.ly;
        let scroll_x = gb.ppu.scroll_x;
        let scroll_y = gb.ppu.scroll_y;

        let tile_map_base = if gb.ppu.bg_tile_map_address() {
            COND_WINDOW_BASE_ADDR
        } else {
            DEFAULT_WINDOW_BASE_ADDR
        };

        for pixel in 0..DMG_SCREEN_WIDTH {
            let bg_x = (pixel as u8).wrapping_add(scroll_x);
            let bg_y = (current_line).wrapping_add(scroll_y);
            let tile_x = bg_x >> 3;
            let tile_y = bg_y >> 3;

            let tile_index = tile_y * 32 + tile_x;
            let tile_address_in_map = tile_map_base + tile_index as u16;
            let tile_number = gb[tile_address_in_map];

            // The "$8000 method" uses $8000 as its base pointer and uses an unsigned addressing,
            // meaning that tiles 0-127 are in block 0, and tiles 128-255 are in block 1.
            //
            // The "$8800 method" uses $9000 as its base pointer and uses a signed addressing,
            // meaning that tiles 0-127 are in block 2, and tiles -128 to -1 are in block 1; or, to put it differently,
            // "$8800 addressing" takes tiles 0-127 from block 2 and tiles 128-255 from block 1.
            let tile_data_base = if gb.ppu.bg_and_window_tile_data() || tile_number >= 128 {
                0x8000
            } else {
                0x9000
            };
            let tile_address = tile_data_base + (tile_number as u16) * 16;

            let line_in_tile = gb.ppu.window_line_counter % 8;

            let first_byte = gb[tile_address + (line_in_tile as u16) * 2];
            let second_byte = gb[tile_address + (line_in_tile as u16) * 2 + 1];

            let bit_index = 7 - (bg_x % 8);

            let low_pixel = (first_byte >> bit_index) & 0xb1;
            let high_pixel = (second_byte >> bit_index) & 0xb1;

            let color_id = (high_pixel << 1) | low_pixel;
            let color = gb.ppu.get_color(gb.ppu.bg_palette, color_id);

            gb.ppu.framebuffer[current_line as usize][pixel] = color;
        }
    }

    /// Writing to DMA register will copy from ROM or RAM to OAM memory
    /// It will take 160 dots or 320 at double speed
    /// CPU can access only HRAM and PPU can't access OAM
    /// Most games transfer to HRAM code to continue execution in CPU, and execute DMA transfer in VBlank
    fn dma_transfer(gb: &mut Dmg) {
        // address from data will be copied
        let src_addr: u16 = to_u16(0, gb.ppu.dma);

        for i in 0..(OAM_END - OAM_START + 1) {
            let byte = gb[(src_addr + i) as u16];
            gb[(OAM_START + i) as u16] = byte;
        }

        gb.ppu.dma = 0;
    }
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
            0xFF48 => &self.obj0_palette,
            0xFF49 => &self.obj1_palette,
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
            0xFF48 => &mut self.obj0_palette,
            0xFF49 => &mut self.obj1_palette,
            0xFF4A => &mut self.wy,
            0xFF4B => &mut self.wx,
            _ => panic!("PPU: Invalid read address {:#04X}", address),
        }
    }
}
