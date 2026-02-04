pub use crate::error::Error;
pub use crate::utils;
pub use crate::{bit_accessors, field_bit_accessors, mem_range};
pub use std::{
    cell::RefCell,
    ops::{Index, IndexMut},
    rc::Rc,
};

pub type Result<T> = std::result::Result<T, Error>;
