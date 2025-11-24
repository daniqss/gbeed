pub use crate::bit_accessors;
pub use crate::error::Error;
pub use crate::utils;
pub use std::{cell::RefCell, rc::Rc};

pub type Result<T> = std::result::Result<T, Error>;
