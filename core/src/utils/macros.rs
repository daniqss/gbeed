#[macro_export]
macro_rules! mem_range {
    ($name:ident, $start:expr, $end:expr) => {
        $crate::__paste! {
            #[allow(dead_code)]
            pub const [<$name _START>]: u16 = $start;
            #[allow(dead_code)]
            pub const [<$name _END>]: u16 = $end;
            #[allow(dead_code)]
            pub const [<$name _SIZE>]: u16 = ($end) - ($start) + 1;
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
        $vis fn $get(&self) -> u16 { to_u16(self.$lo, self.$hi) }

        #[inline]
        $vis fn $set(&mut self, value: u16) { from_u16(&mut self.$lo, &mut self.$hi, value); }
    };
}

pub(crate) use {bit_accessors, field_bit_accessors, flag_methods, reg16};
