use std::fmt::{Display, Formatter};

#[derive(Debug, PartialEq, Clone, Copy)]
pub enum Register8 {
    A,
    F,
    B,
    C,
    D,
    E,
    H,
    L,
}

impl Display for Register8 {
    fn fmt(&self, f: &mut Formatter) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match self {
                Register8::A => "a",
                Register8::F => "f",
                Register8::B => "b",
                Register8::C => "c",
                Register8::D => "d",
                Register8::E => "e",
                Register8::H => "h",
                Register8::L => "l",
            }
        )
    }
}

/// not including SP and PC for now
/// maybe its a good idea, we'll see
#[derive(Debug, PartialEq, Clone, Copy)]
pub enum Register16 {
    AF,
    BC,
    DE,
    HL,
}

impl Display for Register16 {
    fn fmt(&self, f: &mut Formatter) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match self {
                Register16::AF => "af",
                Register16::BC => "bc",
                Register16::DE => "de",
                Register16::HL => "hl",
            }
        )
    }
}
