.intel_syntax noprefix

.section .text

.macro make_irq ent
    cli
    pusha
    pushf

    call irq_\ent
    mov al, 0x20
    outb 0x20, al

    popf
    popa
    iret

.endm

.global irq_entry_0
.global irq_0
irq_entry_0:
    make_irq 0

.global irq_entry_1
.global irq_1
irq_entry_1:
    make_irq 1

.global irq_entry_2
.global irq_2
irq_entry_2:
    make_irq 2

.global irq_entry_3
.global irq_3
irq_entry_3:
    make_irq 3

.global irq_entry_4
.global irq_4
irq_entry_4:
    make_irq 4

.global irq_entry_5
.global irq_5
irq_entry_5:
    make_irq 5

.global irq_entry_6
.global irq_6
irq_entry_6:
    make_irq 6

.global irq_entry_7
.global irq_7
irq_entry_7:
    make_irq 7

.global irq_entry_8
.global irq_8
irq_entry_8:
    make_irq 8
