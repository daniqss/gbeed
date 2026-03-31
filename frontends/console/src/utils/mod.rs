pub mod layout;
pub mod roms;

pub fn truncate_name(name: &str, max_chars: usize) -> String {
    if name.len() <= max_chars {
        name.to_string()
    } else {
        format!("{}...", &name[..max_chars.saturating_sub(3)])
    }
}
