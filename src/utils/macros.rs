#[macro_export]
macro_rules! mem_range {
    ($name:ident, $start:expr, $end:expr) => {
        paste::paste! {
            #[allow(dead_code)]
            pub const [<$name _START>]: u16 = $start;
            #[allow(dead_code)]
            pub const [<$name _END>]: u16 = $end;
            #[allow(dead_code)]
            pub const [<$name _SIZE>]: u16 = $end - $start + 1;
        }
    };
}

#[macro_export]
macro_rules! bit_accessors {
    (
        target: $target:tt;
        $( $bit:ident ),* $(,)?
    ) => {
        paste::paste! {
            $(
                #[inline]
                #[allow(dead_code)]
                pub fn [<$bit:lower>](&self) -> bool {
                    (self.$target & $bit) != 0
                }

                #[inline]
                #[allow(dead_code)]
                pub fn [<set_ $bit:lower>](&mut self, value: bool) {
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

#[macro_export]
macro_rules! field_bit_accessors {
    (
        target: $target:tt;
        $( $bit:ident ),* $(,)?
    ) => {
        paste::paste! {
            $(
                #[inline(always)]
                #[allow(dead_code)]
                pub fn [<$target _ $bit:lower>](&self) -> bool {
                    (self.$target & $bit) != 0
                }

                #[inline(always)]
                #[allow(dead_code)]
                pub fn [<set_ $target _ $bit:lower>](&mut self, value: bool) {
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

#[macro_export]
macro_rules! flag_methods {
    (
        $(
            $name:ident => $mask:ident
        ),+ $(,)?
    ) => {
        paste::paste! {
            $(
                #[inline(always)]
                #[allow(dead_code)]
                pub fn $name(&self) -> bool {
                    self.f & $mask != 0
                }

                #[inline(always)]
                #[allow(dead_code)]
                pub fn [<not_ $name>](&self) -> bool {
                    self.f & $mask == 0
                }

                #[inline(always)]
                #[allow(dead_code)]
                pub fn [<set_ $name>](&mut self) {
                    self.f |= $mask
                }

                #[inline(always)]
                #[allow(dead_code)]
                pub fn [<clear_ $name>](&mut self) {
                    self.f &= !$mask
                }
            )*
        }
    };
}

#[macro_export]
macro_rules! reg16 {
    (
        $get:ident, $set:ident,
        $hi:ident, $lo:ident
    ) => {
        #[inline]
        pub fn $get(&self) -> u16 { to_u16(self.$lo, self.$hi) }

        #[inline]
        pub fn $set(&mut self, value: u16) { from_u16(&mut self.$lo, &mut self.$hi, value); }
    };
}
