#![no_std]
#![no_main]
#![allow(dead_code)]
#![feature(const_trait_impl)]
#![feature(naked_functions)]

use core::panic::PanicInfo;

mod arch;
mod sync;

pub fn kernel_main() -> ! {
    arch::enable_interrupts();
    arch::halt();
}

#[panic_handler]
fn _panic(info: &PanicInfo) -> ! {
    println!("{}", info);
    loop {}
}

#[macro_export]
macro_rules! print {
    ($($arg:tt)*) => ($crate::arch::print(format_args!($($arg)*)));
}

#[macro_export]
macro_rules! println {
    () => ($crate::print!("\n"));
    ($($arg:tt)*) => ($crate::print!("{}\n", format_args!($($arg)*)));
}
