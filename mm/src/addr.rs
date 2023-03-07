use core::fmt::{Display, LowerHex, UpperHex};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(transparent)]
pub struct PhysAddr(usize);

impl PhysAddr {
    pub const fn new(addr: usize) -> Self {
        Self(addr)
    }

    pub const fn zero() -> Self {
        Self(0)
    }

    pub const fn inner(self) -> usize {
        self.0
    }

    pub fn from_ptr<T>(ptr: *const T) -> Self {
        Self(ptr as usize)
    }

    pub const fn as_ptr<T>(self) -> *const T {
        self.0 as *const T
    }

    pub const fn as_ptr_mut<T>(self) -> *mut T {
        self.0 as *mut T
    }

    pub const fn align(self, val: usize) -> Self {
        Self(self.0 & !(val - 1))
    }

    pub const fn align_up(self, val: usize) -> Self {
        Self((self.0 & !(val - 1)) + val)
    }
}

impl From<PhysAddr> for usize {
    fn from(value: PhysAddr) -> Self {
        value.0
    }
}

impl Display for PhysAddr {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl LowerHex for PhysAddr {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "{:x}", self.0)
    }
}

impl UpperHex for PhysAddr {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "{:X}", self.0)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(transparent)]
pub struct VirtAddr(usize);

impl VirtAddr {
    pub const fn new(addr: usize) -> Self {
        Self(addr)
    }

    pub const fn zero() -> Self {
        Self(0)
    }

    pub const fn inner(self) -> usize {
        self.0
    }

    pub fn from_ptr<T>(ptr: *const T) -> Self {
        Self(ptr as usize)
    }

    pub const fn as_ptr<T>(&self) -> *const T {
        self.0 as *const T
    }

    pub const fn as_ptr_mut<T>(&self) -> *mut T {
        self.0 as *mut T
    }

    pub const fn align(self, val: usize) -> Self {
        Self(self.0 & !(val - 1))
    }

    pub const fn align_up(self, val: usize) -> Self {
        Self((self.0 & !(val - 1)) + val)
    }
}

impl From<VirtAddr> for usize {
    fn from(value: VirtAddr) -> Self {
        value.0
    }
}

impl Display for VirtAddr {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl LowerHex for VirtAddr {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "{:x}", self.0)
    }
}

impl UpperHex for VirtAddr {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "{:X}", self.0)
    }
}
