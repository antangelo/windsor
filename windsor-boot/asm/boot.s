.intel_syntax noprefix
.global boot_startup
.global mcpx_enter

.section .low_rom, "ax"

mcpx_enter:
    jmp boot_startup

boot_startup:
    // Copy rwdata into RAM
    mov edi, offset __start_data_ram
    mov esi, offset __start_data_rom

    mov ecx, offset __data_size
    shr ecx, 2

    rep movsd

    // Zero BSS
    xor eax, eax
    mov edi, offset __start_bss_ram

    mov ecx, offset __bss_size
    shr ecx, 2

    rep stosw

    mov esp, offset 0x490000
    mov ebp, esp

    // Done with ROM code, start the kernel
    jmp kenter

.section .hi_rom, "ax"

.code32

start32:
    mov eax, offset 0x10
    mov ds, eax
    mov es, eax
    mov ss, eax
    mov fs, eax
    mov gs, eax

    mov esp, offset 0x490000
    call run_xcodes

    // Clear MTRRs
    // WB caching with valid bit
    xor ecx, ecx
    mov ebx, offset 0x806
    mov ch, 0x2
mtrr_clear:
    xor eax, eax
    xor edx, edx
    wrmsr
    inc ecx
    cmp cl, 0xf
    jna mtrr_clear

    mov cl, 0xff
    mov eax, ebx
    wrmsr

    // Clear CD, NW
    mov eax, cr0
    and eax, 0x9fffffff
    mov cr0, eax

    jmp boot_startup

.code16

start16:
    .byte 0x66
    lgdt [cs:GDTR]

    mov eax, cr0
    or al, 1
    mov cr0, eax

    .byte 0x66
    .code32
    ljmp 0x8, start32

.section .reset, "ax"
.global reset_vector
.code16

reset_vector:
    cli
    jmp short start16
