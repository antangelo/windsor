.section .text

.global reload_segments

reload_segments:
    push %eax
    ljmpl $0x8, $reload_cs

reload_cs:
    mov $0x10, %eax
    mov %eax, %ds
    mov %eax, %es
    mov %eax, %ss
    mov %eax, %fs
    mov %eax, %gs

    pop %eax
    ret
