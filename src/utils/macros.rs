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
