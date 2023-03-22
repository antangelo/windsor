.intel_syntax noprefix
.global kenter

.section .text

kenter:
    // Zero bss
    xor eax, eax
    mov edi, offset __start_bss_ram
    mov ecx, offset __bss_size
    shr ecx, 2
    rep stosw

    // Start kernel
    mov esp, offset 0x490000
    mov ebp, esp
    jmp kmain

.macro make_irq ent
    cli
    pusha
    pushf

    call irq_\ent
    mov al, 0x20
    outb 0x20, al

    popf
    popa
    iret
.endm

.global irq_entry_0
.global irq_0
irq_entry_0:
    make_irq 0

.global irq_entry_8
.global irq_8
irq_entry_8:
    make_irq 8
