#[macro_export]
macro_rules! mem_range {
    ($name:ident, $start:expr, $end:expr) => {
        $crate::mem_range!(pub(crate) $name, $start, $end);
    };

    ($vis:vis $name:ident, $start:expr, $end:expr) => {
        $crate::__paste! {
            #[allow(dead_code)]
            $vis const [<$name _START>]: u16 = $start;

            #[allow(dead_code)]
            $vis const [<$name _END>]: u16 = $end;

            #[allow(dead_code)]
            $vis const [<$name _SIZE>]: u16 = ($end) - ($start) + 1;
        }
    };
}

macro_rules! bit_accessors {
    (
        $vis:vis target: $target:tt;
        $( $bit:ident ),* $(,)?
    ) => {
        $crate::__paste! {
            $(
                #[inline(always)]
                #[allow(dead_code)]
                $vis fn [<$bit:lower>](&self) -> bool {
                    (self.$target & $bit) != 0
                }

                #[inline(always)]
                #[allow(dead_code)]
                $vis fn [<set_ $bit:lower>](&mut self, value: bool) {
                    if value {
                        self.$target |= $bit;
                    } else {
                        self.$target &= !$bit;
                    }
                }
            )*
        }
    };
}

macro_rules! field_bit_accessors {
    (
        $vis:vis target: $target:tt;
        $( $bit:ident ),* $(,)?
    ) => {
        $crate::__paste! {
            $(
                #[inline(always)]
                #[allow(dead_code)]
                $vis fn [<$target _ $bit:lower>](&self) -> bool {
                    (self.$target & $bit) != 0
                }

                #[inline(always)]
                #[allow(dead_code)]
                $vis fn [<set_ $target _ $bit:lower>](&mut self, value: bool) {
                    if value {
                        self.$target |= $bit;
                    } else {
                        self.$target &= !$bit;
                    }
                }
            )*
        }
    };
}

macro_rules! flag_methods {
    (
        $vis:vis
        $( $name:ident => $mask:ident ),+ $(,)?
    ) => {
        $crate::__paste! {
            $(
                #[inline(always)]
                #[allow(dead_code)]
                $vis fn $name(&self) -> bool {
                    self.f & $mask != 0
                }

                #[inline(always)]
                #[allow(dead_code)]
                $vis fn [<not_ $name>](&self) -> bool {
                    self.f & $mask == 0
                }

                #[inline(always)]
                #[allow(dead_code)]
                $vis fn [<set_ $name>](&mut self) {
                    self.f |= $mask
                }

                #[inline(always)]
                #[allow(dead_code)]
                $vis fn [<clear_ $name>](&mut self) {
                    self.f &= !$mask
                }
            )*
        }
    };
}

macro_rules! reg16 {
    (
        $vis:vis $get:ident, $set:ident,
        $hi:ident, $lo:ident
    ) => {
        #[inline]
        $vis fn $get(&self) -> u16 { crate::utils::to_u16(self.$lo, self.$hi) }

        #[inline]
        $vis fn $set(&mut self, value: u16) { crate::utils::from_u16(&mut self.$lo, &mut self.$hi, value); }
    };
}

// an instruction generic over its operands is listed as it is written, `Or<R8>`, and its variant is
// named after the instantiation, `OrR8`, so the enum needs no type alias to name each combination

/// Generates a enum with variants for each instruction and its operands
macro_rules! instruction_dispatch {
    (
        $vis:vis enum $name:ident {
            $( $instruction:ident $(< $( $operand:ident ),+ >)? ),+ $(,)?
        }
    ) => {
        $crate::__paste! {
        $vis enum $name {
            $( [<$instruction $($($operand)+)?>]($instruction $(<$($operand),+>)?), )+
        }

        // the variants are built through `into`,
        // so the tables that decode the opcodes stay unaware of the enum
        // and read as a plain list of instructions
        $(
            impl From<$instruction $(<$($operand),+>)?> for $name {
                #[inline(always)]
                fn from(instruction: $instruction $(<$($operand),+>)?) -> Self {
                    $name::[<$instruction $($($operand)+)?>](instruction)
                }
            }
        )+

        impl $name {
            #[inline(always)]
            pub(crate) fn exec(&mut self, gb: &mut Dmg) -> InstructionResult {
                match self {
                    $( $name::[<$instruction $($($operand)+)?>](instruction) => instruction.exec(gb), )+
                }
            }

            #[inline(always)]
            pub fn info(&self) -> (u8, u8) {
                match self {
                    $( $name::[<$instruction $($($operand)+)?>](instruction) => instruction.info(), )+
                }
            }

            /// Assembly representation of the instruction and its operands
            pub fn disassembly(&self) -> String {
                match self {
                    $( $name::[<$instruction $($($operand)+)?>](instruction) => instruction.disassembly(), )+
                }
            }
        }
        }

        impl core::fmt::Display for $name {
            fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
                write!(f, "{}", self.disassembly())
            }
        }

        impl core::fmt::Debug for $name {
            fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
                write!(f, "{}", self.disassembly())
            }
        }
    };
}

macro_rules! mbc_dispatch {
    ($self:ident.$method:ident($( $arg:expr ),*)) => {
        match $self {
            Mbc::Mbc0(mbc) => mbc.$method($( $arg ),*),
            Mbc::Mbc1(mbc) => mbc.$method($( $arg ),*),
            Mbc::Mbc2(mbc) => mbc.$method($( $arg ),*),
            Mbc::Mbc3(mbc) => mbc.$method($( $arg ),*),
            Mbc::Mbc5(mbc) => mbc.$method($( $arg ),*),
        }
    };
}

pub(crate) use {
    bit_accessors, field_bit_accessors, flag_methods, instruction_dispatch, mbc_dispatch, reg16,
};
