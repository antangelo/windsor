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
    mov esp, offset __kernel_stack
    mov ebp, esp
    jmp kmain
