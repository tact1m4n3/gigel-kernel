use core::{
    arch::asm,
    marker::PhantomData,
    ops::{BitAnd, BitOr},
};

use crate::{AllocError, Frame, FrameAlloc, Level1, Level2, Level3, Level4, PageTableLevel};

pub const PAGE_SHIFT: usize = 12;
pub const PAGE_MASK: usize = 0xFFF;
pub const PAGE_SIZE: usize = 0x1000;

pub struct PageMapper<F> {
    root: &'static mut PageTable<Level4>,
    allocator: F,
}

impl<F: FrameAlloc> PageMapper<F> {
    pub fn new(mut allocator: F) -> PageMapper<F> {
        let root =
            unsafe { &mut *(allocator.alloc(1).unwrap().addr() as *mut PageTable<_>) }.init();
        PageMapper { root, allocator }
    }

    pub fn current(allocator: F) -> PageMapper<F> {
        let root = unsafe { &mut *(read_cr3() as *mut PageTable<_>) }.init();
        PageMapper { root, allocator }
    }

    pub fn make_current(&self) {
        unsafe { write_cr3(self.root as *const _ as usize) }
    }

    pub fn get_page(&self, addr: usize) -> Option<&Page> {
        let p4 = &*self.root;
        if !(p4.get(addr).flags() & PageFlags::PRESENT) {
            return None;
        }
        let p3 = p4.next(addr);
        if !(p3.get(addr).flags() & PageFlags::PRESENT) {
            return None;
        }
        let p2 = p3.next(addr);
        if !(p2.get(addr).flags() & PageFlags::PRESENT) {
            return None;
        }
        let p1 = p2.next(addr);
        if !(p1.get(addr).flags() & PageFlags::PRESENT) {
            return None;
        }
        Some(p1.get(addr))
    }

    pub fn get_page_mut(&mut self, addr: usize) -> Option<&mut Page> {
        let p4 = &mut *self.root;
        if !(p4.get(addr).flags() & PageFlags::PRESENT) {
            return None;
        }
        let p3 = p4.next_mut(addr);
        if !(p3.get(addr).flags() & PageFlags::PRESENT) {
            return None;
        }
        let p2 = p3.next_mut(addr);
        if !(p2.get(addr).flags() & PageFlags::PRESENT) {
            return None;
        }
        let p1 = p2.next_mut(addr);
        if !(p1.get(addr).flags() & PageFlags::PRESENT) {
            return None;
        }
        Some(p1.get_mut(addr))
    }

    pub fn touch_page(&mut self, addr: usize, flags: PageFlags) -> Result<&mut Page, AllocError> {
        let p4 = &mut *self.root;

        let p3 = if !(p4.get(addr).flags() & PageFlags::PRESENT) {
            let frame = self.allocator.alloc(1)?.first();
            p4.get_mut(addr)
                .set_frame(frame)
                .set_flags(PageFlags::PRESENT | flags);
            unsafe { &mut *(frame.addr() as *mut PageTable<_>) }.init()
        } else {
            p4.next_mut(addr)
        };

        let p2 = if !(p3.get(addr).flags() & PageFlags::PRESENT) {
            let frame = self.allocator.alloc(1)?.first();
            p3.get_mut(addr)
                .set_frame(frame)
                .set_flags(PageFlags::PRESENT | flags);
            unsafe { &mut *(frame.addr() as *mut PageTable<_>) }.init()
        } else {
            p3.next_mut(addr)
        };

        let p1 = if !(p2.get(addr).flags() & PageFlags::PRESENT) {
            let frame = self.allocator.alloc(1)?.first();
            p2.get_mut(addr)
                .set_frame(frame)
                .set_flags(PageFlags::PRESENT | flags);
            unsafe { &mut *(frame.addr() as *mut PageTable<_>) }.init()
        } else {
            p2.next_mut(addr)
        };

        Ok(p1.get_mut(addr))
    }

    pub fn map_page(
        &mut self,
        addr: usize,
        frame: Frame,
        flags: PageFlags,
    ) -> Result<&mut Page, AllocError> {
        Ok(self
            .touch_page(addr, flags)?
            .set_frame(frame)
            .set_flags(flags))
    }

    pub fn clear_page(&mut self, addr: usize) {
        if let Some(page) = self.get_page_mut(addr) {
            *page = Page::new();
        }
    }
}

#[repr(transparent)]
pub struct PageTable<L: PageTableLevel> {
    entries: [Page; 512],
    level: PhantomData<L>,
}

impl<L: PageTableLevel> PageTable<L> {
    #[inline]
    pub fn init(&mut self) -> &mut Self {
        self.entries.fill(Page::new());
        self
    }
}

impl PageTable<Level4> {
    #[inline]
    pub fn get(&self, addr: usize) -> &Page {
        &self.entries[(addr >> 39) & 0x1FF]
    }

    #[inline]
    pub fn get_mut(&mut self, addr: usize) -> &mut Page {
        &mut self.entries[(addr >> 39) & 0x1FF]
    }

    #[inline]
    pub fn next(&self, addr: usize) -> &PageTable<Level3> {
        unsafe { &*(self.get(addr).frame().addr() as *const PageTable<_>) }
    }

    #[inline]
    pub fn next_mut(&mut self, addr: usize) -> &mut PageTable<Level3> {
        unsafe { &mut *(self.get(addr).frame().addr() as *mut PageTable<_>) }
    }
}

impl PageTable<Level3> {
    #[inline]
    pub fn get(&self, addr: usize) -> &Page {
        &self.entries[(addr >> 30) & 0x1FF]
    }

    #[inline]
    pub fn get_mut(&mut self, addr: usize) -> &mut Page {
        &mut self.entries[(addr >> 30) & 0x1FF]
    }

    #[inline]
    pub fn next(&self, addr: usize) -> &PageTable<Level2> {
        unsafe { &*(self.get(addr).frame().addr() as *const PageTable<_>) }
    }

    #[inline]
    pub fn next_mut(&mut self, addr: usize) -> &mut PageTable<Level2> {
        unsafe { &mut *(self.get(addr).frame().addr() as *mut PageTable<_>) }
    }
}

impl PageTable<Level2> {
    #[inline]
    pub fn get(&self, addr: usize) -> &Page {
        &self.entries[(addr >> 21) & 0x1FF]
    }

    #[inline]
    pub fn get_mut(&mut self, addr: usize) -> &mut Page {
        &mut self.entries[(addr >> 21) & 0x1FF]
    }

    #[inline]
    pub fn next(&self, addr: usize) -> &PageTable<Level1> {
        unsafe { &*(self.get(addr).frame().addr() as *const PageTable<_>) }
    }

    #[inline]
    pub fn next_mut(&mut self, addr: usize) -> &mut PageTable<Level1> {
        unsafe { &mut *(self.get(addr).frame().addr() as *mut PageTable<_>) }
    }
}

impl PageTable<Level1> {
    #[inline]
    pub fn get(&self, addr: usize) -> &Page {
        &self.entries[(addr >> 12) & 0x1FF]
    }

    #[inline]
    pub fn get_mut(&mut self, addr: usize) -> &mut Page {
        &mut self.entries[(addr >> 12) & 0x1FF]
    }
}

#[derive(Debug, Clone, Copy)]
#[repr(transparent)]
pub struct Page(usize);

impl Page {
    #[inline]
    pub const fn new() -> Self {
        Self(0)
    }

    #[inline]
    pub const fn raw(&self) -> usize {
        self.0
    }

    #[inline]
    pub const fn frame(&self) -> Frame {
        Frame::from_addr(self.0 & !PAGE_MASK)
    }

    #[inline]
    pub const fn flags(&self) -> PageFlags {
        PageFlags::new(self.0 & PAGE_MASK)
    }

    pub fn set_frame(&mut self, frame: Frame) -> &mut Self {
        self.0 &= PAGE_MASK;
        self.0 |= frame.addr();
        self
    }

    pub fn set_flags(&mut self, flags: PageFlags) -> &mut Self {
        self.0 &= !PAGE_MASK;
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

    #[inline]
    const fn new(val: usize) -> Self {
        Self(val)
    }

    #[inline]
    pub const fn raw(self) -> usize {
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

#[inline]
unsafe fn write_cr3(val: usize) {
    asm!("mov {0}, %cr3", in(reg) val, options(att_syntax));
}

#[inline]
unsafe fn read_cr3() -> usize {
    let val: usize;
    asm!(
        "mov %cr3, {0}",
        out(reg) val,
        options(nomem, nostack, att_syntax)
    );
    val
}
