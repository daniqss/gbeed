#[macro_export]
macro_rules! impl_cyclic_enum {
    ($name:ident, [$($variant:expr),+ $(,)?]) => {
        impl $name {
            pub const ALL: [$name; [$(impl_cyclic_enum!(@replace $variant)),+].len()] = [
                $($variant),+
            ];

            #[inline(always)]
            pub fn next(&self) -> Self {
                let index = Self::ALL.iter().position(|x| x == self).unwrap_or(0);
                Self::ALL[(index + 1) % Self::ALL.len()]
            }

            #[inline(always)]
            pub fn prev(&self) -> Self {
                let index = Self::ALL.iter().position(|x| x == self).unwrap_or(0);
                Self::ALL[(index + Self::ALL.len() - 1) % Self::ALL.len()]
            }

            #[inline(always)]
            pub fn position(&self) -> usize {
                Self::ALL.iter().position(|x| x == self).unwrap_or(0)
            }
        }
    };

    (@replace $variant:expr) => { () };
}
