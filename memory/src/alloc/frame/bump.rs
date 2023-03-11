use crate::{AllocError, FrameAlloc, FrameRange, MemoryArea, PAGE_SIZE};

pub struct BumpAlloc {
    offset: usize,
    mmap: &'static [MemoryArea],
}

impl BumpAlloc {
    #[inline]
    pub const fn new(mmap: &'static [MemoryArea]) -> Self {
        Self { offset: 0, mmap }
    }

    #[inline]
    pub const fn offset(&self) -> usize {
        self.offset
    }

    #[inline]
    pub const fn mmap(&self) -> &'static [MemoryArea] {
        self.mmap
    }
}

impl FrameAlloc for BumpAlloc {
    fn alloc(&mut self, count: usize) -> Result<FrameRange, AllocError> {
        let mut offset = self.offset;
        for area in self.mmap {
            if offset < area.size {
                let space = area.size - offset;
                if space > count * PAGE_SIZE {
                    self.offset += count * PAGE_SIZE;
                    return Ok(FrameRange::from_addr(area.start + offset, count));
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
