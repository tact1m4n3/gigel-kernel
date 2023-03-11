pub use frame::*;

use core::fmt::Display;

pub mod frame;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AllocError {
    NoMemory,
}

impl Display for AllocError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Self::NoMemory => f.write_str("no memory"),
        }
    }
}
