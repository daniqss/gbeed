pub mod emulation;
pub mod game_menu;
pub mod selection_menu;
pub mod settings_menu;

use emulation::EmulationState;
use game_menu::GameMenuState;
use selection_menu::SelectionMenuState;
use settings_menu::SettingsMenuState;

pub enum EmulatorState {
    SelectionMenu(SelectionMenuState),
    Emulation(EmulationState),
    GameMenu(GameMenuState),
    SettingsMenu(SettingsMenuState),
}

impl EmulatorState {
    pub fn get_hint(&self) -> Option<&'static str> {
        match self {
            EmulatorState::SelectionMenu(_) => Some("w/s move  enter select"),
            EmulatorState::Emulation(_) => None,
            EmulatorState::GameMenu(_) => Some("r resume  s save  l load  q quit"),
            EmulatorState::SettingsMenu(_) => Some("s save settings  q back"),
        }
    }
}
