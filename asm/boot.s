/* SPDX-License-Identifier: GPL-2.0-only WITH Linux-syscall-note */
.intel_syntax noprefix

.section .hi_rom, "ax"

.code32

start32:
    jmp kenter

.code16

start16:
    mov eax, cr0
    or al, 1
    mov cr0, eax

    jmp cs:start32

.section .reset, "ax"
.global reset_vector

reset_vector:
    cli
    jmp start16
