use mm::{BumpAlloc, MemoryArea};

use super::multiboot::{AreaType, MemoryMapTag, MultibootInfo, TagType};

static mut MMAP: [MemoryArea; 512] = [MemoryArea::new(0, 0); 512];

pub fn init(boot_info: &MultibootInfo) {
    let mmap_tag: &MemoryMapTag = boot_info.find_tag(TagType::Mmap).unwrap();

    let mut len = 0;
    for b_area in mmap_tag.areas() {
        if b_area.typ() == AreaType::Available {
            unsafe { MMAP[len] = MemoryArea::new(b_area.start(), b_area.size()) }
            len += 1;
        }
    }

    let mmap = unsafe { &MMAP[0..len] };
    let _bump_alloc = BumpAlloc::new(mmap);
}
