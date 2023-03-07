use core::sync::atomic::{AtomicU64, Ordering};

use super::io::{self, Mmio};
use super::regs;

pub static TSC_MHZ: AtomicU64 = AtomicU64::new(0);

pub fn init() {
    if TSC_MHZ.load(Ordering::Relaxed) == 0 {
        TSC_MHZ.store(unsafe { measure_tsc_mhz() }, Ordering::Release);
    }
}

pub struct LocalApic {
    id: Mmio<u32>,
    eoi: Mmio<u32>,
    svr: Mmio<u32>,
    icr_lo: Mmio<u32>,
    icr_hi: Mmio<u32>,
    timer: Mmio<u32>,
    ticr: Mmio<u32>,
    tccr: Mmio<u32>,
    tdcr: Mmio<u32>,
}

impl LocalApic {
    pub const fn new(base: Mmio<u32>) -> Self {
        Self {
            id: base + 0x020,
            eoi: base + 0x0B0,
            svr: base + 0x0F0,
            icr_lo: base + 0x300,
            icr_hi: base + 0x310,
            timer: base + 0x320,
            ticr: base + 0x380,
            tccr: base + 0x390,
            tdcr: base + 0x3E0,
        }
    }

    pub fn init(&self) {
        self.svr.write(0x127);
        self.timer.write(0x7E);
        self.tdcr.write(0x3);
        self.ticr.write(u32::MAX);
        unsafe { tsc_delay(100000) }
        self.timer.write(0x10000);

        let freq = u32::MAX - self.tccr.read();

        self.timer.write(0x7E | 0x20000);
        self.tdcr.write(0x3);
        self.ticr.write(freq);
    }

    pub fn wake(&self, id: usize, addr: usize) {
        self.icr_hi.write((id as u32) << 24);
        self.icr_lo.write(0xC500);

        unsafe { tsc_delay(200) }

        self.icr_hi.write((id as u32) << 24);
        self.icr_lo.write(0x8500);

        unsafe { tsc_delay(10000) }

        for _ in 0..2 {
            self.icr_hi.write((id as u32) << 24);
            self.icr_lo.write(0x600 | (addr >> 12) as u32);

            unsafe { tsc_delay(200) }
        }
    }

    pub fn send_ipi(&self, num: usize) {
        self.icr_lo.write(0xC0000 | (num as u32));
        while self.icr_lo.read() & (1 << 12) != 0 {}
    }

    pub fn send_eoi(&self) {
        self.eoi.write(0);
    }
}

unsafe fn measure_tsc_mhz() -> u64 {
    io::outb(0x61, (io::inb(0x61) & 0xDD) | 0x01);

    let freq = 1193180 / 100;
    io::outb(0x43, 0xB2);
    io::outb(0x42, (freq & 0xFF) as u8);
    io::outb(0x42, ((freq >> 8) & 0xFF) as u8);

    io::outb(0x61, io::inb(0x61) & 0xDE);
    io::outb(0x61, io::inb(0x61) | 0x01);

    let start = regs::read_tsc();
    if (io::inb(0x61) & 0x20) != 0 {
        while (io::inb(0x61) & 0x20) != 0 {}
    } else {
        while (io::inb(0x61) & 0x20) == 0 {}
    }

    (regs::read_tsc() - start) / 10000
}

unsafe fn tsc_delay(msecs: u64) {
    let start = regs::read_tsc();
    let freq = TSC_MHZ.load(Ordering::Relaxed);
    while regs::read_tsc() < start + msecs * freq {}
}
