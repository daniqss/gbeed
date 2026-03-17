#[derive(Debug, Default, Clone)]
pub struct Rumble {
    pub enabled: bool,
}

impl Rumble {
    pub fn new() -> Self { Self { enabled: false } }
}
