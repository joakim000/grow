#![feature(error_in_core)]

use core::error::Error;
pub type Result<T> = core::result::Result<T, Box<dyn Error>>;

pub mod zone;
pub mod ops;


