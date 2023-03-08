#![no_std]

pub use alloc::*;
pub use paging::*;

pub mod alloc;
pub mod paging;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
#[repr(transparent)]
pub struct Frame(usize);

impl Frame {
    #[inline]
    pub const fn new(addr: usize) -> Self {
        Self(addr & !PAGE_MASK)
    }

    #[inline]
    pub const fn addr(self) -> usize {
        self.0
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct FrameRange {
    start: usize,
    count: usize,
}

impl FrameRange {
    #[inline]
    pub const fn new(start: usize, count: usize) -> Self {
        Self {
            start: start & !PAGE_MASK,
            count,
        }
    }

    fn get(&self, idx: usize) -> Frame {
        if idx >= self.count {
            panic!("index out of range")
        }
        Frame::new(self.start + idx * self.count)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct MemoryArea {
    start: usize,
    size: usize,
}

impl MemoryArea {
    pub const fn new(start: usize, size: usize) -> Self {
        Self { start, size }
    }

    #[inline]
    pub const fn start_addr(&self) -> usize {
        self.start
    }

    #[inline]
    pub const fn size(&self) -> usize {
        self.size
    }

    #[inline]
    pub const fn end_addr(&self) -> usize {
        self.start_addr() + self.size()
    }
}
