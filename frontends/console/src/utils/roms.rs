use std::{
    fs, io,
    path::{Path, PathBuf},
};

use gbeed_core::{Cartridge, Dmg};

#[inline(always)]
fn home_dir() -> PathBuf {
    std::env::var("HOME")
        .map(PathBuf::from)
        .unwrap_or_else(|_| PathBuf::from("."))
}

#[inline(always)]
pub fn roms_dir() -> PathBuf {
    if cfg!(debug_assertions) {
        PathBuf::from(env!("WORKSPACE_ROOT")).join("roms")
    } else {
        home_dir().join("roms")
    }
}

#[inline(always)]
pub fn saves_dir() -> PathBuf {
    if cfg!(debug_assertions) {
        PathBuf::from(env!("WORKSPACE_ROOT")).join("saves")
    } else {
        home_dir().join("saves")
    }
}
pub fn find_roms() -> Vec<PathBuf> {
    let dir = roms_dir();

    let mut roms: Vec<PathBuf> = fs::read_dir(&dir)
        .map(|entries| {
            entries
                .filter_map(Result::ok)
                .map(|entry| entry.path())
                .filter(|path| {
                    path.is_file()
                        && path
                            .extension()
                            .and_then(|ext| ext.to_str())
                            .map(|ext| ext.eq_ignore_ascii_case("gb") || ext.eq_ignore_ascii_case("gbc"))
                            .unwrap_or(false)
                })
                .collect()
        })
        .unwrap_or_default();

    roms.sort();
    roms
}

pub fn load_cartridge(
    game_path: &PathBuf,
    save_path: &mut Option<PathBuf>,
) -> Result<Cartridge, Box<dyn std::error::Error>> {
    let s_path = save_path_from_rom(game_path);
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
                "Failed to read save file at {s_path:?}: {e}"
            ))));
        }
    };

    Cartridge::new(&game_data, save).map_err(|e| {
        Box::new(io::Error::new(
            io::ErrorKind::InvalidData,
            format!("Failed to create cartridge from ROM at {game_path:?}: {e}"),
        )) as Box<dyn std::error::Error>
    })
}

pub fn save_cartridge(gb: &Dmg, save_path: &Option<PathBuf>) -> Result<(), Box<dyn std::error::Error>> {
    let Some(save_path) = save_path else {
        return Err("Save path not set".into());
    };

    if let Some(save_data) = gb.cartridge.save_game() {
        // Ensure the saves directory exists before writing
        if let Some(parent) = save_path.parent() {
            fs::create_dir_all(parent)?;
        }
        fs::write(save_path, save_data)?;
    }

    Ok(())
}

/// Builds the .sav path for a given ROM path, redirecting it to the
/// correct saves directory depending on the build profile.
///
/// # Debug build
/// `/path/to/project/roms/pokemon.gb` -> `/path/to/project/saves/pokemon.sav`
///
/// # Release build
/// `/home/user/roms/pokemon.gb` -> `/home/user/saves/pokemon.sav`
#[inline(always)]
pub fn save_path_from_rom(rom_path: &Path) -> PathBuf {
    let stem = rom_path.file_stem().unwrap_or(rom_path.as_os_str());

    saves_dir().join(stem).with_extension("sav")
}
