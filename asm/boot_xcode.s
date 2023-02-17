.global mcpx_enter

.section .low_rom, "ax"

.org 0x0
/* MCPX Magic Values - clock timings*/

.long 0xff000009
.long 0xff000008
.long 0x2b16d065
.long 0x3346322d
.long 0x01010101
.long 0x08080808
.long 0x00000801

.long 0xc8fc7c8a	
.long 0x44290213
.long 0x90004998
.long 0x00000000

.long 0xffffffff
.long 0xffffffff

.org 0x40
.long 0	

.org 0x6c
.long 0x00000107

.org 0x70		
.long 0x0000000f
.long 0x40004400

.long 0x12d10070
.long 0x00000c90

.macro xc_peek v1
.byte 0x2
.long \v1
.long 0x0
.endm

.macro xc_poke v1 v2
.byte 0x3;
.long \v1
.long \v2
.endm

.macro xc_pci_out v1 v2
.byte 0x4
.long \v1
.long \v2
.endm

.macro xc_pci_in v1
.byte 0x5
.long \v1
.long 0x0
.endm

.macro xc_bittoggle v1 v2
.byte 0x6
.long \v1
.long \v2
.endm

.macro xc_ifgoto v1 v2
.byte 0x8
.long \v1
.long (9 * (\v2 - 1))
.endm

.macro xc_outb v1 v2
.byte 0x11
.long \v1
.long \v2
.endm

.macro xc_inb v1
.byte 0x12
.long \v1
.long 0x0
.endm

.macro xc_poke_a v1
.byte 0x7
.long 0x3
.long \v1
.endm

.macro xc_pciout_a v1
.byte 0x7
.long 0x4
.long \v1
.endm

.macro xc_outb_a v1
.byte 0x7
.long 0x11
.long \v1
.endm

.macro xc_goto v1
.byte 0x9
.long 0x0
.long (9 * (\v1 - 1))
.endm

.macro xc_end v1
.byte 0xee
.long \v1
.long 0x0
.endm

.macro smb_xc_write v1 v2
xc_outb 0xc008, \v1
xc_outb 0xc006, \v2
xc_outb 0xc002, 0xa
xc_inb 0xc000
xc_ifgoto 0x10, -1
xc_outb 0xc000, 0x10
.endm

.org 0x80
xc_pci_out 0x80000884, 0x8001
xc_pci_out 0x80000810, 0x8001
xc_pci_out 0x80000804, 0x3
xc_outb 0x8049, 0x8
xc_outb 0x80d9, 0x0
xc_outb 0x8026, 0x1
xc_pci_out 0x8000f04c, 0x1
xc_pci_out 0x8000f018, 0x10100
xc_pci_out 0x80000084, 0x7ffffff

xc_pci_out 0x8000f020, 0x0ff00f00
xc_pci_out 0x8000f024, 0xf7f0f000
xc_pci_out 0x80010010, 0x0f000000
xc_pci_out 0x80010014, 0xf0000000
xc_pci_out 0x80010004, 0x00000007
xc_pci_out 0x8000f004, 0x00000007

xc_poke 0x0f0010b0, 0x07633461

xc_poke 0x0f0010cc, 0x66660000

xc_peek 0x0f101000

xc_bittoggle 0x000c0000, 0x0

xc_ifgoto 0x0, 6

xc_peek 0x0f101000

xc_bittoggle 0xe1f3ffff, 0x80000000

xc_poke_a 0x0f101000
xc_poke 0x0f0010b8, 0xeeee0000
xc_goto 11

xc_ifgoto 0x000c0000, 6

xc_peek 0x0f101000
xc_bittoggle 0xe1f3ffff, 0x860c0000
xc_poke_a 0x0f101000

xc_poke 0x0f0010b8, 0xffff0000
xc_goto 5
xc_peek 0x0f101000

xc_bittoggle 0xe1f3ffff, 0x820c0000
xc_poke_a 0x0f101000
xc_poke 0x0f0010b8, 0x11110000
xc_poke 0x0f0010d4, 0x9
xc_poke 0x0f0010b4, 0x0
xc_poke 0x0f0010bc, 0x5866
xc_poke 0x0f0010c4, 0x351c858
xc_poke 0x0f0010c8, 0x30007d67
xc_poke 0x0f0010d8, 0x0
xc_poke 0x0f0010dc, 0xa0423635
xc_poke 0x0f0010e8, 0xc6558c6
xc_poke 0x0f100200, 0x3070103
xc_poke 0x0f100410, 0x11000016
xc_poke 0x0f100330, 0x84848888
xc_poke 0x0f10032c, 0xffffcfff
xc_poke 0x0f100328, 0x1
xc_poke 0x0f100338, 0xdf

xc_pci_out 0x80000904, 0x1
xc_pci_out 0x80000914, 0xc001
xc_pci_out 0x80000918, 0xc201
xc_outb 0xc200, 0x70

// Conexant
xc_outb 0xc004, 0x8a
xc_outb 0xc008, 0xba
xc_outb 0xc006, 0x3f
xc_outb 0xc002, 0xa

xc_inb 0xc000
xc_ifgoto 0x10, 2
xc_goto 4
xc_bittoggle 0x8, 0x0
xc_ifgoto 0x0, -4
xc_goto 39
xc_outb 0xc000, 0x10
smb_xc_write 0x6c, 0x46

smb_xc_write 0xb8, 0x0
smb_xc_write 0xce, 0x19
smb_xc_write 0xc6, 0x9c
smb_xc_write 0x32, 0x8
smb_xc_write 0xc4, 0x1

xc_goto 36

// Focus
xc_outb 0xc000, 0xff
xc_outb 0xc000, 0x10

xc_outb 0xc004, 0xd4
xc_outb 0xc008, 0xc
xc_outb 0xc006, 0x0
xc_outb 0xc002, 0xa
xc_inb 0xc000
xc_ifgoto 0x10, 2
xc_goto 4
xc_bittoggle 0x8, 0x0
xc_ifgoto 0x0, -4
xc_goto 9
xc_outb 0xc000, 0x10
smb_xc_write 0xd, 0x20
xc_goto 16

// Xcalibur
xc_outb 0xc000, 0xff
xc_outb 0xc000, 0x10
xc_outb 0xc004, 0xe0
smb_xc_write 0x0, 0x0
smb_xc_write 0xb8, 0x0

xc_outb 0xc004, 0x20
smb_xc_write 0x1, 0x0

xc_outb 0xc004, 0x21

xc_outb 0xc008, 0x1
xc_outb 0xc002, 0xa
xc_inb 0xc000
xc_ifgoto 0x10, -1
xc_outb 0xc000, 0x10

xc_inb 0xc006

xc_poke 0x0f680500, 0x11c01
xc_poke 0x0f68050c, 0xa0400
xc_poke 0x0f001220, 0x0
xc_poke 0x0f001228, 0x0
xc_poke 0x0f001264, 0x0
xc_poke 0x0f001210, 0x10
xc_peek 0x0f101000
xc_bittoggle 0x06000000, 0x0
xc_ifgoto 0x00000000, 4  
xc_poke 0x0f001214, 0x48480848
xc_poke 0x0f00122c, 0x88888888
xc_goto 7
xc_ifgoto 0x06000000,4
xc_poke 0x0f001214, 0x09090909
xc_poke 0x0f00122c, 0xaaaaaaaa
xc_goto 3

xc_poke 0x0f001214, 0x09090909
xc_poke 0x0f00122c, 0xaaaaaaaa
xc_poke 0x0f001230, 0xffffffff
xc_poke 0x0f001234, 0xaaaaaaaa
xc_poke 0x0f001238, 0xaaaaaaaa
xc_poke 0x0f00123c, 0x8b8b8b8b
xc_poke 0x0f001240, 0xffffffff
xc_poke 0x0f001244, 0x8b8b8b8b
xc_poke 0x0f001248, 0x8b8b8b8b
xc_poke 0x0f1002d4, 0x1
xc_poke 0x0f1002c4, 0x100042
xc_poke 0x0f1002cc, 0x100042
xc_poke 0x0f1002c0, 0x11
xc_poke 0x0f1002c8, 0x11
xc_poke 0x0f1002c0, 0x32
xc_poke 0x0f1002c8, 0x32
xc_poke 0x0f1002c0, 0x132
xc_poke 0x0f1002c8, 0x132
xc_poke 0x0f1002d0, 0x1
xc_poke 0x0f1002d0, 0x1
xc_poke 0x0f100210, 0x80000000
xc_poke 0x0f00124c, 0xaa8baa8b
xc_poke 0x0f001250, 0xaa8b
xc_poke 0x0f100228, 0x081205ff

xc_poke 0x0f001218, 0x10000


xc_pci_in 0x80000860
xc_bittoggle 0xffffffff,0x00000400
xc_pciout_a 0x80000860

xc_pci_out 0x8000084c, 0x0000fdde
xc_pci_out 0x8000089c, 0x871cc707
xc_pci_in 0x800008b4
xc_bittoggle 0xfffff0ff, 0xf00
xc_pciout_a 0x800008b4
xc_pci_out 0x80000340, 0xf0f0c0c0
xc_pci_out 0x80000344, 0xc00000
xc_pci_out 0x8000035c, 0x4070000
xc_pci_out 0x8000036c, 0x230801
xc_pci_out 0x8000036c, 0x1230801
xc_goto 1
xc_goto 1
xc_poke 0x0f100200, 0x03070103
xc_poke 0x0f100204, 0x11448000
xc_pci_out 0x8000103c, 0x0

xc_outb 0xc000, 0x10
xc_outb 0xc004, 0x20

smb_xc_write 0x13, 0xf
smb_xc_write 0x12, 0xf0

xc_pci_out 0x8000f020, 0xfdf0fd00
xc_pci_out 0x80010010, 0xfd000000

xc_poke 0x0, 0xfc1000b8
xc_poke 0x4, 0x90e0ffff
xc_end 0x806

.org 0x1000

mcpx_enter:
    jmp kenter
