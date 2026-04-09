use crate::impl_cyclic_enum;

#[derive(Debug, Copy, Clone, PartialEq)]
pub enum SpeedUpMode {
    Toggle(bool),
    Hold,
}

use SpeedUpMode::*;
impl_cyclic_enum!(SpeedUpMode, [Toggle(true), Toggle(false), Hold]);
impl Default for SpeedUpMode {
    fn default() -> Self { Toggle(false) }
}

#[derive(Debug, Default, Copy, Clone, PartialEq)]
pub enum SpeedUpMultiplier {
    #[default]
    OneAndHalf,
    Double,
    Cuadruple,
}

use SpeedUpMultiplier::*;
impl_cyclic_enum!(SpeedUpMultiplier, [OneAndHalf, Double, Cuadruple]);
impl SpeedUpMultiplier {
    pub fn get_multiplier(&self) -> f32 {
        match self {
            OneAndHalf => 1.5,
            Double => 2.0,
            Cuadruple => 4.0,
        }
    }
}

#[repr(u8)]
#[derive(Debug, Default, Copy, Clone, PartialEq)]
pub enum TargetedFps {
    Target30 = 30,
    #[default]
    Target60 = 60,
    Unlimited = 0,
}

use TargetedFps::*;
impl_cyclic_enum!(TargetedFps, [Target30, Target60, Unlimited]);
