use std::fmt::{Display, Formatter};

#[derive(Debug, PartialEq)]
pub enum Reg8 {
    A,
    F,
    B,
    C,
    D,
    E,
    H,
    L,
}

impl Display for Reg8 {
    fn fmt(&self, f: &mut Formatter) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match self {
                Reg8::A => "a",
                Reg8::F => "f",
                Reg8::B => "b",
                Reg8::C => "c",
                Reg8::D => "d",
                Reg8::E => "e",
                Reg8::H => "h",
                Reg8::L => "l",
            }
        )
    }
}

/// not including SP and PC for now
/// maybe its a good idea, we'll see
#[derive(Debug, PartialEq)]
pub enum Reg16 {
    AF,
    BC,
    DE,
    HL,
}

impl Display for Reg16 {
    fn fmt(&self, f: &mut Formatter) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match self {
                Reg16::AF => "af",
                Reg16::BC => "bc",
                Reg16::DE => "de",
                Reg16::HL => "hl",
            }
        )
    }
}
