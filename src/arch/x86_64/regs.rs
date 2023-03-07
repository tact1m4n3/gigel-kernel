use core::arch::asm;

pub const KERNEL_GS_BASE: u64 = 0xC0000102;

pub unsafe fn load_cs(val: u16) {
    asm!(
        "pushq {0}",
        "leaq 0f, {1}",
        "pushq {1}",
        "lretq",
        "0:",
        in(reg) val as u64,
        out(reg) _,
        options(att_syntax)
    );
}

pub unsafe fn load_ss(val: u16) {
    asm!("mov {0:x}, %ss", in(reg) val, options(nomem, nostack, att_syntax));
}

pub unsafe fn load_ds(val: u16) {
    asm!("mov {0:x}, %ds", in(reg) val, options(nomem, nostack, att_syntax));
}

pub unsafe fn load_es(val: u16) {
    asm!("mov {0:x}, %es", in(reg) val, options(nomem, nostack, att_syntax));
}

pub unsafe fn load_tss(val: u16) {
    asm!("ltr {0:x}", in(reg) val, options(nomem, nostack, att_syntax));
}

pub unsafe fn gs_base() -> u64 {
    let ret: u64;
    asm!("mov %gs:0, {0}", out(reg) ret, options(nomem, nostack, att_syntax));
    ret
}

pub unsafe fn read_cr2() -> u64 {
    let val: u64;
    asm!("mov %cr2, {0}", out(reg) val, options(nomem, nostack, att_syntax));
    val
}

pub unsafe fn write_cr2(val: u64) {
    asm!("mov {0}, %cr2", in(reg) val, options(nomem, nostack, att_syntax));
}

pub unsafe fn read_cr3() -> u64 {
    let val: u64;
    asm!("mov %cr3, {0}", out(reg) val, options(nomem, nostack, att_syntax));
    val
}

pub unsafe fn write_cr3(val: u64) {
    asm!("mov {0}, %cr3", in(reg) val, options(nomem, nostack, att_syntax));
}

pub unsafe fn read_msr(reg: u64) -> u64 {
    let low: u64;
    let high: u64;
    asm!(
        "rdmsr",
        in("rcx") reg,
        out("rax") low,
        out("rdx") high,
        options(nomem, nostack, att_syntax)
    );
    low | (high << 32)
}

pub unsafe fn write_msr(reg: u64, val: u64) {
    asm!(
        "wrmsr",
        in("rcx") reg,
        in("rax") val,
        in("rdx") (val >> 32),
        options(nomem, nostack, att_syntax)
    );
}

pub unsafe fn read_tsc() -> u64 {
    let low: u64;
    let high: u64;
    asm!(
        "rdtsc",
        out("rax") low,
        out("rdx") high,
        options(nomem, nostack, att_syntax)
    );
    low | (high << 32)
}
