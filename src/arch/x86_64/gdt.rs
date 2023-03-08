use core::{
    arch::asm,
    mem,
    sync::atomic::{AtomicUsize, Ordering},
};

use super::regs;

const MAX_GDTS: usize = 16;

const CODE: u32 = 0x18 << 8;
const DATA: u32 = 0x12 << 8;
const TSS: u32 = 0x09 << 8;
const DPL0: u32 = 0 << 13;
const DPL1: u32 = 1 << 13;
const DPL2: u32 = 2 << 13;
const DPL3: u32 = 3 << 13;
const PRESENT: u32 = 1 << 15;
const LONG: u32 = 1 << 21;

static NEXT_ID: AtomicUsize = AtomicUsize::new(0);
static mut GDTS: [(Gdt, Tss); 16] = [(Gdt::new(), Tss::new()); 16];

pub fn init() {
    assert!(NEXT_ID.load(Ordering::Relaxed) < 16, "too many gdts");

    let (gdt, tss) = unsafe { &mut GDTS[NEXT_ID.fetch_add(1, Ordering::Acquire)] };

    gdt.add_segment(PRESENT | DPL0 | CODE | LONG);
    gdt.add_segment(PRESENT | DPL3 | CODE | LONG);
    gdt.add_segment(PRESENT | DPL3 | DATA);

    let tss_base = tss as *const _ as usize;
    let tss_limit = mem::size_of::<Tss>() - 1;
    gdt.add_segment(PRESENT | TSS)
        .encode_tss_low(tss_base, tss_limit);
    gdt.add_segment(0).encode_tss_high(tss_base, tss_limit);

    gdt.load();

    unsafe {
        regs::load_cs(0x08);
        regs::load_ss(0x00);
        regs::load_ds(0x00);
        regs::load_es(0x00);
        regs::load_tss(0x20);
    }
}

pub fn set_kernel_stack(stack: usize) {
    let id = 0;
    let (_, tss) = unsafe { &mut GDTS[id] };
    tss.set_kernel_stack(stack as u64);
}

#[repr(C)]
#[derive(Clone, Copy)]
struct Gdt {
    entries: [Entry; 256],
    length: usize,
}

impl Gdt {
    pub const fn new() -> Self {
        Self {
            entries: [Entry::new(); 256],
            length: 1,
        }
    }

    pub fn add_segment(&mut self, flags: u32) -> &mut Entry {
        let entry = self.entries[self.length].set_addr(0).set_flags(flags);
        self.length += 1;
        entry
    }

    pub fn load(&self) {
        let pointer = GdtPointer {
            limit: (self.length as u16) * 8 - 1,
            base: &self.entries,
        };
        unsafe { asm!("lgdt ({0})", in(reg) &pointer, options(att_syntax)) }
    }
}

#[repr(packed)]
struct GdtPointer {
    limit: u16,
    base: *const [Entry],
}

#[repr(packed)]
#[derive(Clone, Copy)]
struct Tss {
    _reserved0: u32,
    rsp0: u64,
    rsp1: u64,
    rsp2: u64,
    _reserved1: u64,
    ist1: u64,
    ist2: u64,
    ist3: u64,
    ist4: u64,
    ist5: u64,
    ist6: u64,
    ist7: u64,
    _reserved2: u64,
    _reserved3: u16,
    iopb: u16,
}

impl Tss {
    pub const fn new() -> Self {
        Self {
            _reserved0: 0,
            rsp0: 0,
            rsp1: 0,
            rsp2: 0,
            _reserved1: 0,
            ist1: 0,
            ist2: 0,
            ist3: 0,
            ist4: 0,
            ist5: 0,
            ist6: 0,
            ist7: 0,
            _reserved2: 0,
            _reserved3: 0,
            iopb: 0,
        }
    }

    pub fn set_kernel_stack(&mut self, rsp: u64) {
        self.rsp0 = rsp;
    }
}

#[repr(packed)]
#[derive(Clone, Copy)]
struct Entry {
    low: u32,
    high: u32,
}

impl Entry {
    pub const fn new() -> Self {
        Self { low: 0, high: 0 }
    }

    pub fn set_addr(&mut self, addr: u32) -> &mut Self {
        self.low = addr;
        self
    }

    pub fn set_flags(&mut self, flags: u32) -> &mut Self {
        self.high = flags;
        self
    }

    pub fn encode_tss_low(&mut self, base: usize, limit: usize) -> &mut Self {
        self.high |= ((base >> 16) & 0xFF) as u32;
        self.high |= (((base >> 24) & 0xFF) << 24) as u32;
        self.high |= (((limit >> 16) & 0xF) << 16) as u32;
        self.low |= (limit & 0xFFFF) as u32;
        self.low |= ((base & 0xFFFF) << 16) as u32;
        self
    }

    pub fn encode_tss_high(&mut self, base: usize, _limit: usize) -> &mut Self {
        self.low = (base >> 32) as u32;
        self
    }
}
