ENTRY(mcpx_enter)

ROM_SIZE = 256K;
LOW_ROM = 0xFFFC0000;

MEMORY {
    rom (rx) : ORIGIN = LOW_ROM, LENGTH = ROM_SIZE
    ram (rwx) : ORIGIN = 0, LENGTH = 64M
}

RAM_CODE = 0x00400000;

MCPX_BASE = 0xFFFFFE00;
RESET_VECTOR = 0xFFFFFFF0;

SECTIONS {
    /DISCARD/ : {
        *(.comment)
    }

    .low_rom (LOW_ROM) : {
        KEEP(*(.low_rom));
        . = ALIGN(16);
    } > rom

    .text (LOW_ROM + SIZEOF(.low_rom)) : {
        *(.text)
        *(.text.*)
        *(.eh_frame_hdr)
        *(.eh_frame)
        . = ALIGN(16);
    } > rom

    .rodata (LOW_ROM + SIZEOF(.low_rom) + SIZEOF(.text)) : {
        *(.rodata)
        *(.rodata.*)
        . = ALIGN(16);
    } > rom

    .data (RAM_CODE) : {
        *(.data)
        *(.data.*)
        . = ALIGN(16);
    } > ram AT>rom

    .bss (NOLOAD) : {
        *(.bss)
        *(.bss.*)
        *(.sbss)
        *(COMMON)
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

    __start_data_ram = ADDR(.data);
    __start_data_rom = LOADADDR(.data);
    __start_bss_ram = ADDR(.bss);
    __data_size = SIZEOF(.data);
    __bss_size = SIZEOF(.bss);
}
