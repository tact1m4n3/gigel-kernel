use core::arch::{asm, global_asm};

pub mod debug;
pub mod gdt;
pub mod idt;
pub mod io;
pub mod lapic;
pub mod memory;
pub mod pic;
pub mod regs;
pub mod serial;

global_asm!(include_str!("boot.s"), options(att_syntax));

#[no_mangle]
pub extern "C" fn kernel_entry(magic: u64, info: *const u8) -> ! {
    let boot_info = multiboot2::init(magic, info).expect("unsupported bootloader");

    serial::init();
    gdt::init();
    pic::init();
    idt::init();

    memory::init(boot_info);

    crate::kernel_main();
}

pub fn halt() -> ! {
    loop {
        unsafe { asm!("hlt", options(nomem, nostack, att_syntax)) }
    }
}
