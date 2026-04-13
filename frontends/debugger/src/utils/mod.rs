pub mod components;
mod layout;

use gbeed_raylib_common::color::DMG_CLASSIC_PALETTE;
pub use layout::{HEADER_HEIGHT, Layout, PANEL_PADDING};
use raylib::prelude::*;

pub const FOREGROUND: Color = DMG_CLASSIC_PALETTE[0];
pub const SECONDARY: Color = DMG_CLASSIC_PALETTE[1];
pub const PRIMARY: Color = DMG_CLASSIC_PALETTE[2];
pub const BACKGROUND: Color = DMG_CLASSIC_PALETTE[3];
