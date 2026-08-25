use crate::apu::AudioPlayer;
use crate::ppu::Renderer;
use crate::serial::SerialListener;

pub trait Controller: SerialListener + Renderer + AudioPlayer {}

/// Any type implementing the three plataform traits is a controller
impl<T: SerialListener + Renderer + AudioPlayer> Controller for T {}

#[macro_export]
macro_rules! impl_controller {
    ($ty:ty : $($trait:path),+ $(,)?) => {
        $(impl $trait for $ty {})+
    };
}

pub struct DefaultController {}
impl_controller!(DefaultController: Renderer, SerialListener, AudioPlayer);
