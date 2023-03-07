use crate::{AllocError, FrameAlloc, FrameCount, MemoryArea, PhysAddr, PAGE_SIZE};

pub struct BumpAlloc {
    offset: usize,
    areas: &'static [MemoryArea],
}

impl BumpAlloc {
    pub const fn new(areas: &'static [MemoryArea]) -> Self {
        Self { offset: 0, areas }
    }

    pub const fn offset(&self) -> usize {
        self.offset
    }
}

impl FrameAlloc for BumpAlloc {
    fn alloc(&mut self, count: FrameCount) -> Result<PhysAddr, AllocError> {
        let mut offset = self.offset;
        for area in self.areas {
            if offset < area.size {
                self.offset += count * PAGE_SIZE;
                return Ok(PhysAddr::new(area.start + offset));
            }
            offset -= area.size;
        }
        Err(AllocError::NoMemory)
    }

    fn free(&mut self, _first: PhysAddr, _count: FrameCount) {
        unimplemented!("bump allocator: can't free frames");
    }
}
