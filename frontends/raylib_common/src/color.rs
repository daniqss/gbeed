use raylib::prelude::*;

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

impl Palette {
    pub const ALL: [Palette; 9] = [
        Palette::DmgClassic,
        Palette::GrayScale,
        Palette::Red,
        Palette::Turquoise,
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
                Color::new(204, 0, 1, 255),
                Color::new(143, 0, 1, 255),
                Color::new(82, 0, 0, 255),
                Color::new(20, 0, 0, 255),
            ],
            Palette::Turquoise => [
                Color::new(3, 192, 198, 255),
                Color::new(2, 134, 139, 255),
                Color::new(1, 77, 79, 255),
                Color::new(0, 19, 20, 255),
            ],
            Palette::Blue => [
                Color::new(0, 0, 254, 255),
                Color::new(0, 0, 178, 255),
                Color::new(0, 0, 102, 255),
                Color::new(0, 0, 25, 255),
            ],

            Palette::Green => [
                Color::new(1, 204, 0, 255),
                Color::new(1, 143, 0, 255),
                Color::new(0, 82, 0, 255),
                Color::new(0, 20, 0, 255),
            ],

            Palette::Yellow => [
                Color::new(255, 255, 1, 255),
                Color::new(178, 178, 1, 255),
                Color::new(102, 102, 0, 255),
                Color::new(25, 25, 0, 255),
            ],

            Palette::Purple => [
                Color::new(118, 44, 167, 255),
                Color::new(83, 31, 117, 255),
                Color::new(47, 18, 67, 255),
                Color::new(12, 4, 17, 255),
            ],

            Palette::Pink => [
                Color::new(254, 152, 191, 255),
                Color::new(178, 106, 134, 255),
                Color::new(102, 61, 76, 255),
                Color::new(25, 15, 19, 255),
            ],
        }
    }

    pub fn name(&self) -> &str {
        match self {
            Palette::DmgClassic => "DmgClassic",
            Palette::GrayScale => "GrayScale",
            Palette::Red => "Red",
            Palette::Turquoise => "Turquoise",
            Palette::Blue => "Blue",
            Palette::Green => "Green",
            Palette::Yellow => "Yellow",
            Palette::Purple => "Purple",
            Palette::Pink => "Pink",
        }
    }

    pub fn foreground(&self) -> Color { self.colors()[0] }
    pub fn primary(&self) -> Color { self.colors()[1] }
    pub fn secondary(&self) -> Color { self.colors()[2] }
    pub fn background(&self) -> Color { self.colors()[3] }
}
