#[derive(thiserror::Error, Debug)]

pub enum Error {
    #[error("Error: {0}")]
    Generic(String),

    #[error(transparent)]
    Io(#[from] std::io::Error),

    #[error("SDL2 Error: {0}")]
    Sdl2(String),

    #[error("Error loading font: {0}")]
    FontLoad(String),
}

impl From<sdl2::ttf::FontError> for Error {
    fn from(err: sdl2::ttf::FontError) -> Self { Error::FontLoad(err.to_string()) }
}

impl From<String> for Error {
    fn from(err: String) -> Self { Error::Sdl2(err.to_string()) }
}

impl From<sdl2::video::WindowBuildError> for Error {
    fn from(err: sdl2::video::WindowBuildError) -> Self { Error::Sdl2(err.to_string()) }
}

impl From<sdl2::IntegerOrSdlError> for Error {
    fn from(err: sdl2::IntegerOrSdlError) -> Self { Error::Sdl2(err.to_string()) }
}
