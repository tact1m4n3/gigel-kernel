#![no_std]

use core::marker::PhantomData;

pub use alloc::*;
pub use paging::*;

pub mod alloc;
pub mod paging;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
#[repr(transparent)]
pub struct Frame(usize);

impl Frame {
    #[inline]
    pub const fn from_idx(idx: usize) -> Self {
        Self(idx)
    }

    #[inline]
    pub const fn from_addr(addr: usize) -> Self {
        Self(addr >> PAGE_SHIFT)
    }

    #[inline]
    pub const fn idx(self) -> usize {
        self.0
    }

    #[inline]
    pub const fn addr(self) -> usize {
        self.0 << PAGE_SHIFT
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct FrameRange {
    first: usize,
    count: usize,
}

impl FrameRange {
    pub const fn from_idx(first: usize, count: usize) -> Self {
        if count == 0 {
            panic!("frame range with no frames");
        }
        Self { first, count }
    }

    pub const fn from_addr(first: usize, count: usize) -> Self {
        if count == 0 {
            panic!("frame range with no frames");
        }
        Self {
            first: first >> PAGE_SHIFT,
            count,
        }
    }

    #[inline]
    pub fn addr(&self) -> usize {
        self.first >> PAGE_SHIFT
    }

    pub fn get(&self, idx: usize) -> Frame {
        if idx >= self.count {
            panic!("index out of range")
        }
        Frame(self.first + idx)
    }

    #[inline]
    pub fn first(&self) -> Frame {
        self.get(0)
    }

    #[inline]
    pub fn last(&self) -> Frame {
        self.get(self.count - 1)
    }

    #[inline]
    pub fn iter(&self) -> FrameIter<'_> {
        FrameIter {
            current: self.first,
            last: self.first + self.count,
            phantom: PhantomData {},
        }
    }
}

pub struct FrameIter<'a> {
    current: usize,
    last: usize,
    phantom: PhantomData<&'a FrameRange>,
}

impl<'a> Iterator for FrameIter<'a> {
    type Item = Frame;

    fn next(&mut self) -> Option<Self::Item> {
        if self.current < self.last {
            let frame = Frame::from_idx(self.current);
            self.current += 1;
            Some(frame)
        } else {
            None
        }
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
