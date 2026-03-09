use gbeed_core::SerialListener;
use std::io::{self, Write};

pub struct RaylibSerialListener;

impl SerialListener for RaylibSerialListener {
    fn on_transfer(&mut self, data: u8) {
        print!("{}", data as char);
        io::stdout().flush().unwrap();
    }
}
