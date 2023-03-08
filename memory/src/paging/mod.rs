#[cfg(target_arch = "x86_64")]
pub use x86_64::*;

#[cfg(target_arch = "x86_64")]
pub mod x86_64;

pub trait PageTableLevel {}

pub struct Level1;
impl PageTableLevel for Level1 {}

pub struct Level2;
impl PageTableLevel for Level2 {}

pub struct Level3;
impl PageTableLevel for Level3 {}

pub struct Level4;
impl PageTableLevel for Level4 {}
