use core::{
    marker::PhantomData,
    ops::{BitAnd, BitOr},
};

use crate::{
    AllocError, FrameAlloc, Level1, Level2, Level3, Level4, PageTableLevel, PhysAddr, VirtAddr,
};

pub const PAGE_SIZE: usize = 0x1000;

pub fn alloc_kernel_page_directory() -> &'static mut PageTable<Level4> {
    static mut P4: PageTable<Level4> = PageTable::new();
    static mut P3: PageTable<Level3> = PageTable::new();
    static mut P2: PageTable<Level2> = PageTable::new();
    static mut P1: PageTable<Level1> = PageTable::new();

    unsafe {
        P4.get_mut(VirtAddr::zero())
            .set_addr(PhysAddr::from_ptr(&P3))
            .set_flags(PageFlags::PRESENT | PageFlags::WRITE);
        P3.get_mut(VirtAddr::zero())
            .set_addr(PhysAddr::from_ptr(&P2))
            .set_flags(PageFlags::PRESENT | PageFlags::WRITE);
        P2.get_mut(VirtAddr::zero())
            .set_addr(PhysAddr::from_ptr(&P1))
            .set_flags(PageFlags::PRESENT | PageFlags::WRITE);
    }

    unsafe { &mut P4 }
}

pub struct PageMapper<'a, F: FrameAlloc> {
    allocator: &'a mut F,
    root: &'a mut PageTable<Level4>,
}

impl<'a, F: FrameAlloc> PageMapper<'_, F> {
    pub fn new(allocator: &'a mut F, root: &'a mut PageTable<Level4>) -> PageMapper<'a, F> {
        PageMapper { root, allocator }
    }

    pub fn get_page(&self, virt_addr: VirtAddr) -> Option<&Page> {
        let p4 = &*self.root;
        if p4.get(virt_addr).flags() & PageFlags::PRESENT {
            return None;
        }
        let p3 = p4.next(virt_addr);
        if p3.get(virt_addr).flags() & PageFlags::PRESENT {
            return None;
        }
        let p2 = p3.next(virt_addr);
        if p2.get(virt_addr).flags() & PageFlags::PRESENT {
            return None;
        }
        let p1 = p2.next(virt_addr);
        if p1.get(virt_addr).flags() & PageFlags::PRESENT {
            return None;
        }
        Some(p1.get(virt_addr))
    }

    pub fn get_page_mut(&mut self, virt_addr: VirtAddr) -> Option<&mut Page> {
        let p4 = &mut *self.root;
        if p4.get(virt_addr).flags() & PageFlags::PRESENT {
            return None;
        }
        let p3 = p4.next_mut(virt_addr);
        if p3.get(virt_addr).flags() & PageFlags::PRESENT {
            return None;
        }
        let p2 = p3.next_mut(virt_addr);
        if p2.get(virt_addr).flags() & PageFlags::PRESENT {
            return None;
        }
        let p1 = p2.next_mut(virt_addr);
        if p1.get(virt_addr).flags() & PageFlags::PRESENT {
            return None;
        }
        Some(p1.get_mut(virt_addr))
    }

    pub fn touch_page(
        &mut self,
        virt_addr: VirtAddr,
        flags: PageFlags,
    ) -> Result<&mut Page, AllocError> {
        let p4 = &mut *self.root;

        let p3 = if p4.get(virt_addr).flags() & PageFlags::PRESENT {
            let frame = self.allocator.alloc(1)?;
            p4.get_mut(virt_addr)
                .set_addr(frame)
                .set_flags(PageFlags::PRESENT | flags);
            unsafe { &mut *frame.as_ptr_mut::<PageTable<_>>() }.init()
        } else {
            p4.next_mut(virt_addr)
        };

        let p2 = if p3.get(virt_addr).flags() & PageFlags::PRESENT {
            let frame = self.allocator.alloc(1)?;
            p3.get_mut(virt_addr)
                .set_addr(frame)
                .set_flags(PageFlags::PRESENT | flags);
            unsafe { &mut *frame.as_ptr_mut::<PageTable<_>>() }.init()
        } else {
            p3.next_mut(virt_addr)
        };

        let p1 = if p2.get(virt_addr).flags() & PageFlags::PRESENT {
            let frame = self.allocator.alloc(1)?;
            p2.get_mut(virt_addr)
                .set_addr(frame)
                .set_flags(PageFlags::PRESENT | flags);
            unsafe { &mut *frame.as_ptr_mut::<PageTable<_>>() }.init()
        } else {
            p2.next_mut(virt_addr)
        };

        Ok(p1.get_mut(virt_addr))
    }

    pub fn map_page(
        &mut self,
        virt_addr: VirtAddr,
        phys_addr: PhysAddr,
        flags: PageFlags,
    ) -> Result<&mut Page, AllocError> {
        Ok(self
            .touch_page(virt_addr, flags)?
            .set_addr(phys_addr)
            .set_flags(flags))
    }

    pub fn clear_page(&mut self, virt_addr: VirtAddr) {
        if let Some(page) = self.get_page_mut(virt_addr) {
            *page = Page::new();
        }
    }
}

#[repr(C, align(4096))]
pub struct PageTable<L: PageTableLevel> {
    entries: [Page; 512],
    level: PhantomData<L>,
}

impl<L: PageTableLevel> PageTable<L> {
    pub const fn new() -> Self {
        Self {
            entries: [Page::new(); 512],
            level: PhantomData {},
        }
    }

    pub fn init(&mut self) -> &mut Self {
        self.entries.fill(Page::new());
        self
    }
}

impl PageTable<Level4> {
    pub fn get(&self, addr: VirtAddr) -> &Page {
        &self.entries[(usize::from(addr) >> 39) & 0x1FF]
    }

    pub fn get_mut(&mut self, addr: VirtAddr) -> &mut Page {
        &mut self.entries[(usize::from(addr) >> 39) & 0x1FF]
    }

    pub fn next(&self, addr: VirtAddr) -> &PageTable<Level3> {
        unsafe { &*self.get(addr).addr().as_ptr() }
    }

    pub fn next_mut(&mut self, addr: VirtAddr) -> &mut PageTable<Level3> {
        unsafe { &mut *self.get(addr).addr().as_ptr_mut() }
    }
}

impl PageTable<Level3> {
    pub fn get(&self, addr: VirtAddr) -> &Page {
        &self.entries[(usize::from(addr) >> 30) & 0x1FF]
    }

    pub fn get_mut(&mut self, addr: VirtAddr) -> &mut Page {
        &mut self.entries[(usize::from(addr) >> 30) & 0x1FF]
    }

    pub fn next(&self, addr: VirtAddr) -> &PageTable<Level2> {
        unsafe { &*self.get(addr).addr().as_ptr() }
    }

    pub fn next_mut(&mut self, addr: VirtAddr) -> &mut PageTable<Level2> {
        unsafe { &mut *self.get(addr).addr().as_ptr_mut() }
    }
}

impl PageTable<Level2> {
    pub fn get(&self, addr: VirtAddr) -> &Page {
        &self.entries[(usize::from(addr) >> 21) & 0x1FF]
    }

    pub fn get_mut(&mut self, addr: VirtAddr) -> &mut Page {
        &mut self.entries[(usize::from(addr) >> 21) & 0x1FF]
    }

    pub fn next(&self, addr: VirtAddr) -> &PageTable<Level1> {
        unsafe { &*self.get(addr).addr().as_ptr() }
    }

    pub fn next_mut(&mut self, addr: VirtAddr) -> &mut PageTable<Level1> {
        unsafe { &mut *self.get(addr).addr().as_ptr_mut() }
    }
}

impl PageTable<Level1> {
    pub fn get(&self, addr: VirtAddr) -> &Page {
        &self.entries[(usize::from(addr) >> 12) & 0x1FF]
    }

    pub fn get_mut(&mut self, addr: VirtAddr) -> &mut Page {
        &mut self.entries[(usize::from(addr) >> 12) & 0x1FF]
    }
}

#[derive(Debug, Clone, Copy)]
#[repr(transparent)]
pub struct Page(usize);

impl Page {
    pub const fn new() -> Self {
        Self(0)
    }

    pub fn addr(&self) -> PhysAddr {
        PhysAddr::new(self.0 & !0xFFF)
    }

    pub fn flags(&self) -> PageFlags {
        PageFlags::new(self.0 & 0xFFF)
    }

    pub fn set_addr(&mut self, addr: PhysAddr) -> &mut Self {
        self.0 &= 0xFFF;
        self.0 |= addr.align(PAGE_SIZE).inner();
        self
    }

    pub fn set_flags(&mut self, flags: PageFlags) -> &mut Self {
        self.0 &= !0xFFF;
        self.0 |= flags.raw();
        self
    }
}

#[derive(Debug, Clone, Copy)]
#[repr(transparent)]
pub struct PageFlags(usize);

impl PageFlags {
    pub const PRESENT: PageFlags = Self::new(1 << 0);
    pub const WRITE: PageFlags = Self::new(1 << 1);
    pub const USER: PageFlags = Self::new(1 << 2);

    const fn new(val: usize) -> Self {
        Self(val)
    }

    pub fn raw(self) -> usize {
        self.0
    }
}

impl BitAnd for PageFlags {
    type Output = bool;

    fn bitand(self, rhs: Self) -> Self::Output {
        (self.0 & rhs.0) != 0
    }
}

impl BitOr for PageFlags {
    type Output = Self;

    fn bitor(self, rhs: Self) -> Self::Output {
        Self(self.0 | rhs.0)
    }
}
