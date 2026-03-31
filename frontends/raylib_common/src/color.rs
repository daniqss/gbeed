use raylib::prelude::*;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Palette {
    #[default]
    DmgClassic,
    GrayScale,
    Red,
    Blue,
    Green,
    Yellow,
    Purple,
    Pink,
}

impl Palette {
    pub const ALL: [Palette; 8] = [
        Palette::DmgClassic,
        Palette::GrayScale,
        Palette::Red,
        Palette::Blue,
        Palette::Green,
        Palette::Yellow,
        Palette::Purple,
        Palette::Pink,
    ];

    pub fn colors(&self) -> [Color; 4] {
        match self {
            Palette::DmgClassic => [
                Color::new(196, 207, 161, 255),
                Color::new(139, 149, 109, 255),
                Color::new(77, 83, 60, 255),
                Color::new(31, 31, 31, 255),
            ],
            Palette::GrayScale => [
                Color::new(255, 255, 255, 255),
                Color::new(170, 170, 170, 255),
                Color::new(85, 85, 85, 255),
                Color::new(0, 0, 0, 255),
            ],
            Palette::Red => [
                Color::new(255, 204, 204, 255),
                Color::new(255, 136, 136, 255),
                Color::new(204, 68, 68, 255),
                Color::new(136, 0, 0, 255),
            ],
            Palette::Blue => [
                Color::new(204, 204, 255, 255),
                Color::new(136, 136, 255, 255),
                Color::new(68, 68, 204, 255),
                Color::new(0, 0, 136, 255),
            ],
            Palette::Green => [
                Color::new(204, 255, 204, 255),
                Color::new(136, 255, 136, 255),
                Color::new(68, 204, 68, 255),
                Color::new(0, 136, 0, 255),
            ],
            Palette::Yellow => [
                Color::new(255, 255, 204, 255),
                Color::new(255, 255, 136, 255),
                Color::new(204, 204, 68, 255),
                Color::new(136, 136, 0, 255),
            ],
            Palette::Purple => [
                Color::new(255, 204, 255, 255),
                Color::new(255, 136, 255, 255),
                Color::new(204, 68, 204, 255),
                Color::new(136, 0, 136, 255),
            ],
            Palette::Pink => [
                Color::new(255, 204, 229, 255),
                Color::new(255, 153, 204, 255),
                Color::new(204, 102, 153, 255),
                Color::new(153, 51, 102, 255),
            ],
        }
    }

    pub fn name(&self) -> &str {
        match self {
            Palette::DmgClassic => "DmgClassic",
            Palette::GrayScale => "GrayScale",
            Palette::Red => "Red",
            Palette::Blue => "Blue",
            Palette::Green => "Green",
            Palette::Yellow => "Yellow",
            Palette::Purple => "Purple",
            Palette::Pink => "Pink",
        }
    }
}

// dmg greenish palette
pub const DMG_PALETTE: [Color; 4] = [
    Color::new(196, 207, 161, 255),
    Color::new(139, 149, 109, 255),
    Color::new(77, 83, 60, 255),
    Color::new(31, 31, 31, 255),
];

pub const FOREGROUND: Color = DMG_PALETTE[0];
pub const PRIMARY: Color = DMG_PALETTE[1];
pub const SECONDARY: Color = DMG_PALETTE[2];
pub const BACKGROUND: Color = DMG_PALETTE[3];
