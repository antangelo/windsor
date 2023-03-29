use core::arch::global_asm;

global_asm!(include_str!("xcodes.s"), options(att_syntax));
global_asm!(include_str!("gdt.s"), options(att_syntax));
global_asm!(include_str!("boot.s"), options(att_syntax));
