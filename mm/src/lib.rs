#![no_std]

pub use addr::*;
pub use alloc::*;
pub use paging::*;

pub mod addr;
pub mod alloc;
pub mod paging;

#[derive(Debug, Clone, Copy)]
pub struct MemoryArea {
    start: usize,
    size: usize,
}

impl MemoryArea {
    pub const fn new(start: usize, size: usize) -> Self {
        Self { start, size }
    }

    pub fn start(&self) -> usize {
        self.start
    }

    pub fn size(&self) -> usize {
        self.size
    }
}
