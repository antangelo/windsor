ENTRY(reset_vector)

MEMORY {
    rom (rx) : ORIGIN = 0xFFFC0000, LENGTH = 256K - 512
    mcpx_overlap (rx) : ORIGIN = 0xFFFFFE00, LENGTH = 512
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

    .low_rom (LOW_ROM) : AT (0x0) {
        KEEP(*(.low_rom));
        __end_rom = .;
    }

    .text (RAM_CODE) : AT (SIZEOF(.low_rom)) {
        *(.text)
        *(.text.*)
    }

    .data (RAM_CODE + SIZEOF(.text)) :
        AT (ADDR(.text) + SIZEOF(.text)) {
        *(.data)
    }

    .bss (RAM_CODE + SIZEOF(.text) + SIZEOF(.data)) :
        AT (ADDR(.data) + SIZEOF(.data)) {
        *(.bss)
    }

    /* Place reset vector startup code in MCPX overlap region to save space */
    .hi_rom (MCPX_BASE) :
        AT (ADDR(.bss) + SIZEOF(.bss)) {
        KEEP(*(.hi_rom));
    } > mcpx_overlap

    .reset (RESET_VECTOR) :
        AT (ADDR(.hi_rom) + SIZEOF(.hi_rom)) {
        KEEP(*(.reset));
    } > mcpx_overlap
}
