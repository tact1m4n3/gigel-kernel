#[cfg(x86_64)]
pub use x86_64::{
    debug::print,
    gdt::set_kernel_stack,
    halt,
    idt::{disable_interrupts, enable_interrupts, switch_context},
};

#[cfg(x86_64)]
pub mod x86_64;
