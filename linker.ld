ENTRY(mcpx_enter)

ROM_SIZE = 256K;
LOW_ROM = 0xFFFC0000;

MEMORY {
    rom (rx) : ORIGIN = LOW_ROM, LENGTH = ROM_SIZE
    ram (rwx) : ORIGIN = 0, LENGTH = 64M
}

RAM_CODE = 0x00100000;

MCPX_BASE = 0xFFFFFE00;
RESET_VECTOR = 0xFFFFFFF0;

SECTIONS {
    /DISCARD/ : {
        *(.comment)
    }

    .low_rom (LOW_ROM) : {
        KEEP(*(.low_rom));
        __end_rom = .;
    } > rom

    .text (RAM_CODE) : {
        __start_code = .;
        *(.text)
        *(.text.*)
        *(.eh_frame_hdr)
        *(.eh_frame)
    } > ram AT>rom

    .data (RAM_CODE + SIZEOF(.text)) : {
        *(.data)
        *(.data.*)
        *(.rodata)
        *(.rodata.*)
        __end_code = .;
    } > ram AT>rom

    .bss (NOLOAD) : {
        *(.bss)
        *(.bss.*)
        *(.sbss)
        *(COMMON)
        __end_bss = .;
    } > ram

    /* Place reset vector startup code in MCPX overlap region to save space */
    . = MCPX_BASE;
    .hi_rom (MCPX_BASE) : {
        KEEP(*(.hi_rom))
    } > rom

    .reset (RESET_VECTOR) : {
        KEEP(*(.reset));
        . = 0xffffffff;

        /* This will output the correct ROM size,
         * but due to an LLVM bug won't link until
         * LLVM 16 */
        /*BYTE(0x0);*/
    } > rom
}
