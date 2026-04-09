#[macro_export]
macro_rules! impl_cyclic_enum {
    ($name:ident, [$($variant:expr),+ $(,)?]) => {
        impl $name {
            // Contamos las expresiones convirtiendo cada una en una unidad ()
            // dentro de un array temporal para obtener su .len()
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
        }
    };
    // Regla interna para ayudar al conteo
    (@replace $variant:expr) => { () };
}
