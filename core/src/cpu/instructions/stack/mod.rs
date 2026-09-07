mod pop;
mod push;

pub use pop::Pop;
pub use push::Push;

use crate::{Dmg, prelude::*};

/// pushes a 16 bit value on the stack, high byte first
#[inline(always)]
pub(crate) fn push(gb: &mut Dmg, value: u16) {
    let mut sp = gb.cpu.sp.wrapping_sub(1);
    gb.write(sp, utils::high(value));

    sp = sp.wrapping_sub(1);
    gb.write(sp, utils::low(value));

    gb.cpu.sp = sp;
}

/// pops a 16 bit value from the stack
#[inline(always)]
pub(crate) fn pop(gb: &mut Dmg) -> u16 {
    let value = gb.load(gb.cpu.sp);
    gb.cpu.sp = gb.cpu.sp.wrapping_add(2);

    value
}
