/* SPDX-License-Identifier: GPL-2.0-only WITH Linux-syscall-note */
.global mcpx_enter

.section .low_rom, "ax"

mcpx_enter:
    jmp kenter
