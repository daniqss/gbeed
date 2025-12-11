mod fifo;
mod sprite;

use crate::{
    Dmg,
    core::memory::{OAM_END, OAM_START},
    prelude::*,
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
const MODE_2_OAM_INTERRUPT: u8 = 0x20;
const MODE_1_VBLANK_INTERRUPT: u8 = 0x10;
const MODE_0_HBLANK_INTERRUPT: u8 = 0x08;
const LYC_EQ_LY_FLAG: u8 = 0x04;

// screen dimensions
const DMG_SCREEN_WIDTH: usize = 160;
const DMG_SCREEN_HEIGHT: usize = 144;

const COLORS: [u32; 4] = [0xC4CFA1, 0x8B956D, 0x4D533C, 0x1F1F1F];

/// Represents the current mode of the LCD display
///   Mode                  | Action                                     | Duration                             | Accessible video memory
/// -----------------------:|--------------------------------------------|--------------------------------------|-------------------------
///   2 - OAM scan          | Searching for OBJs which overlap this line | 80 dots                              | VRAM, CGB palettes
///   3 - Drawing pixels    | Sending pixels to the LCD                  | Between 172 and 289 dots, see below  | None
///   0 - Horizontal Blank  | Waiting until the end of the scanline      | 376 - mode 3's duration              | VRAM, OAM, CGB palettes
///   1 - Vertical Blank    | Waiting until the next frame               | 4560 dots (10 scanlines)             | VRAM, OAM, CGB palettes
pub enum LCDMode {
    OAMScan = 4,
    Drawing = 3,
    HBlank = 0,
    VBlank = 1,
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
    ly: u8,
    lyc: u8,
    dma: u8,
    bg_palette: u8,
    obj0_palette: u8,
    obj1_palette: u8,
    wy: u8,
    wx: u8,

    framebuffer: [[u32; DMG_SCREEN_WIDTH]; DMG_SCREEN_HEIGHT],
    double_buffer: [[u32; DMG_SCREEN_WIDTH]; DMG_SCREEN_HEIGHT],
    bg_buffer: [[u32; 256]; 256],
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

            framebuffer: [[0; DMG_SCREEN_WIDTH]; DMG_SCREEN_HEIGHT],
            double_buffer: [[0; DMG_SCREEN_WIDTH]; DMG_SCREEN_HEIGHT],
            bg_buffer: [[0; 256]; 256],
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
    pub fn get_mode(&self) -> LCDMode {
        match self.lcd_status & 0x03 {
            0 => LCDMode::HBlank,
            1 => LCDMode::VBlank,
            2 => LCDMode::OAMScan,
            3 => LCDMode::Drawing,
            _ => unreachable!("LCD cannot have mode than two bits"),
        }
    }

    // pub fn update_palette(colors: &mut [u32; 4], palette: u8) {
    //     for i in 0..4 {
    //         colors[i] = COLORS[((palette >> (i * 2)) & 0x03) as usize];
    //     }
    // }

    ///            |  20 dots  | 43+ dots  | 51- dots
    /// -------------------------------------------------
    /// 144 lines  | Oam       | Pixel     |
    ///            | Search    | Transfer  | HBlank
    /// -------------------------------------------------
    /// 10 lines   |             VBlank              
    pub fn step(gb: &mut Dmg, instruction_cycles: u8) {
        if !gb.ppu.lcd_display_enable() {
            return;
        }
        gb.ppu.dots += instruction_cycles as usize;

        // this should be done on register writes
        if gb.ppu.dma != 0 {
            Ppu::dma_transfer(gb);
        }

        // look current mode
        match gb.ppu.get_mode() {
            LCDMode::OAMScan => {}
            LCDMode::Drawing => {}
            LCDMode::HBlank => {}
            LCDMode::VBlank => {
                if gb.ppu.dots >= 456 {
                    gb.ppu.dots -= 456;
                    gb.ppu.ly += 1;

                    if gb.ppu.ly > 153 {
                        gb.ppu.ly = 0;
                        gb.ppu.frames += 1;
                    }
                }
            }
        }
    }

    /// Writing to DMA register will copy from ROM or RAM to OAM memory
    /// It will take 160 dots or 320 at double speed
    /// CPU can access only HRAM and PPU can't access OAM
    /// Most games transfer to HRAM code to continue execution in CPU, and execute DMA transfer in VBlank
    fn dma_transfer(gb: &mut Dmg) {
        // address from data will be copied
        let src_addr: u16 = (gb.ppu.dma << 8) as u16;

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
