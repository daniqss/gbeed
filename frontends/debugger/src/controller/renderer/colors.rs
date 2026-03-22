use raylib::color::Color;

pub const GB_PALETTE: [Color; 4] = [
    Color {
        r: 196,
        g: 207,
        b: 161,
        a: 255,
    },
    Color {
        r: 139,
        g: 149,
        b: 109,
        a: 255,
    },
    Color {
        r: 77,
        g: 83,
        b: 60,
        a: 255,
    },
    Color {
        r: 31,
        g: 31,
        b: 31,
        a: 255,
    },
];

pub const FOREGROUND: Color = GB_PALETTE[0];
pub const PRIMARY: Color = GB_PALETTE[1];
pub const SECONDARY: Color = GB_PALETTE[2];
pub const BACKGROUND: Color = GB_PALETTE[3];
