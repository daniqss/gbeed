mod call;
mod jp;
mod jr;
mod ret;
mod reti;
mod rst;

use std::fmt::Display;

pub use call::Call;
pub use jp::Jp;
pub use jr::Jr;
pub use ret::Ret;
pub use reti::Reti;
pub use rst::Rst;

/// jump condition for conditional jumps
/// jump Zero if zero flag is set, `self.f & ZERO_FLAG_MASK != 0`
/// jump NotZero if zero flag is not set, `self.f & ZERO_FLAG_MASK == 0`
/// jump Carry if carry flag is set
/// jump NotCarry if carry flag is not set
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
