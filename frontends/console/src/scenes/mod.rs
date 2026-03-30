mod emulation;
mod game_menu;
mod selection_menu;
mod settings_menu;

pub use emulation::EmulationState;
pub use game_menu::GameMenuState;
pub use selection_menu::SelectionMenuState;
pub use settings_menu::SettingsMenuState;

pub enum EmulatorState {
    SelectionMenu(SelectionMenuState),
    Emulation(EmulationState),
    GameMenu(GameMenuState),
    SettingsMenu(SettingsMenuState),
}
