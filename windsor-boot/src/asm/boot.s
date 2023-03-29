.global boot_startup
.global mcpx_enter

.section .low_rom, "ax"

mcpx_enter:
    jmp boot_startup

boot_startup:
    // Copy rwdata into RAM
    mov $__start_data_ram, %edi
    mov $__start_data_rom, %esi

    mov $__data_size, %ecx
    shr $2, %ecx

    rep movsl

    // Zero BSS
    xor %eax, %eax
    mov $__start_bss_ram, %edi

    mov $__bss_size, %ecx
    shr $2, %ecx

    rep stosl

    mov $0x490000, %esp
    mov %esp, %ebp

    // Done with ROM code, start the kernel
    jmp kenter

.section .hi_rom, "ax"

.code32

start32:
    mov $0x10, %eax
    mov %eax, %ds
    mov %eax, %es
    mov %eax, %ss
    mov %eax, %fs
    mov %eax, %gs

    mov $0x490000, %esp
    call run_xcodes

    // Clear MTRRs
    // WB caching with valid bit
    xor %ecx, %ecx
    mov $0x806, %ebx
    mov $0x2, %ch

mtrr_clear:
    xor %eax, %eax
    xor %edx, %edx
    wrmsr
    inc %ecx
    cmp $0xf, %cl
    jna mtrr_clear

    mov $0xff, %cl
    mov %ebx, %eax
    wrmsr

    // Clear CD, NW
    mov %cr0, %eax
    and $0x9fffffff, %eax
    mov %eax, %cr0

    jmp boot_startup

.code16

.global start16
start16:
    lgdtl %cs:GDTR

    mov %cr0, %eax
    orb $1, %al
    mov %eax, %cr0

    ljmpl $0x8, $start32

.section .reset, "ax"
.global reset_vector
.code16

reset_vector:
    cli
    jmp start16

// In case any assembly files are loaded after this one in the module,
// this setting will persist
.code32
