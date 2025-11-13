mod jp;

use std::fmt::Display;

pub use jp::Jp;

#[derive(Debug, PartialEq)]
pub enum JumpCondition {
    Zero(bool),
    NotZero(bool),
    Carry(bool),
    NotCarry(bool),
    None,
}

impl JumpCondition {
    pub fn should_jump(&self) -> bool {
        match self {
            JumpCondition::Zero(cond) => *cond,
            JumpCondition::NotZero(cond) => *cond,
            JumpCondition::Carry(cond) => *cond,
            JumpCondition::NotCarry(cond) => *cond,
            JumpCondition::None => true,
        }
    }
}

impl Display for JumpCondition {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            JumpCondition::Zero(_) => write!(f, "z,"),
            JumpCondition::NotZero(_) => write!(f, "nz,"),
            JumpCondition::Carry(_) => write!(f, "c,"),
            JumpCondition::NotCarry(_) => write!(f, "nc,"),
            JumpCondition::None => write!(f, ""),
        }
    }
}
