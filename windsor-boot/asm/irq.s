.intel_syntax noprefix

.section .text

.global reload_segments

reload_segments:
    push eax
    jmp 0x8:reload_cs

reload_cs:
    mov eax, offset 0x10
    mov ds, eax
    mov es, eax
    mov ss, eax
    mov fs, eax
    mov gs, eax

    pop eax
    ret
