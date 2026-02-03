use std::fmt::{Display, Formatter};

#[derive(Debug, PartialEq)]
pub enum Reg {
    A,
    F,
    B,
    C,
    D,
    E,
    H,
    L,
    AF,
    BC,
    DE,
    HL,
}

impl Display for Reg {
    fn fmt(&self, f: &mut Formatter) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match self {
                Reg::A => "a",
                Reg::F => "f",
                Reg::B => "b",
                Reg::C => "c",
                Reg::D => "d",
                Reg::E => "e",
                Reg::H => "h",
                Reg::L => "l",
                Reg::AF => "af",
                Reg::BC => "bc",
                Reg::DE => "de",
                Reg::HL => "hl",
            }
        )
    }
}
