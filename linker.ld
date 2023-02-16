/* SPDX-License-Identifier: GPL-2.0-only WITH Linux-syscall-note */
ENTRY(mcpx_enter)

MEMORY {
    rom (rx) : ORIGIN = 0xFFFC0000, LENGTH = (256K + 1)
    ram (rwx) : ORIGIN = 0, LENGTH = 64M
}

ROM_SIZE = 256K;
LOW_ROM = 0xFFFC0000;
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
    } > ram AT>rom

    .bss (RAM_CODE + SIZEOF(.text) + SIZEOF(.data)) : {
        *(.bss)
        *(.bss.*)
    } > ram AT>rom

    /* Place reset vector startup code in MCPX overlap region to save space */
    . = MCPX_BASE;
    .hi_rom (MCPX_BASE) : {
        KEEP(*(.hi_rom))
    } > rom
 
    .reset (RESET_VECTOR) : {
        KEEP(*(.reset));
        . = 0xFFFFFFFF;
    } > rom
}
