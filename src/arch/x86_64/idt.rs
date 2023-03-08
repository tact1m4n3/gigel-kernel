use core::{
    arch::asm,
    hint,
    sync::atomic::{AtomicUsize, Ordering},
};

use crate::{
    arch::x86_64::{pic::PIC1, regs},
    println,
};

const PRESENT: u8 = 1 << 7;
const DPL0: u8 = 0 << 5;
const DPL1: u8 = 1 << 5;
const DPL2: u8 = 2 << 5;
const DPL3: u8 = 3 << 5;
const INTERRUPT: u8 = 0xE;

static NEXT_ID: AtomicUsize = AtomicUsize::new(0);
pub static mut IDT: Idt = Idt::new();

pub fn init() {
    let idt = unsafe { &mut IDT };

    if NEXT_ID.fetch_add(1, Ordering::Acquire) == 0 {
        idt.set_handler_fn(0x00, division);
        idt.set_handler_fn(0x01, debug);
        idt.set_handler_fn(0x02, non_maskable);
        idt.set_handler_fn(0x03, breakpoint);
        idt.set_handler_fn(0x04, overflow);
        idt.set_handler_fn(0x05, bound_range);
        idt.set_handler_fn(0x06, invalid_opcode);
        idt.set_handler_fn(0x07, device_not_available);
        idt.set_handler_fn(0x08, double);
        idt.set_handler_fn(0x0A, invalid_tss);
        idt.set_handler_fn(0x0B, segment_not_present);
        idt.set_handler_fn(0x0C, stack_segment);
        idt.set_handler_fn(0x0D, general_protection);
        idt.set_handler_fn(0x0E, page);
        idt.set_handler_fn(0x10, x87_fp);
        idt.set_handler_fn(0x11, alignment_check);
        idt.set_handler_fn(0x12, machine_check);
        idt.set_handler_fn(0x13, simd_fp);
        idt.set_handler_fn(0x14, virtualization);
        idt.set_handler_fn(0x15, control_protection);
        idt.set_handler_fn(0x1C, hypervisor_injection);
        idt.set_handler_fn(0x1D, vmm_communication);
        idt.set_handler_fn(0x1E, security);

        idt.set_handler_fn(0x20, pit);
        idt.set_handler_fn(0x21, keyboard);
        idt.set_handler_fn(0x22, cascade);
        idt.set_handler_fn(0x23, com2);
        idt.set_handler_fn(0x24, com1);
        idt.set_handler_fn(0x25, lpt2);
        idt.set_handler_fn(0x26, floppy_disk);
        idt.set_handler_fn(0x27, lpt1);
        idt.set_handler_fn(0x28, cmos);
        idt.set_handler_fn(0x29, peripheral1);
        idt.set_handler_fn(0x2A, peripheral2);
        idt.set_handler_fn(0x2B, peripheral3);
        idt.set_handler_fn(0x2C, mouse);
        idt.set_handler_fn(0x2D, fpu);
        idt.set_handler_fn(0x2E, primary_ata);
        idt.set_handler_fn(0x2F, secondary_ata);

        idt.set_handler_fn(0x7E, lapic);
        idt.set_handler_fn(0x7F, invalidate_tlb);
    }

    idt.load();
}

pub fn enable_interrupts() {
    unsafe { asm!("sti", options(nomem, nostack, att_syntax)) }
}

pub fn disable_interrupts() {
    unsafe { asm!("cli", options(nomem, nostack, att_syntax)) }
}

#[repr(transparent)]
pub struct Idt {
    entries: [Entry; 256],
}

impl Idt {
    pub const fn new() -> Self {
        Self {
            entries: [Entry::new(); 256],
        }
    }

    pub fn set_handler_fn(&mut self, id: usize, handler: HandlerFn) -> &mut Entry {
        self.entries[id]
            .set_base(handler as usize)
            .set_cs(0x08)
            .set_ist(0)
            .set_flags(PRESENT | DPL0 | INTERRUPT)
    }

    pub fn load(&self) {
        let pointer = IdtPointer {
            limit: 256 * 8 - 1,
            base: self,
        };
        unsafe { asm!("lidt ({0})", in(reg) &pointer, options(att_syntax)) }
    }
}

#[repr(packed)]
struct IdtPointer {
    limit: u16,
    base: *const Idt,
}

#[repr(packed)]
#[derive(Clone, Copy)]
pub struct Entry {
    base_low: u16,
    cs: u16,
    ist: u8,
    flags: u8,
    base_mid: u16,
    base_high: u32,
    _reserved: u32,
}

impl Entry {
    pub const fn new() -> Self {
        Self {
            base_low: 0,
            cs: 0,
            ist: 0,
            flags: 0,
            base_mid: 0,
            base_high: 0,
            _reserved: 0,
        }
    }

    pub fn set_base(&mut self, base: usize) -> &mut Self {
        self.base_low = (base & 0xFFFF) as u16;
        self.base_mid = ((base >> 16) & 0xFFFF) as u16;
        self.base_high = ((base >> 32) & 0xFFFFFFFF) as u32;
        self
    }

    pub fn set_cs(&mut self, cs: u16) -> &mut Self {
        self.cs = cs;
        self
    }

    pub fn set_ist(&mut self, ist: u8) -> &mut Self {
        self.ist = ist;
        self
    }

    pub fn set_flags(&mut self, flags: u8) -> &mut Self {
        self.flags |= flags;
        self
    }
}

#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct InterruptStack {
    pub r15: u64,
    pub r14: u64,
    pub r13: u64,
    pub r12: u64,
    pub r11: u64,
    pub r10: u64,
    pub r9: u64,
    pub r8: u64,
    pub rsi: u64,
    pub rdi: u64,
    pub rbp: u64,
    pub rdx: u64,
    pub rcx: u64,
    pub rbx: u64,
    pub rax: u64,
    pub rip: u64,
    pub cs: u64,
    pub rflags: u64,
    pub rsp: u64,
    pub ss: u64,
}

type HandlerFn = unsafe extern "C" fn();

#[macro_export]
macro_rules! push_registers {
    () => {
        concat!(
            "push %rax\n",
            "push %rbx\n",
            "push %rcx\n",
            "push %rdx\n",
            "push %rbp\n",
            "push %rdi\n",
            "push %rsi\n",
            "push %r8\n",
            "push %r9\n",
            "push %r10\n",
            "push %r11\n",
            "push %r12\n",
            "push %r13\n",
            "push %r14\n",
            "push %r15\n",
        )
    };
}

#[macro_export]
macro_rules! push_registers_and_save_error_code {
    () => {
        concat!(
            "push %rax\n",
            "movq 16(%rsp), %rax\n",
            "push %rbx\n",
            "movq 16(%rsp), %rbx\n",
            "movq %rbx, 24(%rsp)\n",
            "movq 8(%rsp), %rbx\n",
            "movq %rbx, 16(%rsp)\n",
            "addq $8, %rsp\n",
            "push %rcx\n",
            "push %rdx\n",
            "push %rbp\n",
            "push %rdi\n",
            "push %rsi\n",
            "push %r8\n",
            "push %r9\n",
            "push %r10\n",
            "push %r11\n",
            "push %r12\n",
            "push %r13\n",
            "push %r14\n",
            "push %r15\n",
        )
    };
}

#[macro_export]
macro_rules! pop_registers {
    () => {
        concat!(
            "pop %r15\n",
            "pop %r14\n",
            "pop %r13\n",
            "pop %r12\n",
            "pop %r11\n",
            "pop %r10\n",
            "pop %r9\n",
            "pop %r8\n",
            "pop %rsi\n",
            "pop %rdi\n",
            "pop %rbp\n",
            "pop %rdx\n",
            "pop %rcx\n",
            "pop %rbx\n",
            "pop %rax\n",
        )
    };
}

#[macro_export]
macro_rules! swapgs {
    () => {
        concat!("cmpq $0x08, 128(%rsp)\n", "je 1f\n", "swapgs\n", "1:\n")
    };
}

#[macro_export]
macro_rules! exception {
    ($name:ident, |$stack:ident| $code:block) => {
        #[naked]
        pub unsafe extern "C" fn $name() {
            unsafe extern "C" fn handler($stack: &$crate::arch::x86_64::idt::InterruptStack) {
                $code
            }

            core::arch::asm!(
                concat!(
                    $crate::push_registers!(),
                    $crate::swapgs!(),
                    "movq %rsp, %rsi\n",
                    "callq {handler}\n",
                    $crate::swapgs!(),
                    $crate::pop_registers!(),
                    "iretq",
                ),
                handler = sym handler,
                options(noreturn, att_syntax),
            );
        }
    };
}

#[macro_export]
macro_rules! exception_with_error {
    ($name:ident, |$err_code:ident, $stack:ident| $code:block) => {
        #[naked]
        pub unsafe extern "C" fn $name() {
            unsafe extern "C" fn handler($err_code: u64, $stack: &$crate::arch::x86_64::idt::InterruptStack) {
                $code
            }

            core::arch::asm!(
                concat!(
                    $crate::push_registers_and_save_error_code!(),
                    $crate::swapgs!(),
                    "movq %rax, %rdi\n",
                    "movq %rsp, %rsi\n",
                    "callq {handler}\n",
                    $crate::swapgs!(),
                    $crate::pop_registers!(),
                    "iretq",
                ),
                handler = sym handler,
                options(noreturn, att_syntax),
            );
        }
    };
}

#[macro_export]
macro_rules! interrupt {
    ($name:ident, |$stack:ident| $code:block) => {
        #[naked]
        pub unsafe extern "C" fn $name() {
            unsafe extern "C" fn handler($stack: &$crate::arch::x86_64::idt::InterruptStack) {
                $code
            }

            core::arch::asm!(
                concat!(
                    $crate::push_registers!(),
                    $crate::swapgs!(),
                    "movq %rsp, %rdi\n",
                    "callq {handler}\n",
                    $crate::swapgs!(),
                    $crate::pop_registers!(),
                    "iretq",
                ),
                handler = sym handler,
                options(noreturn, att_syntax),
            );
        }
    };
}

exception!(division, |_stack| {
    panic!("division by 0");
});

exception!(debug, |_stack| {
    panic!("debug");
});

exception!(non_maskable, |_stack| {
    panic!("non maskable interrupt");
});

exception!(breakpoint, |_stack| {
    panic!("breakpoint");
});

exception!(overflow, |_stack| {
    panic!("overflow");
});

exception!(bound_range, |_stack| {
    panic!("bound range exceeded");
});

exception!(invalid_opcode, |_stack| {
    panic!("invalid opcode");
});

exception!(device_not_available, |_stack| {
    panic!("device not available");
});

exception_with_error!(double, |_err_code, _stack| {
    panic!("double fault");
});

exception_with_error!(invalid_tss, |_err_code, _stack| {
    panic!("invalid tss");
});

exception_with_error!(segment_not_present, |_err_code, _stack| {
    panic!("segment not present");
});

exception_with_error!(stack_segment, |_err_code, _stack| {
    panic!("stack segment fault");
});

exception_with_error!(general_protection, |_err_code, _stack| {
    panic!("general protection fault");
});

exception_with_error!(page, |_err_code, _stack| {
    println!("page fault at {:x}", regs::read_cr2());
    panic!("page fault");
});

exception!(x87_fp, |_stack| {
    panic!("x87 floating point exception");
});

exception_with_error!(alignment_check, |_err_code, _stack| {
    panic!("alignment check");
});

exception!(machine_check, |_stack| {
    panic!("machine check");
});

exception!(simd_fp, |_stack| {
    panic!("simd floating point exception");
});

exception!(virtualization, |_stack| {
    panic!("virtualization exception");
});

exception_with_error!(control_protection, |_err_code, _stack| {
    panic!("control protection exception");
});

exception!(hypervisor_injection, |_stack| {
    panic!("hypervisor injection exception");
});

exception_with_error!(vmm_communication, |_err_code, _stack| {
    panic!("vmm communication exception");
});

exception_with_error!(security, |_err_code, _stack| {
    panic!("security exception");
});

interrupt!(pit, |_stack| {
    // println!("pit interrupt");
    PIC1.lock().send_eoi();
});

interrupt!(keyboard, |_stack| {
    println!("keyboard interrupt");
});

interrupt!(cascade, |_stack| {
    println!("cascade interrupt");
});

interrupt!(com2, |_stack| {
    println!("COM2 interrupt");
});

interrupt!(com1, |_stack| {
    println!("COM1 interrupt");
});

interrupt!(lpt2, |_stack| {
    println!("LPT2 interrupt");
});

interrupt!(floppy_disk, |_stack| {
    println!("floppy disk interrupt");
});

interrupt!(lpt1, |_stack| {
    println!("LPT1/spurious interrupt");
});

interrupt!(cmos, |_stack| {
    println!("CMOS interrupt");
});

interrupt!(peripheral1, |_stack| {
    println!("peripheral interrupt");
});

interrupt!(peripheral2, |_stack| {
    println!("peripheral interrupt");
});

interrupt!(peripheral3, |_stack| {
    println!("peripheral interrupt");
});

interrupt!(mouse, |_stack| {
    println!("mouse interrupt");
});

interrupt!(fpu, |_stack| {
    println!("FPU interrupt");
});

interrupt!(primary_ata, |_stack| {
    println!("primary ATA interrupt");
});

interrupt!(secondary_ata, |_stack| {
    println!("secondary ATA interrupt");
});

interrupt!(lapic, |_stack| {
    // println!("lapic interrupt");
});

interrupt!(invalidate_tlb, |_stack| {
    regs::write_cr3(regs::read_cr3());
});

pub fn switch_context(stack: *mut u64) -> ! {
    unsafe {
        asm!(concat!("mov {0}, %rsp\n", swapgs!(), pop_registers!(), "iretq\n"), in(reg) stack, options(att_syntax));
        hint::unreachable_unchecked();
    }
}
