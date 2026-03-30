use gbeed_core::prelude::Cartridge;
use gbeed_core::Dmg;
use std::fs;
use std::io;
use std::path::{Path, PathBuf};

pub mod layout;

pub fn truncate_name(name: &str, max_chars: usize) -> String {
    if name.len() <= max_chars {
        name.to_string()
    } else {
        format!("{}...", &name[..max_chars.saturating_sub(3)])
    }
}

pub fn save_path_from_rom(rom_path: &str) -> PathBuf {
    let path = Path::new(rom_path);
    match path.extension().and_then(|e| e.to_str()) {
        Some("gb" | "gbc") => path.with_extension("sav"),
        _ => path.with_added_extension("sav"),
    }
}

pub fn load_cartridge(
    game_path: &PathBuf,
    save_path: &mut Option<PathBuf>,
) -> Result<Cartridge, Box<dyn std::error::Error>> {
    let s_path = save_path_from_rom(game_path.to_str().unwrap_or_default());
    *save_path = Some(s_path.clone());

    let game_data = fs::read(game_path).map_err(|e| {
        io::Error::new(
            io::ErrorKind::NotFound,
            format!("Failed to read game ROM at {game_path:?}: {e}"),
        )
    })?;

    let save = match fs::read(&s_path) {
        Ok(data) => Some(data),
        Err(e) if e.kind() == io::ErrorKind::NotFound => None,
        Err(e) => {
            return Err(Box::new(io::Error::other(format!(
                "Failed to read save file at {:?}: {e}",
                s_path
            ))))
        }
    };

    Ok(Cartridge::new(&game_data, save).map_err(|e| {
        Box::new(std::io::Error::new(
            io::ErrorKind::InvalidData,
            format!("Failed to create cartridge from ROM at {game_path:?}: {e}"),
        ))
    })?)
}

pub fn save_cartridge(gb: &Dmg, save_path: &Option<PathBuf>) -> Result<(), Box<dyn std::error::Error>> {
    let Some(save_path) = save_path else {
        return Err("Save path not set".into());
    };

    if let Some(save_data) = gb.cartridge.save_game() {
        fs::write(save_path, save_data)?;
        // println!("Game saved successfully to {:?}", save_path);
    }

    Ok(())
}
