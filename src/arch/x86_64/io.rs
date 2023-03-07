use core::{arch::asm, marker::PhantomData, ops::Add, ptr};

pub unsafe fn inb(port: u16) -> u8 {
    let ret: u8;
    asm!("inb %dx, %al", in("dx") port, out("al") ret, options(nomem, nostack, att_syntax));
    ret
}

pub unsafe fn outb(port: u16, val: u8) {
    asm!("outb %al, %dx", in("al") val, in("dx") port, options(nomem, nostack, att_syntax));
}

pub trait PortInOut {
    unsafe fn port_in(port: u16) -> Self;
    unsafe fn port_out(port: u16, val: Self);
}

impl PortInOut for u8 {
    unsafe fn port_in(port: u16) -> Self {
        inb(port)
    }

    unsafe fn port_out(port: u16, val: Self) {
        outb(port, val)
    }
}

#[derive(Clone, Copy)]
pub struct Pio<T: PortInOut> {
    port: u16,
    phantom: PhantomData<T>,
}

impl<T: PortInOut> Pio<T> {
    pub const fn new(port: u16) -> Self {
        Self {
            port,
            phantom: PhantomData {},
        }
    }

    pub fn read(&self) -> T {
        unsafe { T::port_in(self.port) }
    }

    pub fn write(&self, val: T) {
        unsafe { T::port_out(self.port, val) }
    }
}

impl<T: PortInOut> const Add<u16> for Pio<T> {
    type Output = Pio<T>;

    fn add(self, rhs: u16) -> Self::Output {
        Self {
            port: self.port + rhs,
            phantom: PhantomData {},
        }
    }
}

#[derive(Clone, Copy)]
pub struct Mmio<T> {
    addr: usize,
    phantom: PhantomData<T>,
}

impl<T> Mmio<T> {
    pub const fn new(addr: usize) -> Self {
        Self {
            addr,
            phantom: PhantomData {},
        }
    }

    pub fn read(&self) -> T {
        unsafe { ptr::read_volatile(self.addr as *mut T) }
    }

    pub fn write(&self, val: T) {
        unsafe { ptr::write_volatile(self.addr as *mut T, val) }
    }
}

impl<T> const Add<usize> for Mmio<T> {
    type Output = Mmio<T>;

    fn add(self, rhs: usize) -> Self::Output {
        Self {
            addr: self.addr + rhs,
            phantom: PhantomData {},
        }
    }
}
