use super::license::get_license;
use crate::prelude::*;

const TITLE_START_ADDR: usize = 0x0134;
const TITLE_END_ADDR: usize = 0x0143;

pub struct Cartridge {
    raw: Vec<u8>,
    is_pre_sgb: bool,
    license: Option<String>,
    title: String,
}

impl Cartridge {
    pub fn new(raw: Vec<u8>) -> Result<Self> {
        let title = &raw[TITLE_START_ADDR..TITLE_END_ADDR]
            .iter()
            .map(|&c| c as char)
            .collect::<String>();

        let (is_pre_sgb, license) = get_license(&raw);

        Ok(Self {
            raw,
            is_pre_sgb,
            title: title.to_string(),
            license,
        })
    }
}

impl std::fmt::Debug for Cartridge {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for &byte in &self.raw {
            let c = byte as char;
            match c.is_ascii_graphic() {
                true => write!(f, "{}", c)?,
                false => write!(f, ".")?,
            }
        }
        Ok(())
    }
}

impl std::fmt::Display for Cartridge {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}\n", self.title)?;
        write!(
            f,
            "{}, {}",
            match self.is_pre_sgb {
                true => "Old license",
                false => "New license",
            },
            match &self.license {
                Some(l) => l,
                None => "None",
            }
        )?;

        Ok(())
    }
}
