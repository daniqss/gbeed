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
