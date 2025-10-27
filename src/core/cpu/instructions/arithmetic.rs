use crate::core::{
    cpu::flags::{CARRY_FLAG_MASK, check_carry, check_half_carry, check_zero},
    cpu::instructions::InstructionEffect,
};
pub enum Addend {
    Immediate(u8),
    RegisterB(u8),
    RegisterC(u8),
    RegisterD(u8),
    RegisterE(u8),
    RegisterH(u8),
    RegisterL(u8),
    PointedByHL(u8),
}

pub struct ADC {
    addend: Addend,
}

impl ADC {
    pub fn new(addend: Addend) -> Self { ADC { addend } }

    pub fn exec(addend: Addend, f: u8, a: &mut u8) -> InstructionEffect {
        let (addend, cycles, len) = match addend {
            Addend::Immediate(n8) => (n8, 2, 2),
            Addend::RegisterB(b) => (b, 1, 1),
            Addend::RegisterC(c) => (c, 1, 1),
            Addend::RegisterD(d) => (d, 1, 1),
            Addend::RegisterE(e) => (e, 1, 1),
            Addend::RegisterH(h) => (h, 1, 1),
            Addend::RegisterL(l) => (l, 1, 1),
            Addend::PointedByHL(value) => (value, 2, 1),
        };

        let mut result = a.wrapping_add(addend);
        result = result.wrapping_add(if (f & CARRY_FLAG_MASK) != 0 { 1 } else { 0 });
        let flags = check_zero(result) | check_carry(result, *a) | check_half_carry(result, *a);

        InstructionEffect::new(len, cycles, Some(flags))
    }
}

impl std::fmt::Display for ADC {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "ADC A,{}",
            match self.addend {
                Addend::Immediate(n8) => format!("{}", n8),
                Addend::RegisterB(_) => "B".to_string(),
                Addend::RegisterC(_) => "C".to_string(),
                Addend::RegisterD(_) => "D".to_string(),
                Addend::RegisterE(_) => "E".to_string(),
                Addend::RegisterH(_) => "H".to_string(),
                Addend::RegisterL(_) => "L".to_string(),
                Addend::PointedByHL(_) => "[HL]".to_string(),
            }
        )
    }
}
