use memory::{
    BitmapAlloc, BumpAlloc, Frame, FrameAlloc, FrameRange, MemoryArea, PageFlags, PageMapper,
    PAGE_MASK, PAGE_SIZE,
};

use multiboot2::{AreaType, BootInfo, MemoryMapTag, TagType};

use crate::sync::Mutex;

const FIRST_FREE_FRAME: usize = 0x100000;

static mut MMAP: [MemoryArea; 512] = [MemoryArea::new(0, 0); 512];

pub fn init(boot_info: &BootInfo) {
    extern "C" {
        static kend: u8;
    }

    let mmap_tag: &MemoryMapTag = boot_info.find_tag(TagType::Mmap).unwrap();

    let mut len = 0;
    for b_area in mmap_tag.areas() {
        if b_area.typ() != AreaType::Available {
            continue;
        }

        let mut start = b_area
            .start_addr()
            .max(FIRST_FREE_FRAME)
            .max(boot_info.end_addr())
            .max(unsafe { &kend } as *const _ as usize);
        start += (PAGE_SIZE - (start & PAGE_MASK)) & PAGE_MASK;

        let mut end = b_area.end_addr();
        end += (PAGE_SIZE - (start & PAGE_MASK)) & PAGE_MASK;

        let size = end.saturating_sub(start);
        if size == 0 {
            continue;
        }

        unsafe { MMAP[len] = MemoryArea::new(start, size) }
        len += 1;
    }

    let mmap = unsafe { &MMAP[0..len] };
    let mut bump_alloc = BumpAlloc::new(mmap);

    {
        let mut mapper = PageMapper::new(&mut bump_alloc);
        for b_area in mmap_tag.areas() {
            let mut addr = b_area.start_addr();
            while addr < b_area.end_addr() {
                mapper
                    .map_page(
                        addr,
                        Frame::from_addr(addr),
                        PageFlags::PRESENT | PageFlags::WRITE,
                    )
                    .unwrap();
                addr += PAGE_SIZE;
            }
        }
        mapper.make_current();
    }

    let bitmap_alloc = BitmapAlloc::new(bump_alloc).unwrap();
    *INNER_ALLOC.lock() = Some(bitmap_alloc);
}

static INNER_ALLOC: Mutex<Option<BitmapAlloc>> = Mutex::new(None);
pub static FRAME_ALLOC: LockedAlloc = LockedAlloc;

#[derive(Clone, Copy)]
pub struct LockedAlloc;

impl FrameAlloc for LockedAlloc {
    fn alloc(&mut self, count: usize) -> Result<FrameRange, memory::AllocError> {
        if let Some(ref mut alloc) = *INNER_ALLOC.lock() {
            alloc.alloc(count)
        } else {
            panic!("no frame allocator");
        }
    }

    fn free(&mut self, frames: FrameRange) {
        if let Some(ref mut alloc) = *INNER_ALLOC.lock() {
            alloc.free(frames)
        } else {
            panic!("no frame allocator");
        }
    }
}
