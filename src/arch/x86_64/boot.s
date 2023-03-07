.section .multiboot

.set MBOOT2_MAGIC, 0xe85250d6
.set MBOOT2_ARCH, 0
.set MBOOT2_LENGTH, (mboot2_header_end - mboot2_header)
.set MBOOT2_CHECKSUM, -(MBOOT2_MAGIC + MBOOT2_ARCH + MBOOT2_LENGTH)

.align 8
mboot2_header:
.long MBOOT2_MAGIC
.long MBOOT2_ARCH
.long MBOOT2_LENGTH
.long MBOOT2_CHECKSUM

.word 0
.word 0
.long 8
mboot2_header_end:

.section .bss
.align 16
.skip 8 * 4096
stack_top:

.section .text
.code32

.global start
start:
    cli

    mov $stack_top, %esp

    pushl $0
    pushl %eax
    pushl $0
    pushl %ebx

    mov $PML4, %ebx
    mov %ebx, %cr3

    mov $3, %ebx
    or $PDPT, %ebx
    movl %ebx, (PML4)

    mov $3, %ebx
    or $PDT, %ebx
    movl %ebx, (PDPT)

    mov $PDT, %ebx
    mov $512, %ecx
    mov $0x87, %edx
.set_entry:
    mov %edx, (%ebx)
    add $0x200000, %edx
    add $8, %ebx
    loop .set_entry

    mov %cr4, %eax
    or $0xA0, %eax
    mov %eax, %cr4

    mov $0xC0000080, %ecx
    rdmsr
    or $256, %eax
    wrmsr

    mov %cr0, %eax
    or $0x80000000, %eax
    mov %eax, %cr0

    lgdt gdt64_ptr
    ljmp $0x08, $realm64

.code64

.extern kernel_entry
realm64:
    mov $0x00, %ax
    mov %ax, %ss
    mov %ax, %ds
    mov %ax, %es

    pop %rsi
    pop %rdi

    call kernel_entry

.halt:
    cli
    hlt
    jmp .halt

.section .rodata
.align 8
gdt64:
.quad 0
.quad (1 << 43) | (1 << 44) | (1 << 47) | (1 << 53)
gdt64_ptr:
.word gdt64_ptr - gdt64
.quad gdt64

.section .bss
.align 4096
PML4:
.skip 4096
PDPT:
.skip 4096
PDT:
.skip 4096
