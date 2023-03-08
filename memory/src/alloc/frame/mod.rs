pub use bump::*;

use crate::{AllocError, FrameRange};

pub mod bump;

pub trait FrameAlloc {
    fn alloc(&mut self, count: usize) -> Result<FrameRange, AllocError>;
    fn free(&mut self, frames: FrameRange);
}
