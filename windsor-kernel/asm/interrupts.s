.global kenter

.section .text

kenter:
    // Zero BSS
    xor %eax, %eax
    mov $__start_bss_ram, %edi
    mov $__bss_size, %ecx
    shr $2, %ecx
    rep stosl

    // Start kernel
    mov $__kernel_stack, %esp
    mov %esp, %ebp
    jmp kmain
