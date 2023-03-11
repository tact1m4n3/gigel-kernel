pub use bitmap::*;
pub use bump::*;

use crate::{AllocError, FrameRange};

pub mod bitmap;
pub mod bump;

pub trait FrameAlloc {
    fn alloc(&mut self, count: usize) -> Result<FrameRange, AllocError>;
    fn free(&mut self, frames: FrameRange);
}

impl<T: FrameAlloc> FrameAlloc for &mut T {
    fn alloc(&mut self, count: usize) -> Result<FrameRange, AllocError> {
        T::alloc(self, count)
    }

    fn free(&mut self, frames: FrameRange) {
        T::free(self, frames)
    }
}
