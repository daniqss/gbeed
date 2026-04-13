mod emulation;
mod waiting_file;

pub use emulation::EmulationScene;
pub use waiting_file::WaitingFileScene;

#[derive(Debug)]
pub enum EmulatorState {
    WaitingFile(WaitingFileScene),
    Emulation(EmulationScene),
    Exit,
}
