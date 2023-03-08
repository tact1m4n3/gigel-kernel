use memory::{BumpAlloc, FrameAlloc, FrameRange, MemoryArea, PAGE_MASK, PAGE_SIZE};

use multiboot2::{AreaType, BootInfo, MemoryMapTag, TagType};

use crate::{println, sync::Mutex};

static mut MMAP: [MemoryArea; 512] = [MemoryArea::new(0, 0); 512];

pub fn init(boot_info: &BootInfo) {
    let mmap_tag: &MemoryMapTag = boot_info.find_tag(TagType::Mmap).unwrap();

    extern "C" {
        static kstart: u8;
        static kend: u8;
    }

    let boot_info_start = boot_info.start_addr();
    let boot_info_end = boot_info.end_addr();

    let kernel_start = unsafe { &kstart } as *const _ as usize;
    let kernel_end = unsafe { &kend } as *const _ as usize;

    let mut len = 0;
    for b_area in mmap_tag.areas() {
        if b_area.typ() != AreaType::Available {
            continue;
        }

        let mut start = b_area.start_addr();
        let mut size = b_area.size();

        if start < boot_info_end && start + size > boot_info_start {
            start = start.max(boot_info_end);
        }

        if start < kernel_end && start + size > kernel_start {
            start = start.max(boot_info_end);
        }

        start += (PAGE_SIZE - (start & PAGE_MASK)) & PAGE_MASK;
        size -= start - b_area.start_addr();
        if size == 0 {
            continue;
        }

        unsafe { MMAP[len] = MemoryArea::new(start, size) }
        len += 1;
    }

    let mmap = unsafe { &MMAP[0..len] };
    for area in mmap {
        println!("{:x} {:x}", area.start_addr(), area.size())
    }

    let bump_alloc = BumpAlloc::new(mmap);

    *INNER_ALLOC.lock() = Some(bump_alloc);
}

static INNER_ALLOC: Mutex<Option<BumpAlloc>> = Mutex::new(None);

pub struct GlobalAlloc;

impl FrameAlloc for GlobalAlloc {
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
