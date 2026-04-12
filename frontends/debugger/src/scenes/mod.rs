mod emulation;
mod waiting_file;

pub use emulation::EmulationScene;
pub use waiting_file::{WaitingFileScene, WaitingFileEvent};

#[derive(Default)]
pub enum EmulatorState {
    #[default]
    Exit,
    WaitingFile(WaitingFileScene),
    Emulation(EmulationScene),
}
