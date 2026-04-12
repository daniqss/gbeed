use gbeed_core::SerialListener;

pub struct DebuggerSerialListener;

impl SerialListener for DebuggerSerialListener {
    fn on_transfer(&mut self, data: u8) {
        println!("through serial port -> {data:04X}");
    }
}
