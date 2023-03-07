pub use bump::*;

use crate::{AllocError, PhysAddr};

pub mod bump;

pub type FrameCount = usize;

pub trait FrameAlloc {
    fn alloc(&mut self, count: FrameCount) -> Result<PhysAddr, AllocError>;
    fn free(&mut self, start: PhysAddr, count: FrameCount);
}
