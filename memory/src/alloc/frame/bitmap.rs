use core::{ptr, slice};

use crate::{AllocError, BumpAlloc, Frame, FrameAlloc, FrameRange, MemoryArea, PAGE_SIZE};

pub struct BitmapAlloc {
    bitmap: FrameBitmap,
}

impl BitmapAlloc {
    pub fn new(mut bump_alloc: BumpAlloc) -> Result<Self, AllocError> {
        let mmap = bump_alloc.mmap();
        let last_addr = Self::last_free_addr(mmap);

        let n_frames = last_addr / PAGE_SIZE + 1;
        let mut bitmap = FrameBitmap::new(&mut bump_alloc, n_frames)?;

        let mut offset = bump_alloc.offset();
        for area in mmap {
            if offset >= area.size() {
                offset -= area.size();
                continue;
            }

            let mut addr = area.start_addr() + offset;
            offset = 0;
            while addr < area.end_addr() {
                bitmap.free_frame(Frame::from_addr(addr));
                addr += PAGE_SIZE;
            }
        }

        Ok(Self { bitmap })
    }

    fn last_free_addr(mmap: &'static [MemoryArea]) -> usize {
        let mut addr = 0;
        for area in mmap {
            if area.end_addr() > addr {
                addr = area.end_addr();
            }
        }
        addr
    }
}

impl FrameAlloc for BitmapAlloc {
    fn alloc(&mut self, count: usize) -> Result<FrameRange, AllocError> {
        if let Some(frames) = self.bitmap.first_free_frames(count) {
            for frame in frames.iter() {
                self.bitmap.alloc_frame(frame);
            }
            Ok(frames)
        } else {
            Err(AllocError::NoMemory)
        }
    }

    fn free(&mut self, frames: FrameRange) {
        for frame in frames.iter() {
            self.bitmap.free_frame(frame);
        }
    }
}

struct FrameBitmap {
    inner: &'static mut [u8],
    n_frames: usize,
}

impl FrameBitmap {
    pub fn new(bump_alloc: &mut BumpAlloc, n_frames: usize) -> Result<Self, AllocError> {
        let size = n_frames / 8 + 1;
        Ok(Self {
            inner: unsafe {
                let bitmap = bump_alloc.alloc(size / PAGE_SIZE + 1)?.first().addr() as *mut u8;
                ptr::write_bytes(bitmap, 0, size);
                slice::from_raw_parts_mut(bitmap, size)
            },
            n_frames,
        })
    }

    fn is_used(&self, frame: Frame) -> bool {
        if frame.idx() < self.n_frames {
            let idx = frame.idx() / 8;
            let bit = frame.idx() % 8;
            (self.inner[idx] & (1 << bit)) != 0
        } else {
            false
        }
    }

    fn alloc_frame(&mut self, frame: Frame) {
        if frame.idx() < self.n_frames {
            let idx = frame.idx() / 8;
            let bit = frame.idx() % 8;
            self.inner[idx] &= !(1 << bit);
        }
    }

    fn free_frame(&mut self, frame: Frame) {
        if frame.idx() < self.n_frames {
            let idx = frame.idx() / 8;
            let bit = frame.idx() % 8;
            self.inner[idx] |= 1 << bit;
        }
    }

    fn first_free_frames(&self, count: usize) -> Option<FrameRange> {
        for i in 0..self.n_frames {
            let mut bad = false;
            for j in 0..count {
                if !self.is_used(Frame(i + j)) {
                    bad = true;
                    break;
                }
            }
            if !bad {
                return Some(FrameRange::from_idx(i, count));
            }
        }
        None
    }
}
