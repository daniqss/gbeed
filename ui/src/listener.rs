use gbeed_core::SerialListener;

pub struct RaylibSerialListener;

impl SerialListener for RaylibSerialListener {
    fn on_transfer(&mut self, data: u8) {
        print!("{:04X}", data);
    }
}
