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
