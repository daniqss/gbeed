use raylib::prelude::*;

use crate::impl_cyclic_enum;

pub type PaletteColor = [Color; 4];

pub const DMG_CLASSIC_PALETTE: PaletteColor = [
    Color::new(196, 207, 161, 255),
    Color::new(139, 149, 109, 255),
    Color::new(77, 83, 60, 255),
    Color::new(31, 31, 31, 255),
];

pub const GRAYSCALE_PALETTE: PaletteColor = [
    Color::new(255, 255, 255, 255),
    Color::new(170, 170, 170, 255),
    Color::new(85, 85, 85, 255),
    Color::new(0, 0, 0, 255),
];

pub const RED_PALETTE: PaletteColor = [
    Color::new(204, 0, 1, 255),
    Color::new(143, 0, 1, 255),
    Color::new(82, 0, 0, 255),
    Color::new(20, 0, 0, 255),
];

pub const TURQUOISE_PALETTE: PaletteColor = [
    Color::new(3, 192, 198, 255),
    Color::new(2, 134, 139, 255),
    Color::new(1, 77, 79, 255),
    Color::new(0, 19, 20, 255),
];

pub const BLUE_PALETTE: PaletteColor = [
    Color::new(0, 0, 254, 255),
    Color::new(0, 0, 178, 255),
    Color::new(0, 0, 102, 255),
    Color::new(0, 0, 25, 255),
];

pub const GREEN_PALETTE: PaletteColor = [
    Color::new(1, 204, 0, 255),
    Color::new(1, 143, 0, 255),
    Color::new(0, 82, 0, 255),
    Color::new(0, 20, 0, 255),
];

pub const YELLOW_PALETTE: PaletteColor = [
    Color::new(255, 255, 1, 255),
    Color::new(178, 178, 1, 255),
    Color::new(102, 102, 0, 255),
    Color::new(25, 25, 0, 255),
];

pub const PURPLE_PALETTE: PaletteColor = [
    Color::new(118, 44, 167, 255),
    Color::new(83, 31, 117, 255),
    Color::new(47, 18, 67, 255),
    Color::new(12, 4, 17, 255),
];

pub const PINK_PALETTE: PaletteColor = [
    Color::new(254, 152, 191, 255),
    Color::new(178, 106, 134, 255),
    Color::new(102, 61, 76, 255),
    Color::new(25, 15, 19, 255),
];

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Palette {
    #[default]
    DmgClassic,
    GrayScale,
    Red,
    Turquoise,
    Blue,
    Green,
    Yellow,
    Purple,
    Pink,
}

use Palette::*;

impl_cyclic_enum!(
    Palette,
    [
        DmgClassic, GrayScale, Red, Turquoise, Blue, Green, Yellow, Purple, Pink
    ]
);

impl Palette {
    pub fn get_palette_color(&self) -> PaletteColor {
        match self {
            DmgClassic => DMG_CLASSIC_PALETTE,
            GrayScale => GRAYSCALE_PALETTE,
            Red => RED_PALETTE,
            Turquoise => TURQUOISE_PALETTE,
            Blue => BLUE_PALETTE,
            Green => GREEN_PALETTE,
            Yellow => YELLOW_PALETTE,
            Purple => PURPLE_PALETTE,
            Pink => PINK_PALETTE,
        }
    }

    pub fn name(&self) -> &str {
        match self {
            DmgClassic => "DmgClassic",
            GrayScale => "GrayScale",
            Red => "Red",
            Turquoise => "Turquoise",
            Blue => "Blue",
            Green => "Green",
            Yellow => "Yellow",
            Purple => "Purple",
            Pink => "Pink",
        }
    }
}

#[inline(always)]
pub fn foreground(palette_color: &PaletteColor) -> Color { palette_color[0] }
#[inline(always)]
pub fn primary(palette_color: &PaletteColor) -> Color { palette_color[1] }
#[inline(always)]
pub fn secondary(palette_color: &PaletteColor) -> Color { palette_color[2] }
#[inline(always)]
pub fn background(palette_color: &PaletteColor) -> Color { palette_color[3] }
