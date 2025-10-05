mod interpreter;
use interpreter::Interpreter;

pub trait CpuImplementation {}

pub struct Cpu {
    implementation: Box<dyn CpuImplementation>,
}

impl Cpu {
    pub fn new() -> Cpu {
        Cpu {
            implementation: Box::new(Interpreter::new()),
        }
    }
}
