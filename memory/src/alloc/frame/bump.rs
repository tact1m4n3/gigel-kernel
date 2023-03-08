use crate::{AllocError, FrameAlloc, FrameRange, MemoryArea, PAGE_SIZE};

pub struct BumpAlloc {
    offset: usize,
    areas: &'static [MemoryArea],
}

impl BumpAlloc {
    #[inline]
    pub const fn new(areas: &'static [MemoryArea]) -> Self {
        Self { offset: 0, areas }
    }

    #[inline]
    pub const fn offset(&self) -> usize {
        self.offset
    }
}

impl FrameAlloc for BumpAlloc {
    fn alloc(&mut self, count: usize) -> Result<FrameRange, AllocError> {
        let mut offset = self.offset;
        for area in self.areas {
            if offset < area.size {
                let space = area.size - offset;
                if space > count * PAGE_SIZE {
                    self.offset += count * PAGE_SIZE;
                    return Ok(FrameRange::new(area.start + offset, count));
                }
            } else {
                offset -= area.size;
            }
        }
        Err(AllocError::NoMemory)
    }

    fn free(&mut self, _frames: FrameRange) {
        unimplemented!("bump allocator: can't free frames");
    }
}
