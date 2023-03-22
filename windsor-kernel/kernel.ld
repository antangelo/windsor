ENTRY(kenter)

ROM_SIZE = 256K;
LOW_ROM = 0xFFFC0000;

RAM_CODE = 0x80010000;

MEMORY {
    rom (rx) : ORIGIN = LOW_ROM, LENGTH = ROM_SIZE
    notram (rwx) : ORIGIN = 4M, LENGTH = 64M
    ram(rwx) : ORIGIN = RAM_CODE, LENGTH = 4M
}

SECTIONS {
    /DISCARD/ : {
        *(.comment)
    }

    .text (RAM_CODE) : {
        *(.text)
        *(.text.*)
        *(.eh_frame_hdr)
        *(.eh_frame)
        . = ALIGN(16);
    } > ram

    .data (RAM_CODE + SIZEOF(.text)) : {
        *(.data)
        *(.data.*)
        *(.rodata)
        *(.rodata.*)
        . = ALIGN(16);
    } > ram

    .bss (NOLOAD) : {
        *(.bss)
        *(.bss.*)
        *(.sbss)
        *(COMMON)
    } > ram

    __start_code_ram = ADDR(.text);
    __start_code_rom = LOADADDR(.text);
    __start_data_ram = ADDR(.data);
    __start_data_rom = LOADADDR(.data);
    __start_bss_ram = ADDR(.bss);
    __code_size = SIZEOF(.text);
    __data_size = SIZEOF(.data);
    __bss_size = SIZEOF(.bss);
}
