mod renderer;
mod sprite;

use crate::{
    Interrupt, OAM_SIZE, VRAM_END, VRAM_SIZE,
    dmg::Dmg,
    mem_range,
    memory::{OAM_END, OAM_START, VRAM_START},
    ppu::sprite::{MAX_SPRITES_IN_OAM, MAX_SPRITES_PER_LINE, Sprite},
    prelude::*,
};

pub use renderer::{DefaultRenderer, Renderer};

mem_range!(PPU_REGISTER, 0xFF40, 0xFF4B);

/// LCD Control Reg (R/W) bits
const LCD_DISPLAY_ENABLE: u8 = 0x80;
const WINDOW_TILE_MAP_ADDRESS: u8 = 0x40;
const WINDOW_ENABLE: u8 = 0x20;
const BG_AND_WINDOW_TILE_DATA: u8 = 0x10;
const BG_TILE_MAP_ADDRESS: u8 = 0x08;
const OBJ_SIZE: u8 = 0x04;
const OBJ_ENABLE: u8 = 0x02;
const BG_ENABLE: u8 = 0x01;

/// LCDC Status Reg (R/W) bits
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

const DEFAULT_WINDOW_BASE_ADDR: u16 = 0x9800;
const COND_WINDOW_BASE_ADDR: u16 = 0x9C00;

pub const DMA_REGISTER: u16 = 0xFF46;

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

pub struct Ppu {
    /// A "dot" = one 222 Hz (aprox 4.194 MHz) time unit. A frame is not exactly one 60th of a second: the Game Boy runs slightly slower than 60 Hz, as one frame takes ~16.74 ms instead of ~16.67
    dots: usize,
    frames: usize,

    pub vram: [u8; VRAM_SIZE as usize],
    pub oam_ram: [u8; OAM_SIZE as usize],

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

    pub last_cycles: usize,
}

impl Default for Ppu {
    fn default() -> Self { Self::new() }
}

impl std::fmt::Debug for Ppu {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Ppu {{ dots: {}, frames: {}, lcd_control: {:#04X}, lcd_status: {:#04X}, scroll_y: {}, scroll_x: {}, ly: {}, lyc: {}, dma: {}, bg_palette: {:#04X}, obj0_palette: {:#04X}, obj1_palette: {:#04X}, wy: {}, wx: {} }}",
            self.dots,
            self.frames,
            self.lcd_control,
            self.lcd_status,
            self.scroll_y,
            self.scroll_x,
            self.ly,
            self.lyc,
            self.dma,
            self.bg_palette,
            self.obj0_palette,
            self.obj1_palette,
            self.wy,
            self.wx
        )
    }
}

impl Ppu {
    pub fn new() -> Self {
        Self {
            dots: 0,
            frames: 0,

            vram: [0; VRAM_SIZE as usize],
            oam_ram: [0; OAM_SIZE as usize],

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

            last_cycles: 0,
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
    fn ly_equals_lyc_check(&mut self, interrupt_flag: &mut Interrupt) {
        if self.ly == self.lyc {
            self.set_lyc_eq_ly_flag(true);

            if self.lyc_eq_ly_interrupt() {
                interrupt_flag.set_lcd_stat_interrupt(true);
            }
        } else {
            self.set_lyc_eq_ly_flag(false);
        }
    }

    // # Step the PPU by the number of cycles the last instruction took
    //
    //            |  20 dots  | 43+ dots  | 51- dots
    // -----------:-------------------------------------
    // 144 lines  | Oam       | Pixel     |
    //            | Search    | Transfer  | HBlank
    // -------------------------------------------------
    // 10 lines   |             VBlank
    pub fn step<R: Renderer>(&mut self, renderer: &mut R, cycles: usize, interrupt_flag: &mut Interrupt) {
        if !self.lcd_display_enable() {
            return;
        }

        let instruction_cycles = if cycles >= self.last_cycles {
            cycles - self.last_cycles
        } else {
            cycles
        };
        self.last_cycles = cycles;

        self.dots += instruction_cycles;

        let mode = self.get_mode();
        // look current mode
        let next_mode = match mode {
            LCDMode::OAMScan if self.dots >= FINISH_OAM_SCAN_DOTS => Some(LCDMode::Drawing),

            LCDMode::Drawing if self.dots >= FINISH_DRAWING_DOTS => {
                // render scanline if we're not at the bottom of the screen yet
                if self.ly < DMG_SCREEN_HEIGHT as u8 {
                    // render scanline
                    self.draw_scanline(renderer);
                }

                // set interrupt flag if hblank interrupt is needed
                if self.hblank_interrupt() {
                    interrupt_flag.set_lcd_stat_interrupt(true);
                }

                Some(LCDMode::HBlank)
            }

            LCDMode::HBlank if self.dots >= FINISH_HBLANK_DOTS => {
                self.ly += 1;

                // check for LYC=LY coincidence
                self.ly_equals_lyc_check(interrupt_flag);

                let next_mode = if self.ly == DMG_SCREEN_HEIGHT as u8 {
                    // frame draw is finished
                    if self.vblank_interrupt() {
                        interrupt_flag.set_lcd_stat_interrupt(true);
                    }

                    renderer.draw_screen();

                    LCDMode::VBlank
                }
                // frame not done yet
                else {
                    if self.oam_interrupt() {
                        interrupt_flag.set_lcd_stat_interrupt(true);
                    }
                    LCDMode::OAMScan
                };

                Some(next_mode)
            }

            LCDMode::VBlank => {
                // set VBlank interrupt at the start of VBlank period
                if self.lcd_display_enable() && self.ly >= DMG_SCREEN_HEIGHT as u8 {
                    interrupt_flag.set_vblank_interrupt(true);
                }

                if self.dots >= FINISH_VBLANK_DOTS {
                    self.ly += 1;

                    // check for LYC=LY coincidence
                    self.ly_equals_lyc_check(interrupt_flag);

                    // if we finished all VBlank lines, go to next frame
                    if self.ly > SCANLINES_PER_FRAME as u8 - 1 {
                        self.ly = 0;
                        self.frames += 1;

                        if self.oam_interrupt() {
                            interrupt_flag.set_lcd_stat_interrupt(true);
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
            self.dots -= mode.reset_cycles_at_finish();
            self.set_mode(next_mode);
        }
    }

    pub fn draw_scanline<R: Renderer>(&mut self, renderer: &mut R) {
        // draw background
        self.draw_bg(renderer);

        // draw window
        if self.window_enable() {
            self.draw_window(renderer);
        }

        // draw sprites
        if self.obj_enable() {
            self.draw_sprites(renderer)
        };
    }

    fn draw_sprites<R: Renderer>(&mut self, renderer: &mut R) {
        let current_line = self.ly;
        let sprite_height = if self.obj_size() { 16 } else { 8 };
        let mut drawn_sprites = 0u8;
        let mut sprites_count = 0u8;

        // track which sprite owns each pixel in the current line
        let mut pixel_owner: [Option<u8>; DMG_SCREEN_WIDTH] = [None; DMG_SCREEN_WIDTH];

        while drawn_sprites < MAX_SPRITES_PER_LINE && sprites_count < MAX_SPRITES_IN_OAM {
            let oam_addr = OAM_START + (sprites_count as u16) * 4;
            let sprite = Sprite::from_oam(self, oam_addr);

            // is sprite in current line?
            let line_offset = current_line.wrapping_sub(sprite.ypos);
            if line_offset >= sprite_height {
                sprites_count += 1;
                continue;
            }

            let mut line_in_sprite = if sprite.yflip() {
                sprite_height - 1 - line_offset
            } else {
                line_offset
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
            let low_tile_byte = self.read(tile_addr);
            let high_tile_byte = self.read(tile_addr + 1);

            let palette = if sprite.palette_number() {
                self.obj1_palette
            } else {
                self.obj0_palette
            };

            for pixel in 0..8 {
                let bit_index = if sprite.xflip() { pixel } else { 7 - pixel };

                let low_bit = (low_tile_byte >> bit_index) & 0x01;
                let high_bit = (high_tile_byte >> bit_index) & 0x01;
                let color_id = (high_bit << 1) | low_bit;

                // transparent pixel
                if color_id == 0 {
                    continue;
                }

                let screen_x = sprite.xpos.wrapping_add(pixel);
                if screen_x >= DMG_SCREEN_WIDTH as u8 {
                    continue;
                }

                let sx = screen_x as usize;

                // sprite under background
                if sprite.priority() {
                    let bg_pixel = renderer.read_pixel(sx, current_line as usize);
                    if bg_pixel != renderer.get_color(self.bg_palette, 0) {
                        continue;
                    }
                }

                let should_draw = match pixel_owner[sx] {
                    Some(owner_x) => sprite.xpos < owner_x,
                    None => true,
                };

                if should_draw {
                    pixel_owner[sx] = Some(sprite.xpos);

                    renderer.write_pixel(sx, current_line as usize, renderer.get_color(palette, color_id));
                }
            }

            drawn_sprites += 1;
            sprites_count += 1;
        }
    }

    fn draw_window<R: Renderer>(&mut self, renderer: &mut R) {
        // window enable check is done in draw_scanline
        let current_line = self.ly;
        let window_y = self.wy;
        let window_x = self.wx as isize - 7;

        if current_line < window_y {
            return;
        }

        if current_line == window_y {
            self.window_line_counter = 0;
        }

        let tile_map_base = if self.window_tile_map_address() {
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
            let tile_y = self.window_line_counter as isize / 8;

            let tile_index = tile_y * 32 + tile_x;

            let tile_address_in_map = tile_map_base + tile_index as u16;
            let tile_number = self.read(tile_address_in_map);

            // The "$8000 method" uses $8000 as its base pointer and uses an unsigned addressing,
            // meaning that tiles 0-127 are in block 0, and tiles 128-255 are in block 1.
            //
            // The "$8800 method" uses $9000 as its base pointer and uses a signed addressing,
            // meaning that tiles 0-127 are in block 2, and tiles -128 to -1 are in block 1; or, to put it differently,
            // "$8800 addressing" takes tiles 0-127 from block 2 and tiles 128-255 from block 1.
            let tile_data_base = if self.bg_and_window_tile_data() || tile_number >= 128 {
                0x8000
            } else {
                0x9000
            };
            let tile_address = tile_data_base + (tile_number as u16) * 16;

            let line_in_tile = self.window_line_counter % 8;

            let first_byte = self.read(tile_address + (line_in_tile as u16) * 2);
            let second_byte = self.read(tile_address + (line_in_tile as u16) * 2 + 1);

            let bit_index = 7 - (window_column % 8);

            let low_pixel = (first_byte >> bit_index) & 1;
            let high_pixel = (second_byte >> bit_index) & 1;

            let color_id = (high_pixel << 1) | low_pixel;
            let color = renderer.get_color(self.bg_palette, color_id);

            renderer.write_pixel(pixel, current_line as usize, color);
        }

        if window_rendered {
            self.window_line_counter = self.window_line_counter.wrapping_add(1);
        }
    }

    fn draw_bg<R: Renderer>(&mut self, renderer: &mut R) {
        let current_line = self.ly;
        let scroll_x = self.scroll_x;
        let scroll_y = self.scroll_y;

        let tile_map_base = if self.bg_tile_map_address() {
            COND_WINDOW_BASE_ADDR
        } else {
            DEFAULT_WINDOW_BASE_ADDR
        };

        for pixel in 0..DMG_SCREEN_WIDTH {
            let bg_x = (pixel as u8).wrapping_add(scroll_x);
            let bg_y = (current_line).wrapping_add(scroll_y);
            let tile_x = bg_x >> 3;
            let tile_y = bg_y >> 3;

            let tile_index: usize = tile_y as usize * 32 + tile_x as usize;
            let tile_address_in_map = tile_map_base + tile_index as u16;
            // let tile_number = gb.read(tile_address_in_map);
            let tile_number = self.read(tile_address_in_map);

            // The "$8000 method" uses $8000 as its base pointer and uses an unsigned addressing,
            // meaning that tiles 0-127 are in block 0, and tiles 128-255 are in block 1.
            //
            // The "$8800 method" uses $9000 as its base pointer and uses a signed addressing,
            // meaning that tiles 0-127 are in block 2, and tiles -128 to -1 are in block 1; or, to put it differently,
            // "$8800 addressing" takes tiles 0-127 from block 2 and tiles 128-255 from block 1.
            let tile_data_base = if self.bg_and_window_tile_data() || tile_number >= 128 {
                0x8000
            } else {
                0x9000
            };
            let tile_address = tile_data_base + (tile_number as u16) * 16;

            let line_in_tile = bg_y % 8;

            let first_byte = self.read(tile_address + (line_in_tile as u16) * 2);
            let second_byte = self.read(tile_address + (line_in_tile as u16) * 2 + 1);

            let bit_index = 7 - (bg_x % 8);

            let low_pixel = (first_byte >> bit_index) & 1;
            let high_pixel = (second_byte >> bit_index) & 1;

            let color_id = (high_pixel << 1) | low_pixel;
            let color = renderer.get_color(self.bg_palette, color_id);

            renderer.write_pixel(pixel, current_line as usize, color);
        }
    }

    /// Writing to DMA register will copy from ROM or RAM to OAM memory
    /// It will take 160 dots or 320 at double speed
    /// CPU can access only HRAM and PPU can't access OAM
    /// Most games transfer to HRAM code to continue execution in CPU, and execute DMA transfer in VBlank
    pub fn dma_transfer(gb: &mut Dmg, src_addr: u8) {
        let src_addr = (src_addr as u16) << 8;
        for i in 0..(OAM_END - OAM_START + 1) {
            let byte = gb.read(src_addr + i);
            gb.write(OAM_START + i, byte);
        }

        gb.ppu.dma = 0;
    }

    pub fn tile_block0(&self) -> &[u8] { &self.vram[0..0x800] }
    pub fn tile_block1(&self) -> &[u8] { &self.vram[0x800..0x1000] }
    pub fn tile_block2(&self) -> &[u8] { &self.vram[0x1000..0x1800] }
    pub fn tile_data(&self) -> &[u8] { &self.vram[0..0x1800] }
    pub fn bg_map0(&self) -> &[u8] { &self.vram[0x1800..0x1C00] }
    pub fn bg_map1(&self) -> &[u8] { &self.vram[0x1C00..0x2000] }
}

impl Accessible<u16> for Ppu {
    fn read(&self, address: u16) -> u8 {
        match address {
            VRAM_START..=VRAM_END => self.vram[(address - VRAM_START) as usize],
            OAM_START..=OAM_END => self.oam_ram[(address - OAM_START) as usize],

            0xFF40 => self.lcd_control,
            0xFF41 => self.lcd_status,
            0xFF42 => self.scroll_y,
            0xFF43 => self.scroll_x,
            0xFF44 => self.ly,
            0xFF45 => self.lyc,
            0xFF46 => self.dma,
            0xFF47 => self.bg_palette,
            0xFF48 => self.obj0_palette,
            0xFF49 => self.obj1_palette,
            0xFF4A => self.wy,
            0xFF4B => self.wx,
            _ => unreachable!(
                "Ppu: read of address {address:04X} should have been handled by other components",
            ),
        }
    }

    fn write(&mut self, address: u16, value: u8) {
        match address {
            VRAM_START..=VRAM_END => self.vram[(address - VRAM_START) as usize] = value,
            OAM_START..=OAM_END => self.oam_ram[(address - OAM_START) as usize] = value,

            0xFF40 => {
                self.lcd_control = value;

                // if LCD is turned off, reset some PPU state
                if !self.lcd_display_enable() {
                    // clear ppu mode
                    self.lcd_status &= 0x7C;
                    self.ly = 0;
                }
            }
            0xFF41 => self.lcd_status = value,
            0xFF42 => self.scroll_y = value,
            0xFF43 => self.scroll_x = value,
            0xFF44 => self.ly = value,
            0xFF45 => self.lyc = value,
            DMA_REGISTER => unreachable!(
                "Writing to DMA register should have been handled by Dmg, address: {address:04X}"
            ),
            0xFF47 => self.bg_palette = value,
            0xFF48 => self.obj0_palette = value,
            0xFF49 => self.obj1_palette = value,
            0xFF4A => self.wy = value,
            0xFF4B => self.wx = value,
            _ => unreachable!(
                "Ppu: write of address {address:04X} should have been handled by other components",
            ),
        }
    }
}
