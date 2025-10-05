use super::CpuImplementation;
pub struct Interpreter;

impl CpuImplementation for Interpreter {}

impl Interpreter {
    pub fn new() -> Interpreter {
        Interpreter
    }
}
